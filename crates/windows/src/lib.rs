//! The Windows backend: Win32 behind the platform's two traits.

#![cfg(windows)]

use std::env;
use std::path::PathBuf;

use views_core::NAME;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{FindWindowW, PostMessageW, RegisterWindowMessageW};
use windows::core::PCWSTR;

mod desktop;
mod host;

pub use desktop::Desktop;
pub use host::Host;

/// Where the config file lives.
pub fn config_path() -> PathBuf {
    let roaming = env::var_os("APPDATA").unwrap_or_default();
    PathBuf::from(roaming).join(NAME).join("config.toml")
}

/// Asks the running Engine to reload its config. Returns whether one was there to ask.
pub fn request_reload() -> bool {
    let Ok(engine) = (unsafe { FindWindowW(host::CLASS, PCWSTR::null()) }) else {
        return false;
    };
    let reload = unsafe { RegisterWindowMessageW(host::RELOAD) };
    let _ = unsafe { PostMessageW(Some(engine), reload, WPARAM(0), LPARAM(0)) };
    true
}
