//! Monitors and windows, read from AppKit and the Accessibility API.

use std::path::Path;
use std::ptr::NonNull;

use dispatch2::run_on_main;
use objc2::MainThreadMarker;
use objc2_app_kit::{NSRunningApplication, NSScreen};
use objc2_application_services::AXError;
use objc2_core_foundation::{CFRetained, CFUUID, CGFloat, CGPoint, CGSize};
use objc2_foundation::{NSNumber, NSRect, NSString};
use views_platform::{Monitor, PlaceError, Rect, Window, WindowId};

use crate::{app, ax};

unsafe extern "C" {
    /// Not in `objc2-core-graphics`; the framework behind it is linked already.
    fn CGDisplayCreateUUIDFromDisplayID(display: u32) -> Option<NonNull<CFUUID>>;
}

/// Everything on macOS is measured in points, which `Rect` calls pixels.
pub struct Desktop;

impl views_platform::Desktop for Desktop {
    fn monitors(&self) -> Vec<Monitor> {
        run_on_main(monitors)
    }

    fn windows(&self) -> Vec<WindowId> {
        app::pids().into_iter().flat_map(ax::windows).filter_map(|window| ax::id(&window)).collect()
    }

    fn window(&self, window: WindowId) -> Option<Window> {
        let element = ax::element(window)?;
        let app = NSRunningApplication::runningApplicationWithProcessIdentifier(ax::pid(window))?;
        let path = app.executableURL()?.path()?.to_string();
        let exe = Path::new(&path).file_name()?.to_string_lossy().into_owned();
        let title = ax::string(&element, "AXTitle").unwrap_or_default();
        Some(Window { exe, path, class: String::new(), title })
    }

    fn is_tileable(&self, window: WindowId) -> bool {
        let Some(element) = ax::element(window) else {
            return false;
        };
        let on = |attribute| ax::flag(&element, attribute).unwrap_or(false);
        ax::string(&element, "AXSubrole").is_some_and(|subrole| subrole == "AXStandardWindow")
            && !on("AXMinimized")
            && !on("AXFullScreen")
    }

    fn place(&self, window: WindowId, target: Rect) -> Result<(), PlaceError> {
        let element = ax::element(window).ok_or(PlaceError::Gone)?;
        let size = CGSize { width: target.width().into(), height: target.height().into() };
        let origin = CGPoint { x: target.left.into(), y: target.top.into() };
        // Some apps cap the first resize to the room left at the old position.
        let results = [
            ax::set_size(&element, size),
            ax::set_position(&element, origin),
            ax::set_size(&element, size),
        ];
        if results.contains(&AXError::InvalidUIElement) { Err(PlaceError::Gone) } else { Ok(()) }
    }
}

fn monitors(mtm: MainThreadMarker) -> Vec<Monitor> {
    let screens = NSScreen::screens(mtm);
    // The first screen is the primary one: its bottom left is AppKit's origin.
    let Some(primary) = screens.firstObject() else {
        return Vec::new();
    };
    let height = primary.frame().size.height;
    screens
        .iter()
        .filter_map(|screen| {
            Some(Monitor {
                id: uuid(&screen)?,
                name: screen.localizedName().to_string(),
                bounds: top_left(screen.frame(), height),
                work_area: top_left(screen.visibleFrame(), height),
            })
        })
        .collect()
}

/// Flips `frame` from AppKit's bottom-left origin to the top-left one, given the
/// height of the primary screen.
fn top_left(frame: NSRect, height: CGFloat) -> Rect {
    Rect {
        left: frame.origin.x as i32,
        top: (height - frame.origin.y - frame.size.height) as i32,
        right: (frame.origin.x + frame.size.width) as i32,
        bottom: (height - frame.origin.y) as i32,
    }
}

/// The display's UUID, which survives restarts and reconnects, unlike its display id.
fn uuid(screen: &NSScreen) -> Option<String> {
    let number = screen.deviceDescription().objectForKey(&NSString::from_str("NSScreenNumber"))?;
    let display = number.downcast::<NSNumber>().ok()?.unsignedIntValue();
    // SAFETY: any display id is valid input; an unknown one yields null.
    let uuid = unsafe { CGDisplayCreateUUIDFromDisplayID(display) }?;
    // SAFETY: the function follows the Create rule.
    let uuid = unsafe { CFRetained::from_raw(uuid) };
    Some(CFUUID::new_string(None, Some(&uuid))?.to_string())
}
