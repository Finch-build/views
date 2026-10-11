//! The Settings UI executable: one window, and the commands its page calls.
//!
//! A command that fails returns a message fit to show the user as it is.

#![windows_subsystem = "windows"]

use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use tauri::{Manager, Webview, WindowEvent};
use views_core::Config;

#[cfg(target_os = "macos")]
use views_macos as os;
#[cfg(windows)]
use views_windows as os;

fn main() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            // A second launch brings the window of the first to the front.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .on_window_event(|window, event| {
            // A minimised window's page is not told it is hidden, and would go on animating.
            if let WindowEvent::Resized(_) = event
                && let Some(window) = window.get_webview_window(window.label())
            {
                let page: &Webview = window.as_ref();
                let _ =
                    if window.is_minimized().unwrap_or(false) { page.hide() } else { page.show() };
            }
        })
        .invoke_handler(tauri::generate_handler![config_path, version, load_config, save_config])
        .run(tauri::generate_context!())
        .expect("the Settings UI could not start");
}

#[tauri::command]
fn config_path() -> PathBuf {
    os::config_path()
}

#[tauri::command]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The config on disk, or an empty one if there is no file yet.
#[tauri::command]
fn load_config() -> Result<Config, String> {
    let path = os::config_path();
    match fs::read_to_string(&path) {
        Ok(text) => text.parse().map_err(|error: views_core::ConfigError| at(&path, error)),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
        Err(error) => Err(at(&path, error)),
    }
}

/// Writes `config` if it is valid, then asks the Engine to reload. Returns whether an
/// Engine was there to ask.
#[tauri::command]
fn save_config(config: Config) -> Result<bool, String> {
    config.validate().map_err(|error| error.to_string())?;
    let path = os::config_path();
    replace(&path, &config.to_string()).map_err(|error| at(&path, error))?;
    Ok(os::request_reload())
}

/// Puts `text` at `path` in one step, so no reader ever sees half a file.
fn replace(path: &Path, text: &str) -> io::Result<()> {
    if let Some(directory) = path.parent() {
        fs::create_dir_all(directory)?;
    }
    let draft = path.with_extension("tmp");
    let mut file = File::create(&draft)?;
    file.write_all(text.as_bytes())?;
    file.sync_all()?;
    drop(file);
    fs::rename(&draft, path)
}

fn at(path: &Path, error: impl std::fmt::Display) -> String {
    format!("{}: {error}", path.display())
}
