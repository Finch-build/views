//! The event loop and everything the Engine needs while it runs.

use std::fs::File;
use std::ops::ControlFlow;
use std::process::Command;
use std::time::Duration;

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSApplication, NSApplicationActivationPolicy,
    NSApplicationDidChangeScreenParametersNotification,
};
use objc2_foundation::{NSDistributedNotificationCenter, NSNotificationCenter, NSString};
use views_platform::{Event, Interest, Status, WindowId};

use crate::events::{after, deliver, observe, serve};
use crate::menu::Menu;
use crate::{RELOAD, ax, lock_file, watcher};

/// The distributed notification that the system posts when any app's Accessibility
/// access changes.
const TRUST_CHANGED: &str = "com.apple.accessibility.api";

/// How long after that notification the new answer can be relied on.
const TRUST_SETTLE: Duration = Duration::from_secs(1);

pub struct Host {
    app: Retained<NSApplication>,
    menu: Menu,
    /// Holds the single-instance lock until the Engine exits.
    _lock: File,
}

impl Host {
    /// The host for this session's Engine, or `None` if an Engine is already running or
    /// this is not the main thread.
    pub fn new() -> Option<Self> {
        let mtm = MainThreadMarker::new()?;
        let lock = lock_file().ok()?;
        lock.try_lock().ok()?;

        // Nothing is watched until this is granted, and granting it triggers a reload.
        ax::request_trust();

        let app = NSApplication::sharedApplication(mtm);
        app.setActivationPolicy(NSApplicationActivationPolicy::Accessory);
        watcher::follow_apps();

        // SAFETY: reading a constant the framework exports.
        let screens = unsafe { NSApplicationDidChangeScreenParametersNotification };
        observe(&NSNotificationCenter::defaultCenter(), screens, |_| {
            deliver(Event::DisplaysChanged);
        });

        let distributed = NSDistributedNotificationCenter::defaultCenter();
        observe(&distributed, &NSString::from_str(RELOAD), |_| deliver(Event::ReloadRequested));
        observe(&distributed, &NSString::from_str(TRUST_CHANGED), |_| {
            after(TRUST_SETTLE, || {
                if ax::is_trusted() {
                    deliver(Event::ReloadRequested);
                }
            });
        });

        Some(Self { app, menu: Menu::new(mtm), _lock: lock })
    }
}

impl views_platform::Host for Host {
    fn run(&mut self, mut handler: impl FnMut(&mut Self, Event) -> ControlFlow<()>) {
        let app = self.app.clone();
        let sink = |event| {
            if handler(self, event).is_break() {
                app.stop(None);
            }
        };
        serve(sink, || app.run());
    }

    fn watch(&mut self, Interest { windows, .. }: Interest) {
        watcher::watch(windows);
    }

    fn defer(&mut self, window: WindowId, delay: Duration) {
        after(delay, move || deliver(Event::Deferred(window)));
    }

    fn watch_exit(&mut self, _: WindowId) -> bool {
        false
    }

    fn fullscreen_game(&self) -> bool {
        false
    }

    fn trim_memory(&mut self) {}

    fn show(&mut self, status: &Status) {
        self.menu.show(status);
    }

    fn open_settings(&mut self) {
        if let Ok(engine) = std::env::current_exe() {
            let _ = Command::new(engine.with_file_name("views-settings")).spawn();
        }
    }
}
