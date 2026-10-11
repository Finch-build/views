//! Starting the Engine at sign-in: a value in the user's Run key that points at it.

use std::path::{Path, PathBuf};

use views_core::NAME;
use windows_registry::{CURRENT_USER, Key};

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// The Engine, which sits beside the Settings executable.
fn engine() -> Result<PathBuf, String> {
    let settings = std::env::current_exe().map_err(|error| error.to_string())?;
    Ok(settings.with_file_name("views.exe"))
}

fn run_key() -> Result<Key, String> {
    CURRENT_USER.create(RUN).map_err(|error| format!("The Run key could not be opened: {error}"))
}

fn command(engine: &Path) -> String {
    format!("\"{}\"", engine.display())
}

/// Whether `run` starts `engine`.
fn is_on(run: &Key, engine: &Path) -> bool {
    run.get_string(NAME).is_ok_and(|value| value.eq_ignore_ascii_case(&command(engine)))
}

/// Makes `run` start `engine` at sign-in, or stop doing so.
fn set(run: &Key, engine: &Path, on: bool) -> Result<(), String> {
    if on && !engine.is_file() {
        return Err(format!("The Engine was not found at {}.", engine.display()));
    }
    let done = if on {
        run.set_string(NAME, command(engine))
    } else if run.get_type(NAME).is_ok() {
        run.remove_value(NAME)
    } else {
        Ok(())
    };
    done.map_err(|error| format!("The Run key could not be changed: {error}"))
}

/// Whether the Engine starts when the user signs in.
#[tauri::command]
pub fn autostart() -> Result<bool, String> {
    Ok(is_on(&run_key()?, &engine()?))
}

/// Turns starting the Engine at sign-in on or off.
#[tauri::command]
pub fn set_autostart(on: bool) -> Result<(), String> {
    set(&run_key()?, &engine()?, on)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A key of its own per test, so that the user's Run key is never touched.
    struct Scratch(String, Key);

    impl Scratch {
        fn new(test: &str) -> Self {
            let path = format!(r"Software\views-settings-test-{test}");
            let key = CURRENT_USER.create(&path).unwrap();
            Self(path, key)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            CURRENT_USER.remove_tree(&self.0).unwrap();
        }
    }

    #[test]
    fn autostart_is_off_until_turned_on_and_off_again_after() {
        let scratch = Scratch::new("toggle");
        let run = &scratch.1;
        let engine = std::env::current_exe().unwrap();
        assert!(!is_on(run, &engine));
        set(run, &engine, true).unwrap();
        assert!(is_on(run, &engine));
        assert_eq!(run.get_string(NAME).unwrap(), format!("\"{}\"", engine.display()));
        set(run, &engine, false).unwrap();
        assert!(!is_on(run, &engine));
        set(run, &engine, false).unwrap();
    }

    #[test]
    fn autostart_is_not_turned_on_for_an_engine_that_is_missing() {
        let scratch = Scratch::new("missing");
        let run = &scratch.1;
        let engine = std::env::temp_dir().join("views-missing.exe");
        assert!(set(run, &engine, true).is_err());
        assert!(!is_on(run, &engine));
    }
}
