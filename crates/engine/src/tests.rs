use std::cell::RefCell;

use views_platform::{PlaceError, Rect, Window};

use super::*;

const CONFIG: &str = r#"
    version = 1

    [gaming]
    enabled = true
    games = ["game.exe"]

    [[layout]]
    name = "halves"
    columns = 2
    rows = 1
    zones = [
        { name = "left",  column = 0, row = 0, width = 1, height = 1 },
        { name = "right", column = 1, row = 0, width = 1, height = 1 },
    ]

    [[rule]]
    exe = "notes.exe"
    title_contains = "Notes"
    monitor = "desk"
    layout = "halves"
    zone = "left"

    [[rule]]
    exe = "chat.exe"
    monitor = "desk"
    layout = "halves"
    zone = "right"
    reapply_after_ms = 500
"#;

const LEFT: Rect = Rect { left: 0, top: 0, right: 50, bottom: 100 };
const RIGHT: Rect = Rect { left: 50, top: 0, right: 100, bottom: 100 };

const NOTES: WindowId = WindowId(1);
const CHAT: WindowId = WindowId(2);
const GAME: WindowId = WindowId(3);

/// A desktop with one monitor, `desk`, and whatever windows a test opens on it.
#[derive(Default)]
struct World {
    config: RefCell<String>,
    monitors: RefCell<Vec<Monitor>>,
    windows: RefCell<Vec<(WindowId, Window)>>,
    placed: RefCell<Vec<(WindowId, Rect)>>,
}

impl World {
    fn new() -> Self {
        let world = Self::default();
        world.config.replace(CONFIG.into());
        world.attach("desk");
        world
    }

    fn attach(&self, id: &str) {
        let area = Rect { left: 0, top: 0, right: 100, bottom: 100 };
        let monitor = Monitor { id: id.into(), name: String::new(), bounds: area, work_area: area };
        self.monitors.borrow_mut().push(monitor);
    }

    fn open(&self, id: WindowId, exe: &str, title: &str) {
        let window = Window {
            exe: exe.into(),
            path: String::new(),
            class: String::new(),
            title: title.into(),
        };
        self.windows.borrow_mut().push((id, window));
    }

    fn retitle(&self, id: WindowId, title: &str) {
        let mut windows = self.windows.borrow_mut();
        windows.iter_mut().find(|(window, _)| *window == id).unwrap().1.title = title.into();
    }

    /// The placements made since the last call.
    fn placed(&self) -> Vec<(WindowId, Rect)> {
        self.placed.take()
    }

    /// An Engine that has loaded this world's config and swept its windows.
    fn engine(&self) -> (Engine<&Self, impl Fn() -> io::Result<String>>, Tray) {
        let mut engine = Engine::new(self, || Ok(self.config.borrow().clone()));
        let mut tray = Tray::default();
        engine.reload(&mut tray);
        (engine, tray)
    }
}

impl Desktop for &World {
    fn monitors(&self) -> Vec<Monitor> {
        self.monitors.borrow().clone()
    }

    fn windows(&self) -> Vec<WindowId> {
        self.windows.borrow().iter().map(|(id, _)| *id).collect()
    }

    fn window(&self, id: WindowId) -> Option<Window> {
        let windows = self.windows.borrow();
        windows.iter().find(|(window, _)| *window == id).map(|(_, window)| window.clone())
    }

    fn is_tileable(&self, _: WindowId) -> bool {
        true
    }

    fn place(&self, id: WindowId, target: Rect) -> Result<(), PlaceError> {
        self.placed.borrow_mut().push((id, target));
        Ok(())
    }
}

/// A host that records what the Engine asked of it.
#[derive(Default)]
struct Tray {
    interest: Interest,
    status: Status,
    deferred: Vec<(WindowId, Duration)>,
    watching_exit: bool,
    fullscreen_game: bool,
    trims: usize,
    settings_opened: bool,
}

impl Tray {
    /// The delays requested since the last call.
    fn deferred(&mut self) -> Vec<(WindowId, Duration)> {
        mem::take(&mut self.deferred)
    }
}

impl Host for Tray {
    fn run(&mut self, _: impl FnMut(&mut Self, Event) -> ControlFlow<()>) {}

    fn watch(&mut self, interest: Interest) {
        self.interest = interest;
    }

    fn defer(&mut self, window: WindowId, delay: Duration) {
        self.deferred.push((window, delay));
    }

    fn watch_exit(&mut self, _: WindowId) -> bool {
        self.watching_exit = true;
        true
    }

    fn fullscreen_game(&self) -> bool {
        self.fullscreen_game
    }

    fn trim_memory(&mut self) {
        self.trims += 1;
    }

    fn show(&mut self, status: &Status) {
        self.status = status.clone();
    }

    fn open_settings(&mut self) {
        self.settings_opened = true;
    }
}

const WATCHING: Interest = Interest { windows: true, foreground: true };
const IDLE: Interest = Interest { windows: false, foreground: false };

#[test]
fn starting_sweeps_the_windows_that_are_already_open() {
    let world = World::new();
    world.open(NOTES, "notes.exe", "Notes");
    world.open(WindowId(9), "other.exe", "Other");
    world.open(CHAT, "Chat.exe", "Chat");

    let (_, mut tray) = world.engine();

    assert_eq!(world.placed(), [(NOTES, LEFT), (CHAT, RIGHT)]);
    assert_eq!(tray.deferred(), [], "a Sweep places once and schedules nothing");
    assert_eq!(tray.interest, WATCHING);
    assert_eq!(tray.status.summary, "views");
}

#[test]
fn a_new_window_is_placed_and_an_unruled_one_ignored() {
    let world = World::new();
    let (mut engine, mut tray) = world.engine();

    world.open(NOTES, "notes.exe", "Notes");
    world.open(WindowId(9), "other.exe", "Other");
    let _ = engine.handle(&mut tray, Event::WindowCreated(NOTES));
    let _ = engine.handle(&mut tray, Event::WindowCreated(WindowId(9)));

    assert_eq!(world.placed(), [(NOTES, LEFT)]);
    assert_eq!(tray.deferred(), []);
}

#[test]
fn a_window_titled_late_is_placed_on_a_retry() {
    let world = World::new();
    let (mut engine, mut tray) = world.engine();

    world.open(NOTES, "notes.exe", "");
    let _ = engine.handle(&mut tray, Event::WindowCreated(NOTES));
    assert_eq!(world.placed(), []);
    assert_eq!(tray.deferred(), [(NOTES, RETRY_DELAY)]);

    world.retitle(NOTES, "Notes");
    let _ = engine.handle(&mut tray, Event::Deferred(NOTES));
    assert_eq!(world.placed(), [(NOTES, LEFT)]);
    assert_eq!(tray.deferred(), []);
}

#[test]
fn retries_wait_twice_as_long_each_time_and_then_stop() {
    let world = World::new();
    let (mut engine, mut tray) = world.engine();

    world.open(NOTES, "notes.exe", "");
    let _ = engine.handle(&mut tray, Event::WindowCreated(NOTES));
    let mut delays = Vec::new();
    while let [(_, delay)] = tray.deferred()[..] {
        delays.push(delay.as_millis());
        let _ = engine.handle(&mut tray, Event::Deferred(NOTES));
    }
    assert_eq!(delays, [200, 400, 800, 1600]);

    world.retitle(NOTES, "Notes");
    let _ = engine.handle(&mut tray, Event::Deferred(NOTES));
    assert_eq!(world.placed(), [], "a delay the Engine did not ask for is ignored");
}

#[test]
fn a_rule_with_a_reapply_delay_places_twice() {
    let world = World::new();
    let (mut engine, mut tray) = world.engine();

    world.open(CHAT, "chat.exe", "Chat");
    let _ = engine.handle(&mut tray, Event::WindowCreated(CHAT));
    assert_eq!(tray.deferred(), [(CHAT, Duration::from_millis(500))]);

    let _ = engine.handle(&mut tray, Event::Deferred(CHAT));
    assert_eq!(world.placed(), [(CHAT, RIGHT), (CHAT, RIGHT)]);
    assert_eq!(tray.deferred(), [], "the reapply does not reschedule itself");
}

#[test]
fn pausing_stops_rules_until_resumed() {
    let world = World::new();
    let (mut engine, mut tray) = world.engine();

    let _ = engine.handle(&mut tray, Event::Command(Command::TogglePause));
    world.open(NOTES, "notes.exe", "Notes");
    let _ = engine.handle(&mut tray, Event::WindowCreated(NOTES));
    assert_eq!(world.placed(), []);
    assert_eq!(tray.interest, IDLE);
    assert!(tray.status.paused);
    assert_eq!(tray.status.summary, "views\npaused");

    let _ = engine.handle(&mut tray, Event::Command(Command::TogglePause));
    assert_eq!(world.placed(), [], "resuming does not Sweep");
    assert_eq!(tray.interest, WATCHING);

    let _ = engine.handle(&mut tray, Event::WindowCreated(NOTES));
    assert_eq!(world.placed(), [(NOTES, LEFT)]);
}

#[test]
fn a_reload_applies_a_good_config_and_keeps_the_last_one_over_a_bad_one() {
    let world = World::new();
    world.open(NOTES, "notes.exe", "Notes");
    let (mut engine, mut tray) = world.engine();
    world.placed();

    world.config.replace("version = 1\n[[rule]]".into());
    let _ = engine.handle(&mut tray, Event::ReloadRequested);
    assert_eq!(world.placed(), [(NOTES, LEFT)], "the last good config still applies");
    assert!(tray.status.summary.starts_with("views\nconfig not loaded: line 2: "));

    world.config.replace(CONFIG.replace("zone = \"left\"", "zone = \"right\""));
    let _ = engine.handle(&mut tray, Event::ReloadRequested);
    assert_eq!(world.placed(), [(NOTES, RIGHT)]);
    assert_eq!(tray.status.summary, "views");
}

#[test]
fn a_missing_config_file_is_an_empty_config() {
    let world = World::new();
    let mut engine = Engine::new(&world, || Err(io::ErrorKind::NotFound.into()));
    let mut tray = Tray::default();
    engine.reload(&mut tray);
    assert_eq!(tray.status.summary, "views");
}

#[test]
fn rules_for_a_detached_monitor_are_reported_until_it_returns() {
    let world = World::new();
    world.monitors.take();
    let (mut engine, mut tray) = world.engine();
    assert_eq!(tray.status.summary, "views\n2 rules skipped: their monitors are not connected");

    world.attach("desk");
    let _ = engine.handle(&mut tray, Event::DisplaysChanged);
    assert_eq!(tray.status.summary, "views");
}

#[test]
fn gaming_mode_lasts_from_a_game_taking_the_foreground_until_it_exits() {
    let world = World::new();
    world.open(NOTES, "notes.exe", "Notes");
    world.open(GAME, "Game.exe", "Game");
    let (mut engine, mut tray) = world.engine();
    world.placed();

    let _ = engine.handle(&mut tray, Event::ForegroundChanged(NOTES));
    assert_eq!(tray.interest, WATCHING, "an ordinary app changes nothing");

    let _ = engine.handle(&mut tray, Event::ForegroundChanged(GAME));
    assert!(tray.watching_exit);
    assert_eq!(tray.interest, IDLE, "nothing wakes the Engine while the game runs");
    assert_eq!(tray.trims, 1);
    assert_eq!(tray.status.summary, "views\ngaming mode");

    let _ = engine.handle(&mut tray, Event::WindowCreated(NOTES));
    assert_eq!(world.placed(), []);

    let _ = engine.handle(&mut tray, Event::GameExited);
    assert_eq!(world.placed(), [(NOTES, LEFT)], "leaving Gaming mode Sweeps");
    assert_eq!(tray.interest, WATCHING);
    assert_eq!(tray.trims, 1);
}

#[test]
fn a_fullscreen_app_counts_as_a_game_only_when_detection_is_on() {
    let world = World::new();
    world.open(NOTES, "notes.exe", "Notes");
    let (mut engine, mut tray) = world.engine();
    tray.fullscreen_game = true;

    let _ = engine.handle(&mut tray, Event::ForegroundChanged(NOTES));
    assert_eq!(tray.interest, WATCHING);

    world
        .config
        .replace(CONFIG.replace("enabled = true", "enabled = true\ndetect_fullscreen = true"));
    let _ = engine.handle(&mut tray, Event::ReloadRequested);
    let _ = engine.handle(&mut tray, Event::ForegroundChanged(NOTES));
    assert_eq!(tray.interest, IDLE);
}

#[test]
fn the_tray_override_outranks_detection_in_both_directions() {
    let world = World::new();
    world.open(NOTES, "notes.exe", "Notes");
    world.open(GAME, "game.exe", "Game");
    let (mut engine, mut tray) = world.engine();
    world.placed();

    let _ = engine.handle(&mut tray, Event::Command(Command::Gaming(Override::On)));
    assert_eq!(tray.interest, IDLE, "on with no game running");
    assert_eq!(tray.trims, 1);

    let _ = engine.handle(&mut tray, Event::Command(Command::Gaming(Override::Auto)));
    assert_eq!(world.placed(), [(NOTES, LEFT)]);
    let _ = engine.handle(&mut tray, Event::ForegroundChanged(GAME));
    assert_eq!(tray.interest, IDLE);

    let _ = engine.handle(&mut tray, Event::Command(Command::Gaming(Override::Off)));
    let watching_windows = Interest { windows: true, foreground: false };
    assert_eq!(tray.interest, watching_windows, "off while the game still runs");
    assert_eq!(world.placed(), [(NOTES, LEFT)]);

    let _ = engine.handle(&mut tray, Event::Command(Command::Gaming(Override::Auto)));
    assert_eq!(tray.interest, IDLE, "the game never stopped running");
}

#[test]
fn the_tray_opens_settings_and_quits() {
    let world = World::new();
    let (mut engine, mut tray) = world.engine();

    assert!(engine.handle(&mut tray, Event::Command(Command::OpenSettings)).is_continue());
    assert!(tray.settings_opened);
    assert!(engine.handle(&mut tray, Event::Command(Command::Quit)).is_break());
}
