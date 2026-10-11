//! Monitors and windows, as Win32 reports them.
//!
//! The Engine's manifest makes it Per-Monitor V2 DPI aware, so every coordinate here
//! is a physical pixel.

use std::ffi::c_void;

use views_platform::{Monitor, PlaceError, Rect, Window, WindowId};
use windows::Win32::Foundation::{E_ACCESSDENIED, HANDLE, HWND, LPARAM, RECT};
use windows::Win32::Graphics::Dwm::{
    DWMWA_CLOAKED, DWMWA_EXTENDED_FRAME_BOUNDS, DwmGetWindowAttribute,
};
use windows::Win32::Graphics::Gdi::{
    DISPLAY_DEVICEW, EnumDisplayDevicesW, EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR,
    MONITOR_DEFAULTTONEAREST, MONITORINFO, MONITORINFOEXW, MonitorFromWindow,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_ACCESS_RIGHTS, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION,
    QueryFullProcessImageNameW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    EDD_GET_DEVICE_INTERFACE_NAME, EnumWindows, FindWindowExW, GW_OWNER, GWL_EXSTYLE, GWL_STYLE,
    GetClassNameW, GetWindow, GetWindowLongW, GetWindowRect, GetWindowTextLengthW, GetWindowTextW,
    GetWindowThreadProcessId, IsZoomed, SW_SHOWNOACTIVATE, SWP_NOACTIVATE, SWP_NOZORDER,
    SetWindowPos, ShowWindow, WINDOW_EX_STYLE, WINDOW_STYLE, WS_CAPTION, WS_CHILD,
    WS_EX_TOOLWINDOW, WS_MINIMIZE, WS_VISIBLE,
};
use windows::core::{BOOL, Owned, PCWSTR, PWSTR, w};

/// The desktop of the current session.
pub struct Desktop;

impl views_platform::Desktop for Desktop {
    fn monitors(&self) -> Vec<Monitor> {
        let mut monitors = Vec::<Monitor>::new();
        let list = LPARAM(&raw mut monitors as isize);
        // SAFETY: `add_monitor` runs only during this call, while `monitors` is alive.
        let _ = unsafe { EnumDisplayMonitors(None, None, Some(add_monitor), list) };
        monitors
    }

    fn windows(&self) -> Vec<WindowId> {
        let mut windows = Vec::<WindowId>::new();
        let list = LPARAM(&raw mut windows as isize);
        // SAFETY: `add_window` runs only during this call, while `windows` is alive.
        let _ = unsafe { EnumWindows(Some(add_window), list) };
        windows
    }

    fn window(&self, window: WindowId) -> Option<Window> {
        let window = hwnd(window);
        let (_, path) = owner(window, PROCESS_ACCESS_RIGHTS::default())?;

        let mut class = [0; 256];
        let length = unsafe { GetClassNameW(window, &mut class) };
        let class = String::from_utf16_lossy(&class[..length as usize]);

        let mut title = vec![0; unsafe { GetWindowTextLengthW(window) } as usize + 1];
        let length = unsafe { GetWindowTextW(window, &mut title) };
        let title = String::from_utf16_lossy(&title[..length as usize]);

        Some(Window { exe: file_name(&path).to_owned(), path, class, title })
    }

    fn is_tileable(&self, window: WindowId) -> bool {
        let window = hwnd(window);
        let style = WINDOW_STYLE(unsafe { GetWindowLongW(window, GWL_STYLE) } as u32);
        let extended = WINDOW_EX_STYLE(unsafe { GetWindowLongW(window, GWL_EXSTYLE) } as u32);
        let has = |flag| style.contains(flag);

        has(WS_VISIBLE)
            && has(WS_CAPTION)
            && !has(WS_CHILD)
            && !has(WS_MINIMIZE)
            && !extended.contains(WS_EX_TOOLWINDOW)
            && unsafe { GetWindow(window, GW_OWNER) }.is_err()
            && unsafe { GetWindowTextLengthW(window) } > 0
            && !is_cloaked(window)
            && !is_fullscreen(window)
    }

    fn place(&self, window: WindowId, target: Rect) -> Result<(), PlaceError> {
        let window = hwnd(window);
        if unsafe { IsZoomed(window) }.as_bool() {
            // Restores without activating, unlike `SW_RESTORE`.
            let _ = unsafe { ShowWindow(window, SW_SHOWNOACTIVATE) };
        }
        move_frame(window, target)?;
        // A window that lands on a Monitor with another scale has its borders rescaled
        // on arrival, which the first move could not allow for.
        if frame(window) != Some(target) {
            move_frame(window, target)?;
        }
        Ok(())
    }
}

/// The process that owns `window`, opened with `access`, and the path of its executable.
///
/// A UWP app's window belongs to `ApplicationFrameHost.exe`. The app is the process of
/// the `Windows.UI.Core.CoreWindow` inside it, which is absent while the app starts.
pub(crate) fn owner(
    window: HWND,
    access: PROCESS_ACCESS_RIGHTS,
) -> Option<(Owned<HANDLE>, String)> {
    let mut id = 0;
    unsafe { GetWindowThreadProcessId(window, Some(&mut id)) };
    let access = access | PROCESS_QUERY_LIMITED_INFORMATION;
    let process = unsafe { OpenProcess(access, false, id) }.ok()?;
    // SAFETY: the handle is open and nothing else closes it.
    let process = unsafe { Owned::new(process) };

    let mut path = [0; LONGEST_PATH];
    let (name, mut length) = (PWSTR(path.as_mut_ptr()), path.len() as u32);
    unsafe { QueryFullProcessImageNameW(*process, PROCESS_NAME_WIN32, name, &mut length) }.ok()?;
    let path = String::from_utf16_lossy(&path[..length as usize]);

    if file_name(&path).eq_ignore_ascii_case("ApplicationFrameHost.exe") {
        let class = w!("Windows.UI.Core.CoreWindow");
        let core = unsafe { FindWindowExW(Some(window), None, class, PCWSTR::null()) }.ok()?;
        return owner(core, access);
    }
    Some((process, path))
}

pub(crate) fn hwnd(window: WindowId) -> HWND {
    HWND(window.0 as *mut c_void)
}

/// The length Windows allows a path, in UTF-16 units and counting its terminator.
const LONGEST_PATH: usize = 32768;

unsafe extern "system" fn add_monitor(
    monitor: HMONITOR,
    _: HDC,
    _: *mut RECT,
    list: LPARAM,
) -> BOOL {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = size_of_val(&info) as u32;
    if unsafe { GetMonitorInfoW(monitor, (&raw mut info).cast()) }.as_bool() {
        let mut device = DISPLAY_DEVICEW::default();
        device.cb = size_of_val(&device) as u32;
        let adapter = PCWSTR(info.szDevice.as_ptr());
        let flags = EDD_GET_DEVICE_INTERFACE_NAME;
        let _ = unsafe { EnumDisplayDevicesW(adapter, 0, &mut device, flags) };

        // The interface path names the display and the port it is on, so it outlives a
        // restart, which the adapter's `\\.\DISPLAYn` need not. A display with no
        // device behind it, such as a remote session's, has only the latter.
        let id = if device.DeviceID[0] == 0 { &info.szDevice[..] } else { &device.DeviceID };
        // SAFETY: `list` is the `Vec` that `Desktop::monitors` is enumerating into.
        let monitors = unsafe { &mut *(list.0 as *mut Vec<Monitor>) };
        monitors.push(Monitor {
            id: string(id),
            name: string(&device.DeviceString),
            bounds: rect(info.monitorInfo.rcMonitor),
            work_area: rect(info.monitorInfo.rcWork),
        });
    }
    true.into()
}

unsafe extern "system" fn add_window(window: HWND, list: LPARAM) -> BOOL {
    // SAFETY: `list` is the `Vec` that `Desktop::windows` is enumerating into.
    let windows = unsafe { &mut *(list.0 as *mut Vec<WindowId>) };
    windows.push(WindowId(window.0 as usize));
    true.into()
}

/// Whether the compositor is hiding `window`, as it does one on another virtual
/// desktop or a suspended UWP app's.
fn is_cloaked(window: HWND) -> bool {
    let mut cloaked = 0u32;
    let (attribute, size) = (DWMWA_CLOAKED, size_of_val(&cloaked) as u32);
    unsafe { DwmGetWindowAttribute(window, attribute, (&raw mut cloaked).cast(), size) }.is_err()
        || cloaked != 0
}

/// Whether `window` covers its whole monitor without being maximized. A maximized
/// window covers it too wherever there is no taskbar.
fn is_fullscreen(window: HWND) -> bool {
    let mut info = MONITORINFO { cbSize: size_of::<MONITORINFO>() as u32, ..Default::default() };
    let monitor = unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) };
    unsafe { GetMonitorInfoW(monitor, &mut info) }.as_bool()
        && !unsafe { IsZoomed(window) }.as_bool()
        && bounds(window).is_some_and(|window| {
            let screen = info.rcMonitor;
            window.left <= screen.left
                && window.top <= screen.top
                && window.right >= screen.right
                && window.bottom >= screen.bottom
        })
}

/// Moves `window` so that its frame fills `target`.
fn move_frame(window: HWND, target: Rect) -> Result<(), PlaceError> {
    // A window's bounds extend past the frame the user sees, by resize borders that
    // are invisible but still part of the window.
    let (Some(bounds), Some(frame)) = (bounds(window), frame(window)) else {
        return Err(PlaceError::Gone);
    };
    let x = target.left + bounds.left - frame.left;
    let y = target.top + bounds.top - frame.top;
    let width = target.width() + bounds.width() - frame.width();
    let height = target.height() + bounds.height() - frame.height();
    let flags = SWP_NOZORDER | SWP_NOACTIVATE;
    unsafe { SetWindowPos(window, None, x, y, width, height, flags) }.map_err(|error| {
        // Windows will not let a process move the window of a more privileged one.
        if error.code() == E_ACCESSDENIED { PlaceError::Elevated } else { PlaceError::Gone }
    })
}

/// The whole of `window`, invisible borders included.
fn bounds(window: HWND) -> Option<Rect> {
    let mut bounds = RECT::default();
    unsafe { GetWindowRect(window, &mut bounds) }.ok()?;
    Some(rect(bounds))
}

/// The part of `window` that the user sees.
fn frame(window: HWND) -> Option<Rect> {
    let mut frame = RECT::default();
    let (attribute, size) = (DWMWA_EXTENDED_FRAME_BOUNDS, size_of_val(&frame) as u32);
    unsafe { DwmGetWindowAttribute(window, attribute, (&raw mut frame).cast(), size) }.ok()?;
    Some(rect(frame))
}

fn rect(rect: RECT) -> Rect {
    Rect { left: rect.left, top: rect.top, right: rect.right, bottom: rect.bottom }
}

/// The text in a zero-terminated buffer.
fn string(buffer: &[u16]) -> String {
    let length = buffer.iter().position(|&unit| unit == 0).unwrap_or(buffer.len());
    String::from_utf16_lossy(&buffer[..length])
}

fn file_name(path: &str) -> &str {
    path.rsplit_once('\\').map_or(path, |(_, name)| name)
}
