//! The Engine's message loop, tray icon and timers.

use std::cell::RefCell;
use std::collections::VecDeque;
use std::env;
use std::mem;
use std::ops::ControlFlow;
use std::process;
use std::time::Duration;

use views_platform::{Command, Event, Host as _, Interest, Override, Status, WindowId};
use windows::Win32::Foundation::{
    ERROR_ALREADY_EXISTS, GetLastError, HANDLE, HWND, LPARAM, LRESULT, WAIT_OBJECT_0, WPARAM,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Threading::{
    CreateMutexW, GetCurrentProcess, INFINITE, MEMORY_PRIORITY, MEMORY_PRIORITY_INFORMATION,
    MEMORY_PRIORITY_LOW, MEMORY_PRIORITY_NORMAL, PROCESS_INFORMATION_CLASS,
    PROCESS_POWER_THROTTLING_CURRENT_VERSION, PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
    PROCESS_POWER_THROTTLING_STATE, PROCESS_SYNCHRONIZE, ProcessMemoryPriority,
    ProcessPowerThrottling, SetProcessInformation, SetProcessWorkingSetSize,
};
use windows::Win32::UI::Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent};
use windows::Win32::UI::Shell::{
    NIF_ICON, NIF_MESSAGE, NIF_SHOWTIP, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NIM_SETVERSION,
    NOTIFYICON_VERSION_4, NOTIFYICONDATAW, NOTIFYICONDATAW_0, QUNS_RUNNING_D3D_FULL_SCREEN,
    SHQueryUserNotificationState, Shell_NotifyIconW,
};
use windows::Win32::UI::WindowsAndMessaging::{
    AppendMenuW, CheckMenuItem, CheckMenuRadioItem, CreateIconFromResourceEx, CreatePopupMenu,
    CreateWindowExW, DefWindowProcW, DeregisterShellHookWindow, DestroyIcon, DestroyMenu,
    DestroyWindow, DispatchMessageW, EVENT_SYSTEM_FOREGROUND, GetSystemMetrics, HICON,
    HSHELL_WINDOWCREATED, InsertMenuW, KillTimer, LR_DEFAULTCOLOR, MF_BYCOMMAND, MF_BYPOSITION,
    MF_CHECKED, MF_POPUP, MF_STRING, MSG, MWMO_INPUTAVAILABLE, MsgWaitForMultipleObjectsEx,
    PM_REMOVE, PeekMessageW, PostQuitMessage, QS_ALLINPUT, RegisterClassW, RegisterShellHookWindow,
    RegisterWindowMessageW, SM_CXSMICON, SPI_SETWORKAREA, SetForegroundWindow, SetTimer,
    TPM_RETURNCMD, TPM_RIGHTBUTTON, TrackPopupMenu, USER_TIMER_MAXIMUM, WINEVENT_OUTOFCONTEXT,
    WINEVENT_SKIPOWNPROCESS, WM_APP, WM_CONTEXTMENU, WM_DESTROY, WM_DISPLAYCHANGE, WM_ENDSESSION,
    WM_LBUTTONDBLCLK, WM_QUIT, WM_SETTINGCHANGE, WM_TIMER, WNDCLASSW,
};
use windows::core::{Owned, PCWSTR, w};

use crate::desktop;

/// The class of the Engine's window, by which `request_reload` finds it. Also the name
/// of the mutex that keeps the Engine to one instance.
pub(crate) const CLASS: PCWSTR = w!("views.engine");

/// The name of the registered message that asks the Engine to reload its config.
pub(crate) const RELOAD: PCWSTR = w!("views.reload");

/// The message the tray icon sends about itself.
const TRAY: u32 = WM_APP;
/// Recorded by the foreground hook, with the window in `wparam`.
const FOREGROUND: u32 = WM_APP + 1;
/// Recorded when the game's process exits.
const GAME_EXITED: u32 = WM_APP + 2;

/// The tray menu's items, in the order of their identifiers, which start at one.
const ITEMS: [(Command, PCWSTR); 6] = [
    (Command::OpenSettings, w!("Open settings")),
    (Command::TogglePause, w!("Pause")),
    (Command::Gaming(Override::Auto), w!("Auto")),
    (Command::Gaming(Override::On), w!("On")),
    (Command::Gaming(Override::Off), w!("Off")),
    (Command::Quit, w!("Quit")),
];

type Message = (u32, WPARAM, LPARAM);

thread_local! {
    /// What `Host::run` has yet to act on. Windows calls the window procedure and the
    /// foreground hook itself, so neither can reach the `Host`; they leave their
    /// messages here.
    static RECORDED: RefCell<VecDeque<Message>> = const { RefCell::new(VecDeque::new()) };
}

/// The Engine's hidden window and everything that hangs off it.
pub struct Host {
    window: HWND,
    tray: NOTIFYICONDATAW,
    /// Held so that a second Engine in this session finds it taken.
    _instance: Owned<HANDLE>,
    shell_hook: u32,
    taskbar_created: u32,
    reload: u32,
    watching_windows: bool,
    foreground_hook: HWINEVENTHOOK,
    game: Option<Owned<HANDLE>>,
    paused: bool,
    gaming: Override,
}

impl Host {
    /// The host for this session's Engine, or `None` if an Engine is already running.
    pub fn new() -> Option<Self> {
        let instance = unsafe { CreateMutexW(None, false, CLASS) }.ok()?;
        let taken = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
        // SAFETY: the handle is open and nothing else closes it.
        let instance = unsafe { Owned::new(instance) };
        if taken {
            return None;
        }

        let module = unsafe { GetModuleHandleW(None) }.ok()?.into();
        let class = WNDCLASSW {
            lpfnWndProc: Some(on_message),
            hInstance: module,
            lpszClassName: CLASS,
            ..Default::default()
        };
        unsafe { RegisterClassW(&class) };
        // Never shown, but top-level: a message-only window is sent no broadcasts, and
        // both a display change and the taskbar's restart arrive as one.
        let window = unsafe {
            CreateWindowExW(
                Default::default(),
                CLASS,
                PCWSTR::null(),
                Default::default(),
                0,
                0,
                0,
                0,
                None,
                None,
                Some(module),
                None,
            )
        }
        .ok()?;

        // Together these mark the process as background work: scheduled for
        // efficiency, and first to give up its pages when memory is short.
        let throttling = PROCESS_POWER_THROTTLING_STATE {
            Version: PROCESS_POWER_THROTTLING_CURRENT_VERSION,
            ControlMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
            StateMask: PROCESS_POWER_THROTTLING_EXECUTION_SPEED,
        };
        set_process(ProcessPowerThrottling, &throttling);
        set_memory_priority(MEMORY_PRIORITY_LOW);

        let host = Self {
            window,
            tray: NOTIFYICONDATAW {
                cbSize: size_of::<NOTIFYICONDATAW>() as u32,
                hWnd: window,
                uFlags: NIF_MESSAGE | NIF_ICON | NIF_TIP | NIF_SHOWTIP,
                uCallbackMessage: TRAY,
                hIcon: icon(),
                Anonymous: NOTIFYICONDATAW_0 { uVersion: NOTIFYICON_VERSION_4 },
                ..Default::default()
            },
            _instance: instance,
            shell_hook: unsafe { RegisterWindowMessageW(w!("SHELLHOOK")) },
            taskbar_created: unsafe { RegisterWindowMessageW(w!("TaskbarCreated")) },
            reload: unsafe { RegisterWindowMessageW(RELOAD) },
            watching_windows: false,
            foreground_hook: HWINEVENTHOOK::default(),
            game: None,
            paused: false,
            gaming: Override::Auto,
        };
        host.add_icon();
        Some(host)
    }

    fn add_icon(&self) {
        let _ = unsafe { Shell_NotifyIconW(NIM_ADD, &self.tray) };
        let _ = unsafe { Shell_NotifyIconW(NIM_SETVERSION, &self.tray) };
    }

    /// The event that a recorded message stands for, if any.
    fn translate(&mut self, (message, wparam, lparam): Message) -> Option<Event> {
        Some(match message {
            WM_DISPLAYCHANGE | WM_SETTINGCHANGE => Event::DisplaysChanged,
            WM_TIMER => {
                // Timers repeat until killed, and a deferral is delivered once.
                let _ = unsafe { KillTimer(Some(self.window), wparam.0) };
                Event::Deferred(WindowId(wparam.0))
            }
            FOREGROUND => Event::ForegroundChanged(WindowId(wparam.0)),
            GAME_EXITED => Event::GameExited,
            // What happened to the icon is the low word of `lparam`, and where is the
            // two words of `wparam`.
            TRAY => match lparam.0 as u16 as u32 {
                WM_CONTEXTMENU => {
                    let (x, y) = (wparam.0 as i16, (wparam.0 >> 16) as i16);
                    Event::Command(self.menu(x.into(), y.into())?)
                }
                WM_LBUTTONDBLCLK => Event::Command(Command::OpenSettings),
                _ => return None,
            },
            _ if message == self.shell_hook && wparam.0 == HSHELL_WINDOWCREATED as usize => {
                Event::WindowCreated(WindowId(lparam.0 as usize))
            }
            _ if message == self.reload => Event::ReloadRequested,
            // Explorer has restarted and its new taskbar has no icons.
            _ if message == self.taskbar_created => {
                self.add_icon();
                return None;
            }
            _ => return None,
        })
    }

    /// Shows the tray menu at a point on the desktop and waits for the user's choice.
    fn menu(&self, x: i32, y: i32) -> Option<Command> {
        let menu = unsafe { CreatePopupMenu() }.unwrap_or_default();
        let gaming = unsafe { CreatePopupMenu() }.unwrap_or_default();
        for (id, (command, label)) in (1..).zip(ITEMS) {
            let parent = if matches!(command, Command::Gaming(_)) { gaming } else { menu };
            let _ = unsafe { AppendMenuW(parent, MF_STRING, id as usize, label) };
            if command == Command::TogglePause && self.paused {
                unsafe { CheckMenuItem(menu, id, MF_CHECKED.0) };
            } else if command == Command::Gaming(self.gaming) {
                let _ = unsafe { CheckMenuRadioItem(gaming, id, id, id, MF_BYCOMMAND.0) };
            }
        }
        let (flags, label) = (MF_BYPOSITION | MF_POPUP, w!("Gaming mode"));
        let _ = unsafe { InsertMenuW(menu, 2, flags, gaming.0 as usize, label) };

        // A menu whose window is not in the foreground stays open when the user
        // clicks away from it.
        let _ = unsafe { SetForegroundWindow(self.window) };
        let flags = TPM_RETURNCMD | TPM_RIGHTBUTTON;
        let chosen = unsafe { TrackPopupMenu(menu, flags, x, y, None, self.window, None) };
        let _ = unsafe { DestroyMenu(menu) };

        let index = (chosen.0 as usize).checked_sub(1)?;
        Some(ITEMS[index].0)
    }
}

impl views_platform::Host for Host {
    fn run(&mut self, mut handler: impl FnMut(&mut Self, Event) -> ControlFlow<()>) {
        loop {
            let mut message = MSG::default();
            while unsafe { PeekMessageW(&mut message, None, 0, 0, PM_REMOVE) }.as_bool() {
                if message.message == WM_QUIT {
                    return;
                }
                unsafe { DispatchMessageW(&message) };
            }
            // The handler moves windows, which lets Windows call the window procedure
            // again, so the queue is not borrowed while it runs.
            while let Some(message) = RECORDED.with_borrow_mut(VecDeque::pop_front) {
                if let Some(event) = self.translate(message)
                    && handler(self, event).is_break()
                {
                    return;
                }
            }

            let game = self.game.as_deref().copied();
            let (handles, wake, flags) = (game.as_slice(), QS_ALLINPUT, MWMO_INPUTAVAILABLE);
            let woken =
                unsafe { MsgWaitForMultipleObjectsEx(Some(handles), INFINITE, wake, flags) };
            if game.is_some() && woken == WAIT_OBJECT_0 {
                self.game = None;
                record(GAME_EXITED, WPARAM(0), LPARAM(0));
            }
        }
    }

    fn watch(&mut self, interest: Interest) {
        if interest.windows != self.watching_windows {
            self.watching_windows = interest.windows;
            let _ = if interest.windows {
                unsafe { RegisterShellHookWindow(self.window) }
            } else {
                unsafe { DeregisterShellHookWindow(self.window) }
            };
        }
        if interest.foreground && self.foreground_hook.is_invalid() {
            let (event, flags) = (EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT);
            // The tray menu brings the Engine's own window to the foreground.
            let flags = flags | WINEVENT_SKIPOWNPROCESS;
            self.foreground_hook =
                unsafe { SetWinEventHook(event, event, None, Some(on_foreground), 0, 0, flags) };
        } else if !interest.foreground && !self.foreground_hook.is_invalid() {
            let _ = unsafe { UnhookWinEvent(mem::take(&mut self.foreground_hook)) };
        }
    }

    fn defer(&mut self, window: WindowId, delay: Duration) {
        let delay = delay.as_millis().min(USER_TIMER_MAXIMUM.into()) as u32;
        unsafe { SetTimer(Some(self.window), window.0, delay, None) };
    }

    fn watch_exit(&mut self, window: WindowId) -> bool {
        let process = desktop::owner(desktop::hwnd(window), PROCESS_SYNCHRONIZE);
        self.game = process.map(|(process, _)| process);
        self.game.is_some()
    }

    fn fullscreen_game(&self) -> bool {
        let state = unsafe { SHQueryUserNotificationState() };
        state == Ok(QUNS_RUNNING_D3D_FULL_SCREEN)
    }

    fn trim_memory(&mut self) {
        // Both sizes at their maximum is how Windows is asked to empty a working set.
        let _ = unsafe { SetProcessWorkingSetSize(GetCurrentProcess(), usize::MAX, usize::MAX) };
    }

    fn show(&mut self, status: &Status) {
        (self.paused, self.gaming) = (status.paused, status.gaming);
        self.tray.szTip = tip(&status.summary);
        let _ = unsafe { Shell_NotifyIconW(NIM_MODIFY, &self.tray) };
    }

    fn open_settings(&mut self) {
        let Ok(engine) = env::current_exe() else {
            return;
        };
        // A process starts with its parent's memory priority, and the Settings UI is
        // not background work.
        set_memory_priority(MEMORY_PRIORITY_NORMAL);
        let _ = process::Command::new(engine.with_file_name("views-settings.exe")).spawn();
        set_memory_priority(MEMORY_PRIORITY_LOW);
    }
}

impl Drop for Host {
    fn drop(&mut self) {
        self.watch(Interest::default());
        let _ = unsafe { Shell_NotifyIconW(NIM_DELETE, &self.tray) };
        let _ = unsafe { DestroyWindow(self.window) };
        let _ = unsafe { DestroyIcon(self.tray.hIcon) };
    }
}

/// The hidden window's procedure. It records what `Host::run` acts on.
unsafe extern "system" fn on_message(
    window: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match message {
        WM_DESTROY => unsafe { PostQuitMessage(0) },
        WM_ENDSESSION if wparam.0 != 0 => unsafe { PostQuitMessage(0) },
        // Registered messages are numbered from 0xC000: the shell hook's, the
        // taskbar's and the request to reload are all among them.
        WM_DISPLAYCHANGE | WM_TIMER | TRAY | 0xC000..=0xFFFF => record(message, wparam, lparam),
        // A taskbar that moves, resizes or hides changes a work area and no display.
        WM_SETTINGCHANGE if wparam.0 == SPI_SETWORKAREA.0 as usize => {
            record(message, wparam, lparam);
        }
        _ => {}
    }
    unsafe { DefWindowProcW(window, message, wparam, lparam) }
}

/// The foreground hook. Windows calls it on the Engine's thread, from the message loop.
unsafe extern "system" fn on_foreground(
    _: HWINEVENTHOOK,
    _: u32,
    window: HWND,
    _: i32,
    _: i32,
    _: u32,
    _: u32,
) {
    record(FOREGROUND, WPARAM(window.0 as usize), LPARAM(0));
}

fn record(message: u32, wparam: WPARAM, lparam: LPARAM) {
    RECORDED.with_borrow_mut(|recorded| recorded.push_back((message, wparam, lparam)));
}

fn set_memory_priority(priority: MEMORY_PRIORITY) {
    set_process(ProcessMemoryPriority, &MEMORY_PRIORITY_INFORMATION { MemoryPriority: priority });
}

fn set_process<T>(class: PROCESS_INFORMATION_CLASS, information: &T) {
    let (information, size) = ((&raw const *information).cast(), size_of::<T>() as u32);
    let _ = unsafe { SetProcessInformation(GetCurrentProcess(), class, information, size) };
}

/// The image in the embedded icon file that best fits the tray.
fn icon() -> HICON {
    // An .ico file is a six-byte header, whose fifth byte counts the images, and then
    // sixteen bytes for each image: its width first, and the length and the offset of
    // its data last, as two 32-bit integers.
    const FILE: &[u8] = include_bytes!("../assets/tray.ico");
    let size = unsafe { GetSystemMetrics(SM_CXSMICON) };
    let (images, _) = FILE[6..].as_chunks::<16>();
    // The smallest image that need not be stretched or, failing that, the largest.
    let best = images.iter().take(FILE[4].into()).min_by_key(|image| {
        let width = i32::from(image[0]);
        (width < size, width.abs_diff(size))
    });
    let Some(&[.., l0, l1, l2, l3, o0, o1, o2, o3]) = best else {
        return HICON::default();
    };
    let length = u32::from_le_bytes([l0, l1, l2, l3]) as usize;
    let offset = u32::from_le_bytes([o0, o1, o2, o3]) as usize;
    let data = &FILE[offset..][..length];
    unsafe { CreateIconFromResourceEx(data, true, 0x0003_0000, size, size, LR_DEFAULTCOLOR) }
        .unwrap_or_default()
}

/// `text` as a tooltip: as much of it as the icon holds, cut between characters.
fn tip(text: &str) -> [u16; 128] {
    let mut tip = [0; 128];
    let mut length = 0;
    for character in text.chars() {
        // The last unit stays zero, to end the string.
        if length + character.len_utf16() >= tip.len() {
            break;
        }
        length += character.encode_utf16(&mut tip[length..]).len();
    }
    tip
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tooltip_too_long_for_the_icon_is_cut_between_characters() {
        // Each of these is two UTF-16 units, so 127 units end in the middle of one.
        let tip = tip(&"\u{1F426}".repeat(100));
        let length = tip.iter().position(|&unit| unit == 0).unwrap();
        assert_eq!(length, 126);
        assert!(String::from_utf16(&tip[..length]).is_ok());
    }
}
