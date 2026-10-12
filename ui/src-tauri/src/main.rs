//! The Settings UI executable: one window, and the commands its page calls.
//!
//! A command that fails returns a message fit to show the user as it is.

#![windows_subsystem = "windows"]

use std::collections::HashSet;
use std::env::{self, consts::EXE_SUFFIX};
use std::fs::{self, File};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Serialize;
use tauri::{Manager, Webview, WebviewWindow, WindowEvent};
use tauri_plugin_dialog::DialogExt;
use views_core::{Config, Monitor};
use views_platform::Desktop;

#[cfg(windows)]
mod autostart;
mod icons;
mod installed;

#[cfg(target_os = "macos")]
use views_macos as os;
#[cfg(windows)]
use views_windows as os;

fn main() {
    start_engine();
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _, _| {
            // A second launch brings the window of the first to the front.
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_dialog::init())
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
        .invoke_handler(tauri::generate_handler![
            config_path,
            version,
            load_config,
            save_config,
            features,
            #[cfg(windows)]
            autostart::autostart,
            #[cfg(windows)]
            autostart::set_autostart,
            monitors,
            running_apps,
            installed::installed_apps,
            browse
        ])
        .run(tauri::generate_context!())
        .expect("the Settings UI could not start");
}

/// Starts the Engine, which sits beside this executable. If one is already running,
/// the new one exits at once.
fn start_engine() {
    if let Ok(settings) = env::current_exe() {
        let _ = Command::new(settings.with_file_name(format!("views{EXE_SUFFIX}"))).spawn();
    }
}

#[tauri::command]
fn config_path() -> PathBuf {
    os::config_path()
}

#[tauri::command]
fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// What this platform has, so that the page shows only what works.
#[derive(Serialize)]
struct Features {
    gaming: bool,
    autostart: bool,
}

#[tauri::command]
fn features() -> Features {
    Features { gaming: cfg!(windows), autostart: cfg!(windows) }
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

/// The attached Monitors.
#[tauri::command]
fn monitors() -> Vec<Monitor> {
    os::Desktop.monitors()
}

/// An app with a Tileable window, as the picker lists it.
#[derive(Serialize)]
struct App {
    exe: String,
    class: String,
    title: String,
    icon: Option<icons::Icon>,
}

/// The apps that have a Tileable window now, one each, in the order of their file names.
#[tauri::command]
async fn running_apps() -> Vec<App> {
    let desktop = os::Desktop;
    let mut seen = HashSet::new();
    let mut apps: Vec<App> = desktop
        .windows()
        .into_iter()
        .filter(|&window| desktop.is_tileable(window))
        .filter_map(|window| desktop.window(window))
        .filter(|window| seen.insert(window.exe.to_lowercase()))
        .map(|window| App {
            icon: icons::of(&window.path),
            exe: window.exe,
            class: window.class,
            title: window.title,
        })
        .collect();
    apps.sort_by_key(|app| app.exe.to_lowercase());
    apps
}

/// The file name of the executable the user chooses, or nothing if they cancel.
#[tauri::command]
async fn browse(window: WebviewWindow) -> Option<String> {
    let file = window.dialog().file().set_parent(&window).blocking_pick_file()?;
    Some(file.into_path().ok()?.file_name()?.to_string_lossy().into_owned())
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
