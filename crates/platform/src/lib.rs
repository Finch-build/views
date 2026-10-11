//! The seam between the Engine and an operating system.
//!
//! A backend implements two traits. [`Desktop`] answers questions about monitors and
//! windows and moves a window; it holds no state, so the Settings UI and the debug CLI
//! use it too. [`Host`] is what only a running Engine needs: its event loop, its tray
//! icon and its timers.

use std::ops::ControlFlow;
use std::time::Duration;

pub use views_core::{Monitor, Rect, Window};

/// Identifies a window to the backend that produced it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct WindowId(pub usize);

/// The monitors and windows of the current session.
pub trait Desktop {
    /// The attached Monitors.
    fn monitors(&self) -> Vec<Monitor>;

    /// Every top-level window, Tileable or not.
    fn windows(&self) -> Vec<WindowId>;

    /// What a Rule can match `window` against. `None` if the window is gone, belongs
    /// to a process this one may not inspect, or is still being set up by its app.
    fn window(&self, window: WindowId) -> Option<Window>;

    /// Whether `window` is a Tileable window.
    fn is_tileable(&self, window: WindowId) -> bool;

    /// Moves and resizes `window` so that its visible frame fills `target`.
    fn place(&self, window: WindowId, target: Rect) -> Result<(), PlaceError>;
}

/// Why a window could not be placed.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaceError {
    /// The window belongs to a process with more privilege than the Engine.
    Elevated,
    /// The window no longer exists.
    Gone,
}

/// The Engine's event loop and its presence in the tray.
pub trait Host {
    /// Runs the event loop on the calling thread, handing each event to `handler`,
    /// until `handler` breaks or the session ends.
    fn run(&mut self, handler: impl FnMut(&mut Self, Event) -> ControlFlow<()>);

    /// Chooses which events are worth waking for. What is not watched costs nothing.
    fn watch(&mut self, interest: Interest);

    /// Delivers [`Event::Deferred`] for `window` once, after `delay`.
    fn defer(&mut self, window: WindowId, delay: Duration);

    /// Delivers [`Event::GameExited`] once, when the process that owns `window` exits.
    /// Returns whether that process could be watched.
    fn watch_exit(&mut self, window: WindowId) -> bool;

    /// Whether an app is running exclusive-fullscreen Direct3D.
    fn fullscreen_game(&self) -> bool;

    /// Hands the Engine's resident memory back to the system.
    fn trim_memory(&mut self);

    /// Reflects `status` in the tray icon and its menu.
    fn show(&mut self, status: &Status);

    /// Opens the Settings UI.
    fn open_settings(&mut self);
}

/// Something the Engine reacts to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Event {
    /// A top-level window appeared.
    WindowCreated(WindowId),
    /// A delay requested with [`Host::defer`] ran out.
    Deferred(WindowId),
    /// A different window took the foreground.
    ForegroundChanged(WindowId),
    /// The process registered with [`Host::watch_exit`] exited.
    GameExited,
    /// Monitors were attached, detached, rearranged or resized.
    DisplaysChanged,
    /// The Settings UI saved a new config.
    ReloadRequested,
    /// The user chose an item from the tray menu.
    Command(Command),
}

/// An item of the tray menu.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    OpenSettings,
    TogglePause,
    Gaming(Override),
    Quit,
}

/// The user's say over Gaming mode, which outranks detection.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Override {
    /// Gaming mode follows the config: it lasts while a detected game runs.
    #[default]
    Auto,
    /// Gaming mode stays on.
    On,
    /// Gaming mode stays off.
    Off,
}

/// The events a [`Host`] should deliver when they happen.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Interest {
    /// [`Event::WindowCreated`].
    pub windows: bool,
    /// [`Event::ForegroundChanged`].
    pub foreground: bool,
}

/// What the tray shows.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Status {
    pub paused: bool,
    pub gaming: Override,
    /// The tooltip: the app's name and anything the user should know, one item a line.
    pub summary: String,
}
