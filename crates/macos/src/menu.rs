//! The menu-bar item.

use objc2::rc::Retained;
use objc2::runtime::{AnyObject, Sel};
use objc2::{MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSControlStateValueOff, NSControlStateValueOn, NSImage, NSMenu, NSMenuItem, NSStatusBar,
    NSStatusItem, NSVariableStatusItemLength,
};
use objc2_foundation::{NSObject, NSString, ns_string};
use views_core::NAME;
use views_platform::{Command, Event, Status};

use crate::events::deliver;

define_class!(
    /// Receives the menu's actions.
    #[unsafe(super(NSObject))]
    #[thread_kind = MainThreadOnly]
    struct Target;

    impl Target {
        #[unsafe(method(openSettings:))]
        fn open_settings(&self, _: &AnyObject) {
            deliver(Event::Command(Command::OpenSettings));
        }

        #[unsafe(method(togglePause:))]
        fn toggle_pause(&self, _: &AnyObject) {
            deliver(Event::Command(Command::TogglePause));
        }

        #[unsafe(method(quit:))]
        fn quit(&self, _: &AnyObject) {
            deliver(Event::Command(Command::Quit));
        }
    }
);

impl Target {
    fn new(mtm: MainThreadMarker) -> Retained<Self> {
        // SAFETY: `NSObject`'s `init` takes no arguments.
        unsafe { msg_send![super(mtm.alloc::<Self>().set_ivars(())), init] }
    }
}

pub struct Menu {
    mtm: MainThreadMarker,
    item: Retained<NSStatusItem>,
    pause: Retained<NSMenuItem>,
    /// Menu items hold their target weakly.
    _target: Retained<Target>,
}

impl Menu {
    pub fn new(mtm: MainThreadMarker) -> Self {
        let target = Target::new(mtm);
        let menu = NSMenu::new(mtm);
        let add = |title: &str, action: Sel| {
            // SAFETY: `action` is one of `Target`'s methods.
            let item = unsafe {
                NSMenuItem::initWithTitle_action_keyEquivalent(
                    mtm.alloc(),
                    &NSString::from_str(title),
                    Some(action),
                    ns_string!(""),
                )
            };
            // SAFETY: `Menu` keeps the target alive for as long as the item.
            unsafe { item.setTarget(Some(&target)) };
            menu.addItem(&item);
            item
        };
        add("Open settings", sel!(openSettings:));
        let pause = add("Pause", sel!(togglePause:));
        menu.addItem(&NSMenuItem::separatorItem(mtm));
        add("Quit", sel!(quit:));

        let item = NSStatusBar::systemStatusBar().statusItemWithLength(NSVariableStatusItemLength);
        item.setMenu(Some(&menu));
        if let Some(button) = item.button(mtm) {
            let symbol = NSImage::imageWithSystemSymbolName_accessibilityDescription(
                ns_string!("rectangle.split.2x2"),
                None,
            );
            match symbol {
                Some(symbol) => button.setImage(Some(&symbol)),
                None => button.setTitle(&NSString::from_str(NAME)),
            }
        }
        Self { mtm, item, pause, _target: target }
    }

    pub fn show(&self, status: &Status) {
        if let Some(button) = self.item.button(self.mtm) {
            button.setToolTip(Some(&NSString::from_str(&status.summary)));
        }
        let state = if status.paused { NSControlStateValueOn } else { NSControlStateValueOff };
        self.pause.setState(state);
    }
}
