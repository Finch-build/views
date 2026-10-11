//! The Engine's logic: which window goes where, and when to stand down.
//!
//! It is driven by [`Event`]s and acts only through the [`Desktop`] and [`Host`]
//! traits, so it is the same on every platform and runs under test without one.

use std::io;
use std::mem;
use std::ops::ControlFlow;
use std::time::Duration;

use views_core::{Config, ConfigError, Monitor, NAME};
use views_platform::{Command, Desktop, Event, Host, Interest, Override, Status, WindowId};

#[cfg(test)]
mod tests;

/// How long a new window gets to become one that a Rule can match. Apps often create
/// a window before they title it or show it.
const RETRY_DELAY: Duration = Duration::from_millis(200);

/// Runs an Engine on `host` until the user quits or the session ends.
///
/// `read_config` returns the text of the config file each time it is called.
pub fn run(
    desktop: impl Desktop,
    host: &mut impl Host,
    read_config: impl Fn() -> io::Result<String>,
) {
    let mut engine = Engine::new(desktop, read_config);
    engine.reload(host);
    host.run(|host, event| engine.handle(host, event));
}

struct Engine<D, R> {
    desktop: D,
    read_config: R,
    config: Config,
    /// Why the config in use is not the one on disk.
    problem: Option<String>,
    monitors: Vec<Monitor>,
    mode: Mode,
    paused: bool,
    gaming: Override,
    /// A game was detected and its process has not exited.
    game_running: bool,
    /// Windows whose pending [`Event::Deferred`] is a Rule's reapply, not a retry.
    reapplying: Vec<WindowId>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Mode {
    Normal,
    Gaming,
    Paused,
}

/// What became of an attempt to place a window.
enum Arranged {
    /// A Rule placed it and wants it placed again after this long, if at all.
    Placed { again_after: Duration },
    /// No Rule can place it as it is now, but one names its executable.
    NotYet,
    /// No Rule names its executable.
    Never,
}

/// Why the Engine is looking at a window.
#[derive(PartialEq, Eq)]
enum Look {
    Created,
    Retry,
    Reapply,
}

impl<D: Desktop, R: Fn() -> io::Result<String>> Engine<D, R> {
    fn new(desktop: D, read_config: R) -> Self {
        Self {
            monitors: desktop.monitors(),
            desktop,
            read_config,
            config: Config::default(),
            problem: None,
            mode: Mode::Normal,
            paused: false,
            gaming: Override::Auto,
            game_running: false,
            reapplying: Vec::new(),
        }
    }

    fn handle(&mut self, host: &mut impl Host, event: Event) -> ControlFlow<()> {
        match event {
            Event::WindowCreated(window) => self.look(host, window, Look::Created),
            Event::Deferred(window) => {
                let pending = self.reapplying.len();
                self.reapplying.retain(|&reapplying| reapplying != window);
                let reapply = self.reapplying.len() < pending;
                self.look(host, window, if reapply { Look::Reapply } else { Look::Retry });
            }
            Event::ForegroundChanged(window) => self.detect_game(host, window),
            Event::GameExited => {
                self.game_running = false;
                self.settle(host, false);
            }
            Event::DisplaysChanged => {
                self.monitors = self.desktop.monitors();
                host.show(&self.status());
            }
            Event::ReloadRequested => self.reload(host),
            Event::Command(Command::OpenSettings) => host.open_settings(),
            Event::Command(Command::TogglePause) => {
                self.paused = !self.paused;
                self.settle(host, false);
            }
            Event::Command(Command::Gaming(gaming)) => {
                self.gaming = gaming;
                self.settle(host, false);
            }
            Event::Command(Command::Quit) => return ControlFlow::Break(()),
        }
        ControlFlow::Continue(())
    }

    /// Rereads the config and runs a Sweep with it. A config that cannot be read or
    /// is invalid leaves the one in use in place.
    fn reload(&mut self, host: &mut impl Host) {
        let loaded = match (self.read_config)() {
            Ok(text) => text.parse().map_err(|error: ConfigError| error.to_string()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(Config::default()),
            Err(error) => Err(error.to_string()),
        };
        match loaded {
            Ok(config) => (self.config, self.problem) = (config, None),
            Err(problem) => self.problem = Some(problem),
        }
        self.settle(host, true);
    }

    /// Enters the mode that the user's choices and any running game call for, and
    /// brings the host in line with it. Sweeps if `sweep` is set or Gaming mode has
    /// just ended, provided Rules apply in the new mode.
    fn settle(&mut self, host: &mut impl Host, sweep: bool) {
        let gaming = match self.gaming {
            Override::On => true,
            Override::Off => false,
            Override::Auto => self.config.gaming.enabled && self.game_running,
        };
        let mode = match (self.paused, gaming) {
            (true, _) => Mode::Paused,
            (false, true) => Mode::Gaming,
            (false, false) => Mode::Normal,
        };
        let was = mem::replace(&mut self.mode, mode);

        host.watch(self.interest());
        match mode {
            Mode::Normal if sweep || was == Mode::Gaming => {
                for window in self.desktop.windows() {
                    self.arrange(window);
                }
            }
            Mode::Gaming if was != Mode::Gaming => host.trim_memory(),
            _ => {}
        }
        host.show(&self.status());
    }

    /// Outside Normal mode nothing is watched. In it, the foreground is watched only
    /// while a game could still be detected.
    fn interest(&self) -> Interest {
        let gaming = &self.config.gaming;
        let detecting = gaming.enabled
            && self.gaming == Override::Auto
            && !self.game_running
            && (gaming.detect_fullscreen || !gaming.games.is_empty());
        let normal = self.mode == Mode::Normal;
        Interest { windows: normal, foreground: normal && detecting }
    }

    fn status(&self) -> Status {
        let mut summary = String::from(NAME);
        match self.mode {
            Mode::Normal => {}
            Mode::Gaming => summary += "\ngaming mode",
            Mode::Paused => summary += "\npaused",
        }
        if let Some(problem) = &self.problem {
            summary += &format!("\nconfig not loaded: {problem}");
        }
        match self.config.skipped(&self.monitors) {
            0 => {}
            1 => summary += "\n1 rule skipped: its monitor is not connected",
            skipped => {
                summary += &format!("\n{skipped} rules skipped: their monitors are not connected")
            }
        }
        Status { paused: self.paused, gaming: self.gaming, summary }
    }

    /// Starts Gaming mode if the window that took the foreground belongs to a game.
    fn detect_game(&mut self, host: &mut impl Host, window: WindowId) {
        if !self.interest().foreground {
            return;
        }
        let gaming = &self.config.gaming;
        let listed = self.desktop.window(window).is_some_and(|window| gaming.lists(&window.exe));
        let game = listed || (gaming.detect_fullscreen && host.fullscreen_game());
        if game && host.watch_exit(window) {
            self.game_running = true;
            self.settle(host, false);
        }
    }

    /// Places `window`, then schedules whichever second look it has earned: a retry
    /// if a new window could not be matched yet, a reapply if its Rule asks for one.
    fn look(&mut self, host: &mut impl Host, window: WindowId, look: Look) {
        if self.mode != Mode::Normal {
            return;
        }
        match self.arrange(window) {
            Arranged::Placed { again_after } if look != Look::Reapply && !again_after.is_zero() => {
                self.reapplying.push(window);
                host.defer(window, again_after);
            }
            Arranged::NotYet if look == Look::Created => host.defer(window, RETRY_DELAY),
            _ => {}
        }
    }

    /// Moves `window` to the Zone its Rule assigns, if it has one and is Tileable.
    fn arrange(&self, window: WindowId) -> Arranged {
        let Some(info) = self.desktop.window(window) else {
            return Arranged::NotYet;
        };
        if !self.config.names(&info.exe) {
            return Arranged::Never;
        }
        if !self.desktop.is_tileable(window) {
            return Arranged::NotYet;
        }
        let Some((rule, target)) = self.config.placement(&info, &self.monitors) else {
            return Arranged::NotYet;
        };
        // A window that is elevated or already gone stays put; neither improves with time.
        let _ = self.desktop.place(window, target);
        Arranged::Placed { again_after: Duration::from_millis(rule.reapply_after_ms.into()) }
    }
}
