# 04: Engine: window-created to window-moved

**What to build:** A single-instance engine that, when a Tileable window is created, moves it to its Rule's Zone using the shell hook. A new window that is not yet matchable gets one one-shot retry. A Sweep runs on start. The engine opts into EcoQoS and low memory priority.

**Blocked by:** 03

**Status:** ready-for-agent

- [ ] Launching a ruled app moves it within 50 ms for well-behaved apps
- [ ] Idle: 0% CPU, no wakeups, one thread (below 3 MB private working set, under 1 MB binary)
- [ ] A title-based Rule works for apps that set their title late
- [ ] Measured numbers reported against the budgets
