//! macOS implementation of the platform trait.
//!
//! Stub: it compiles on every target and does nothing. Ticket 11 fills it in.

use views_platform::{Monitor, PlaceError, PlatformBackend, Rect, WindowHandle, WindowInfo};

pub struct MacosBackend {}

impl PlatformBackend for MacosBackend {
    fn monitors(&self) -> Vec<Monitor> {
        Vec::new()
    }

    fn tileable_windows(&self) -> Vec<WindowInfo> {
        Vec::new()
    }

    fn place_window(&self, _handle: WindowHandle, _target: Rect) -> Result<(), PlaceError> {
        Err(PlaceError::UnknownWindow)
    }
}
