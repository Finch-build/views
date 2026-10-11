//! Windows implementation of the platform trait.
//!
//! ponytail: stub until ticket 03 — no OS calls yet, so this compiles on every
//! target and CI stays green on macOS.

use views_platform::{Monitor, PlaceError, PlatformBackend, Rect, WindowHandle, WindowInfo};

pub struct WindowsBackend {}

impl PlatformBackend for WindowsBackend {
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
