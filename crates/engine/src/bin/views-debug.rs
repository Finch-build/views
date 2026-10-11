//! A console tool that drives the platform's `Desktop` by hand.

use std::fs;
use std::process::ExitCode;

use views_core::{Config, ConfigError};
use views_platform::{Desktop as _, WindowId};

#[cfg(target_os = "macos")]
use views_macos as os;
#[cfg(windows)]
use views_windows as os;

const USAGE: &str = "usage: views-debug list
       views-debug move <window> <monitor> <layout> <zone>";

fn main() -> ExitCode {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let arguments: Vec<&str> = arguments.iter().map(String::as_str).collect();
    let done = match arguments[..] {
        ["list"] => {
            list();
            Ok(())
        }
        ["move", window, monitor, layout, zone] => place(window, monitor, layout, zone),
        _ => Err(USAGE.to_owned()),
    };
    match done {
        Ok(()) => ExitCode::SUCCESS,
        Err(problem) => {
            eprintln!("{problem}");
            ExitCode::FAILURE
        }
    }
}

/// Prints the Monitors and the Tileable windows.
fn list() {
    for monitor in os::Desktop.monitors() {
        let area = monitor.work_area;
        println!("monitor {}", monitor.id);
        println!("  name       {}", monitor.name);
        println!("  work area  {} {} {} {}", area.left, area.top, area.right, area.bottom);
    }
    for id in os::Desktop.windows() {
        if os::Desktop.is_tileable(id)
            && let Some(window) = os::Desktop.window(id)
        {
            println!("window {}", id.0);
            println!("  exe    {}", window.exe);
            println!("  class  {}", window.class);
            println!("  title  {}", window.title);
        }
    }
}

/// Places a window in a Zone of the config's Layout, as a Rule naming them would.
fn place(window: &str, monitor: &str, layout: &str, zone: &str) -> Result<(), String> {
    let window = WindowId(window.parse().map_err(|_| format!("{window} is not a window id"))?);
    let config = fs::read_to_string(os::config_path()).map_err(|error| error.to_string())?;
    let config: Config = config.parse().map_err(|error: ConfigError| error.to_string())?;

    let monitors = os::Desktop.monitors();
    let monitor = monitors.iter().find(|candidate| candidate.id == monitor);
    let monitor = monitor.ok_or("no such monitor")?;
    let layout = config.layouts.iter().find(|candidate| candidate.name == layout);
    let layout = layout.ok_or("no such layout")?;
    let zone = layout.zones.iter().find(|candidate| candidate.name == zone);
    let zone = zone.ok_or("no such zone in that layout")?;

    if !os::Desktop.is_tileable(window) {
        return Err(format!("window {} is not tileable", window.0));
    }
    let target = layout.rect(zone, monitor.work_area);
    os::Desktop.place(window, target).map_err(|error| format!("not placed: {error:?}"))?;
    println!("placed at {} {} {} {}", target.left, target.top, target.right, target.bottom);
    Ok(())
}
