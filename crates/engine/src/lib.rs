//! The always-running background executable that watches for windows and applies rules.
//!
//! Ticket 04 builds the executable; all that exists now is the seam that picks the
//! backend for the platform this binary was built for.

use views_platform::PlatformBackend;

#[cfg(windows)]
pub fn backend() -> Box<dyn PlatformBackend> {
    Box::new(views_windows::WindowsBackend {})
}

#[cfg(not(windows))]
pub fn backend() -> Box<dyn PlatformBackend> {
    Box::new(views_macos::MacosBackend {})
}
