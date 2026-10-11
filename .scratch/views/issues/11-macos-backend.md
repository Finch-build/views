# 11: macOS backend (Accessibility and menu bar)

**What to build:** The macOS implementation of the platform trait: Accessibility permission prompt, app launch and window-created observers, set size then position then size, Monitors from NSScreen with display UUID, top-left coordinate conversion, and a menu-bar item mirroring the tray menu. Gaming mode is hidden on macOS.

**Blocked by:** 02, 05

**Status:** ready-for-agent

- [ ] Same Rule behaviour as ticket 4 for standard apps
- [ ] Nothing happens until Accessibility access is granted
- [ ] Spaces, native fullscreen and sandboxing limits are documented
