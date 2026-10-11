# 03: Windows backend and views-debug CLI

**What to build:** The Windows implementation of the platform trait: Monitors with stable IDs, window info (including UWP resolution), the Tileable window filter, border-compensated moves that restore maximized windows, Per-Monitor V2 DPI manifest, and a debug CLI with list and move commands.

**Blocked by:** 01, 02

**Status:** ready-for-agent

- [ ] On a two-monitor setup with different scaling, Notepad lands flush in each Zone of a thirds Layout with no gaps
- [ ] Elevated windows return an Elevated error and do not crash
- [ ] Minimized and fullscreen windows are left alone
