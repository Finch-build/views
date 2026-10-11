//! The way from the callbacks that AppKit and Accessibility invoke to the handler that
//! `Host::run` was given.

use std::cell::Cell;
use std::mem;
use std::ptr::NonNull;
use std::time::Duration;

use block2::RcBlock;
use objc2_foundation::{NSNotification, NSNotificationCenter, NSNotificationName, NSTimer};
use views_platform::Event;

type Sink = NonNull<dyn FnMut(Event)>;

thread_local! {
    static SINK: Cell<Option<Sink>> = const { Cell::new(None) };
}

/// Runs `body` with `sink` as the receiver of [`deliver`].
pub fn serve(mut sink: impl FnMut(Event), body: impl FnOnce()) {
    let sink: &mut dyn FnMut(Event) = &mut sink;
    // SAFETY: only the lifetime is erased, and the pointer is withdrawn before `sink`
    // goes out of scope.
    SINK.set(Some(unsafe { mem::transmute::<&mut dyn FnMut(Event), Sink>(sink) }));
    body();
    SINK.set(None);
}

/// Hands `event` to the handler, unless none is running.
pub fn deliver(event: Event) {
    // Taken for the length of the call, so that an event raised by the handler itself
    // is dropped instead of re-entering it.
    if let Some(mut sink) = SINK.take() {
        // SAFETY: `serve` keeps the sink alive while it is stored, and only this thread
        // can reach it.
        unsafe { sink.as_mut()(event) };
        SINK.set(Some(sink));
    }
}

/// Calls `handler` each time `center` posts `name`.
pub fn observe(
    center: &NSNotificationCenter,
    name: &NSNotificationName,
    handler: impl Fn(&NSNotification) + 'static,
) {
    let block = RcBlock::new(move |notification: NonNull<NSNotification>| {
        // SAFETY: the center passes a live notification.
        handler(unsafe { notification.as_ref() });
    });
    // SAFETY: with no queue the block runs on the thread that posts, and every name
    // observed here is posted on the main thread, where this is called.
    let token = unsafe {
        center.addObserverForName_object_queue_usingBlock(Some(name), None, None, &block)
    };
    // The center keeps the token, and nothing here ever unregisters.
    drop(token);
}

/// Calls `action` once, on this thread's run loop, after `delay`.
pub fn after(delay: Duration, action: impl Fn() + 'static) {
    let block = RcBlock::new(move |_: NonNull<NSTimer>| action());
    // SAFETY: the block runs on the run loop of the thread that called this.
    let timer = unsafe {
        NSTimer::scheduledTimerWithTimeInterval_repeats_block(delay.as_secs_f64(), false, &block)
    };
    // The run loop keeps the timer until it fires.
    drop(timer);
}
