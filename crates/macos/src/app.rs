//! Running applications.

use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};

/// Whether `app` is one with windows and a Dock icon, as opposed to a background agent.
pub fn is_regular(app: &NSRunningApplication) -> bool {
    app.activationPolicy() == NSApplicationActivationPolicy::Regular
}

/// The process ids of the running regular apps.
pub fn pids() -> Vec<i32> {
    let apps = NSWorkspace::sharedWorkspace().runningApplications();
    apps.iter().filter(|app| is_regular(app)).map(|app| app.processIdentifier()).collect()
}
