//! The macOS backend: the Accessibility API for windows, AppKit for the menu bar.

#![cfg(target_os = "macos")]

use std::fs::{self, File, OpenOptions, TryLockError};
use std::io;
use std::path::PathBuf;

use objc2_foundation::{NSDistributedNotificationCenter, NSString};
use views_core::NAME;

mod app;
mod ax;
mod desktop;
mod events;
mod host;
mod menu;
mod watcher;

pub use desktop::Desktop;
pub use host::Host;

/// The distributed notification that asks the Engine to reload its config.
const RELOAD: &str = "views.reload";

/// Where the config file lives.
pub fn config_path() -> PathBuf {
    let home = std::env::home_dir().unwrap_or_default();
    home.join("Library/Application Support").join(NAME).join("config.toml")
}

/// Asks the running Engine to reload its config. Returns whether one was there to ask.
pub fn request_reload() -> bool {
    let running =
        lock_file().is_ok_and(|lock| matches!(lock.try_lock(), Err(TryLockError::WouldBlock)));
    if running {
        // Immediately, because the system holds back notifications for an app that is
        // not in the foreground.
        let center = NSDistributedNotificationCenter::defaultCenter();
        let name = NSString::from_str(RELOAD);
        // SAFETY: the notification carries no object and no user info.
        unsafe {
            center.postNotificationName_object_userInfo_deliverImmediately(&name, None, None, true)
        };
    }
    running
}

/// The file beside the config that the running Engine holds locked.
fn lock_file() -> io::Result<File> {
    let path = config_path().with_file_name("engine.lock");
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir)?;
    }
    OpenOptions::new().create(true).truncate(false).write(true).open(path)
}
