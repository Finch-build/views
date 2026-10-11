# 04: Engine: window-created to window-moved

**What to build:** A single-instance engine that, when a Tileable window is created, moves it to its Rule's Zone using the shell hook. A new window that is not yet matchable gets one one-shot retry. A Sweep runs on start. The engine opts into EcoQoS and low memory priority.

**Blocked by:** 03

**Status:** ready-for-agent

- [x] Launching a ruled app moves it within 50 ms for well-behaved apps
- [x] Idle: 0% CPU, no wakeups, one thread (below 3 MB private working set, under 1 MB binary)
- [x] A title-based Rule works for apps that set their title late
- [x] Measured numbers reported against the budgets

## Comments

The logic is in `crates/engine` (tested against fakes); the Windows host is `crates/windows/src/host.rs`.

| Budget | Measured |
|---|---|
| Binary under 1 MB | `views.exe` 527,360 bytes |
| One thread | 1 at steady state; Windows adds 3 or 4 idle thread-pool workers for a while after start and after the tray menu, then retires them |
| Private working set under 3 MB | 1,180 K fresh, 1,676 K after the menu and Open settings |
| Idle: no CPU, no wakeups | 0 context switches in 60 s on a quiet desktop |
| Shown to moved under 50 ms | 3.9 to 5.7 ms on one monitor, 8.8 to 12.8 ms across scales (8 launches each) |

Numbers are from the GNU toolchain on the development machine; CI and releases build with MSVC, which has not been measured.

- **Retries.** The plan's single retry was too short for UWP apps, whose frame took about 480 ms to gain its content. A new window that cannot be matched yet is now retried up to four times, waiting 200, 400, 800 and 1600 ms; each is a one-shot timer, and only windows whose exe some Rule names are retried.
- **Busy desktops.** In Normal mode the shell hook wakes the Engine once for every window created, destroyed, activated or retitled anywhere, not only for creations. With the desktop in use that was 64 context switches in 56 s, with CPU at or below one timer tick.
- Also confirmed: single instance, Sweep on start, EcoQoS and low memory priority set.
