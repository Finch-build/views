# 05: Engine: tray, pause, reload, reapply

**What to build:** The tray icon (head-and-beak crop of the Finch bird, 16/24/32 px ICO) with menu Open settings, Pause, Gaming mode, Quit. Reload via the registered message followed by a Sweep. reapply_after_ms through a one-shot timer. Display-change handling. Tooltip shows config errors and skipped Rules for missing Monitors. The ipc reload helper.

**Blocked by:** 04

**Status:** ready-for-agent

- [x] Pause stops rule application and resume works
- [x] Editing the config then sending reload applies without restart, keeping the last good config on error
- [x] Reapply timer is killed after firing, with no standing timers
- [ ] Display change refreshes the Monitor list

## Comments

- **Tray.** Menu: Open settings, Pause, Gaming mode (Auto / On / Off), Quit. Every item was driven on the real icon. The icon is a filled silhouette of the Finch head, paper on an ink tile, at 16, 24 and 32 px (`crates/windows/assets/tray.ico`); the line drawing itself does not survive at 16 px.
- **Reload** is `request_reload()` in each backend rather than an `ipc` crate (ADR 0002). An invalid edit kept the last good config and showed `config not loaded: ...` in the tooltip; a Rule for a missing Monitor showed `1 rule skipped: ...`.
- **Reapply.** `reapply_after_ms = 600` placed twice and left no timer (0 context switches afterwards).
- **Display change is the open box.** The Engine refreshes its Monitors on `WM_DISPLAYCHANGE` and when a work area changes, and the logic is unit-tested, but on the machine it was only exercised by posting the message. Unplug a monitor to close it.
