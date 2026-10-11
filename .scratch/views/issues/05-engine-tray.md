# 05: Engine: tray, pause, reload, reapply

**What to build:** The tray icon (head-and-beak crop of the Finch bird, 16/24/32 px ICO) with menu Open settings, Pause, Gaming mode, Quit. Reload via the registered message followed by a Sweep. reapply_after_ms through a one-shot timer. Display-change handling. Tooltip shows config errors and skipped Rules for missing Monitors. The ipc reload helper.

**Blocked by:** 04

**Status:** ready-for-agent

- [ ] Pause stops rule application and resume works
- [ ] Editing the config then sending reload applies without restart, keeping the last good config on error
- [ ] Reapply timer is killed after firing, with no standing timers
- [ ] Display change refreshes the Monitor list
