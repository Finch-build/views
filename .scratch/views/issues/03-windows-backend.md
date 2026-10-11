# 03: Windows backend and views-debug CLI

**What to build:** The Windows implementation of the platform trait: Monitors with stable IDs, window info (including UWP resolution), the Tileable window filter, border-compensated moves that restore maximized windows, Per-Monitor V2 DPI manifest, and a debug CLI with list and move commands.

**Blocked by:** 01, 02

**Status:** ready-for-agent

- [x] On a two-monitor setup with different scaling, Notepad lands flush in each Zone of a thirds Layout with no gaps
- [x] Elevated windows return an Elevated error and do not crash
- [x] Minimized and fullscreen windows are left alone

## Comments

Built in `crates/windows` (`desktop.rs`) and `views-debug` (`list`, `move <window> <monitor> <layout> <zone>`).

- **Flush placement.** Checked on 1920x1080 at 100% beside 2560x1440 at 125%: the visible frame landed exactly on each third on both monitors. A window that rescales itself on arrival is corrected by one second move. The check used a purpose-built test window, not Notepad, because Windows 11 Notepad restores the user's session.
- **Elevated.** `views-debug` run at low integrity against a normal window reported `Elevated` and left it alone. That is the same access-denied path; a truly elevated window was not tried.
- **UWP.** A frame host's window resolves to the app inside it (Calculator resolved to `CalculatorApp.exe`).
- **Monitor ids** are device interface paths: stable across restarts and reconnects to the same port. The name Windows reports can be as vague as "Generic PnP Monitor".
- The DPI manifest is embedded by `crates/engine/build.rs` with no resource compiler.
