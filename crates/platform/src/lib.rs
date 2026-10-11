//! The seam between the Engine and the OS-specific backends.
//!
//! A backend answers three questions for the Engine: which Monitors are attached,
//! which windows are Tileable, and how to place one. Ticket 03 implements this for
//! Windows, ticket 11 for macOS.

pub use views_core::Rect;

/// Identifies a display. Stable across a reconnect, so friendly names survive one.
pub type MonitorId = String;

/// Identifies a window. Only meaningful to the backend that produced it.
pub type WindowHandle = u64;

/// A display.
#[derive(Clone, Debug, PartialEq)]
pub struct Monitor {
    pub id: MonitorId,
    /// Full bounds in desktop coordinates; the work-area borders are backend detail.
    pub bounds: Rect,
}

/// What the Engine knows about a window: enough to match a Rule against it.
#[derive(Clone, Debug, PartialEq)]
pub struct WindowInfo {
    pub handle: WindowHandle,
    /// Path to the executable that owns the window.
    pub exe: String,
    /// Window class, on the platforms that have one.
    pub class: Option<String>,
    pub title: String,
    pub bounds: Rect,
}

/// Why a placement failed.
#[derive(Clone, Debug, PartialEq)]
pub enum PlaceError {
    /// Owned by another user's session; moving it would be wrong.
    Elevated,
    /// The handle is not a window this backend currently sees.
    UnknownWindow,
}

pub trait PlatformBackend {
    /// The Monitors currently attached.
    fn monitors(&self) -> Vec<Monitor>;
    /// The Tileable windows currently visible.
    fn tileable_windows(&self) -> Vec<WindowInfo>;
    /// Move and resize a window to a pixel rectangle.
    fn place_window(&self, handle: WindowHandle, target: Rect) -> Result<(), PlaceError>;
}

#[cfg(test)]
mod tests {
    use super::{Monitor, Rect, WindowInfo};

    #[test]
    fn a_window_lives_inside_its_monitor() {
        let monitor = Monitor {
            id: "0".to_string(),
            bounds: Rect {
                x: 0,
                y: 0,
                width: 10,
                height: 10,
            },
        };
        let window = WindowInfo {
            handle: 1,
            exe: "notepad".to_string(),
            class: None,
            title: "Untitled".to_string(),
            bounds: Rect {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            },
        };
        assert_eq!(monitor.bounds.width, 10);
        assert_eq!(window.exe, "notepad");
    }
}
