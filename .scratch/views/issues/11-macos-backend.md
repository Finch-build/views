# 11: macOS backend (Accessibility and menu bar)

**What to build:** The macOS implementation of the platform trait: Accessibility permission prompt, app launch and window-created observers, set size then position then size, Monitors from NSScreen with display UUID, top-left coordinate conversion, and a menu-bar item mirroring the tray menu. Gaming mode is hidden on macOS.

**Blocked by:** 02, 05

**Status:** ready-for-human

- [ ] Same Rule behaviour as ticket 4 for standard apps
- [ ] Nothing happens until Accessibility access is granted
- [x] Spaces, native fullscreen and sandboxing limits are documented

## Comments

Written in `crates/macos`, with the limits in `docs/macos.md`. **It has never run**: it was written on Windows and is only type-checked and linted for `aarch64-apple-darwin`. The two open boxes need a Mac. Test in this order:

1. Granting Accessibility access is noticed through an undocumented system notification. If it is not, restart the Engine after granting.
2. Windows are identified with the private `_AXUIElementGetWindow`. If it gives nothing for a window at creation, that window is only placed by a later Sweep.
3. An observer registered the moment an app launches may fail, and is not retried.
4. Quit calls `NSApp.stop`; if the loop does not return, use `terminate:`.

On macOS a Rule's `exe` is the executable's file name (`Safari`). The menu-bar item has Open settings, Pause and Quit; there is no Gaming mode.
