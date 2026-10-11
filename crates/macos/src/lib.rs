//! The macos backend. Not yet implemented: every query comes back empty.

#![cfg(target_os = "macos")]

use std::ops::ControlFlow;
use std::path::PathBuf;
use std::time::Duration;

use views_platform::{Event, Interest, Monitor, PlaceError, Rect, Status, Window, WindowId};

pub struct Desktop;

impl views_platform::Desktop for Desktop {
    fn monitors(&self) -> Vec<Monitor> {
        Vec::new()
    }

    fn windows(&self) -> Vec<WindowId> {
        Vec::new()
    }

    fn window(&self, _: WindowId) -> Option<Window> {
        None
    }

    fn is_tileable(&self, _: WindowId) -> bool {
        false
    }

    fn place(&self, _: WindowId, _: Rect) -> Result<(), PlaceError> {
        Err(PlaceError::Gone)
    }
}

pub struct Host;

impl Host {
    /// The host for this session's Engine, or `None` if an Engine is already running.
    pub fn new() -> Option<Self> {
        None
    }
}

impl views_platform::Host for Host {
    fn run(&mut self, _: impl FnMut(&mut Self, Event) -> ControlFlow<()>) {}

    fn watch(&mut self, _: Interest) {}

    fn defer(&mut self, _: WindowId, _: Duration) {}

    fn watch_exit(&mut self, _: WindowId) -> bool {
        false
    }

    fn fullscreen_game(&self) -> bool {
        false
    }

    fn trim_memory(&mut self) {}

    fn show(&mut self, _: &Status) {}

    fn open_settings(&mut self) {}
}

/// Where the config file lives.
pub fn config_path() -> PathBuf {
    PathBuf::new()
}

/// Asks the running Engine to reload its config. Returns whether one was there to ask.
pub fn request_reload() -> bool {
    false
}
