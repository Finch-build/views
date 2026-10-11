//! The logic that needs no operating system: the config, Zone geometry and Rule matching.

mod config;
mod geometry;
mod placement;

pub use config::{Config, ConfigError, Gaming, Layout, MonitorName, Rule, Zone};
pub use geometry::Rect;
pub use placement::{Monitor, Window};

/// The application's name, as it appears in paths and in the tray.
pub const NAME: &str = "views";
