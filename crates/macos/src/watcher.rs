//! Window creation, observed through one Accessibility observer per running app.

use std::cell::RefCell;
use std::collections::HashMap;
use std::ffi::c_void;
use std::ptr::{self, NonNull};

use objc2::rc::Retained;
use objc2_app_kit::{
    NSRunningApplication, NSWorkspace, NSWorkspaceApplicationKey,
    NSWorkspaceDidLaunchApplicationNotification, NSWorkspaceDidTerminateApplicationNotification,
};
use objc2_application_services::{AXError, AXObserver, AXUIElement};
use objc2_core_foundation::{CFRetained, CFRunLoop, CFString, kCFRunLoopCommonModes};
use objc2_foundation::NSNotification;
use views_platform::Event;

use crate::events::{deliver, observe};
use crate::{app, ax};

thread_local! {
    /// The observers, keyed by process id, while windows are being watched.
    static OBSERVERS: RefCell<Option<HashMap<i32, Observer>>> = const { RefCell::new(None) };
}

struct Observer(CFRetained<AXObserver>);

impl Drop for Observer {
    fn drop(&mut self) {
        if let Some(run_loop) = CFRunLoop::main() {
            // SAFETY: the observer owns its source.
            let source = unsafe { self.0.run_loop_source() };
            // SAFETY: reading a constant the framework exports.
            run_loop.remove_source(Some(&source), unsafe { kCFRunLoopCommonModes });
        }
    }
}

/// Starts or stops delivering [`Event::WindowCreated`]. Until the user has granted
/// Accessibility access it does nothing, and only a later call picks up the grant.
pub fn watch(windows: bool) {
    let windows = windows && ax::is_trusted();
    OBSERVERS.with_borrow_mut(|observers| match (windows, observers.is_some()) {
        (true, false) => {
            let all = app::pids().into_iter().filter_map(|pid| Some((pid, observer(pid)?)));
            *observers = Some(all.collect());
        }
        (false, true) => *observers = None,
        _ => {}
    });
}

/// Follows apps as they start and quit, so that each has an observer while windows are
/// watched.
pub fn follow_apps() {
    let center = NSWorkspace::sharedWorkspace().notificationCenter();
    // SAFETY: reading constants the framework exports.
    let (launched, terminated) = unsafe {
        (
            NSWorkspaceDidLaunchApplicationNotification,
            NSWorkspaceDidTerminateApplicationNotification,
        )
    };
    observe(&center, launched, |notification| {
        if let Some(app) = running_app(notification).filter(|app| app::is_regular(app)) {
            launch(app.processIdentifier());
        }
    });
    observe(&center, terminated, |notification| {
        if let Some(app) = running_app(notification) {
            let pid = app.processIdentifier();
            OBSERVERS.with_borrow_mut(|observers| observers.as_mut().and_then(|o| o.remove(&pid)));
        }
    });
}

fn running_app(notification: &NSNotification) -> Option<Retained<NSRunningApplication>> {
    let info = notification.userInfo()?;
    // SAFETY: reading a constant the framework exports.
    let app = info.objectForKey(unsafe { NSWorkspaceApplicationKey })?;
    app.downcast().ok()
}

/// Observes an app that has just started and delivers the windows it already has, which
/// may have opened before the observer was there to see them.
fn launch(pid: i32) {
    let watching = OBSERVERS.with_borrow_mut(|observers| {
        let Some(observers) = observers else {
            return false;
        };
        observers.extend(observer(pid).map(|observer| (pid, observer)));
        true
    });
    if watching {
        for window in ax::windows(pid).iter().filter_map(|window| ax::id(window)) {
            deliver(Event::WindowCreated(window));
        }
    }
}

fn observer(pid: i32) -> Option<Observer> {
    let mut created = ptr::null_mut();
    // SAFETY: `created` is a valid out pointer, and `window_created` has the callback's
    // signature.
    let error =
        unsafe { AXObserver::create(pid, Some(window_created), NonNull::from(&mut created)) };
    if error != AXError::Success {
        return None;
    }
    // SAFETY: a successful create returns an owned reference.
    let observer = Observer(unsafe { CFRetained::from_raw(NonNull::new(created)?) });
    // SAFETY: an unknown pid yields an element that the next call rejects.
    let app = unsafe { AXUIElement::new_application(pid) };
    // SAFETY: no refcon is passed.
    let error = unsafe {
        observer.0.add_notification(&app, &CFString::from_str("AXWindowCreated"), ptr::null_mut())
    };
    if error != AXError::Success {
        return None;
    }
    // SAFETY: the observer owns its source.
    let source = unsafe { observer.0.run_loop_source() };
    // SAFETY: reading a constant the framework exports.
    CFRunLoop::main()?.add_source(Some(&source), unsafe { kCFRunLoopCommonModes });
    Some(observer)
}

unsafe extern "C-unwind" fn window_created(
    _: NonNull<AXObserver>,
    window: NonNull<AXUIElement>,
    _: NonNull<CFString>,
    _: *mut c_void,
) {
    // SAFETY: the element is live for the length of the callback.
    if let Some(window) = ax::id(unsafe { window.as_ref() }) {
        deliver(Event::WindowCreated(window));
    }
}
