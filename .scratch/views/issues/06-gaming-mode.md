# 06: Gaming mode

**What to build:** The Normal, Gaming and Paused state machine. Gaming mode starts when a listed game or a fullscreen Direct3D app takes the foreground, deregisters the shell hook, trims the working set once, and ends when the game process exits, followed by a Sweep. The tray override wins in both directions.

**Blocked by:** 05

**Status:** ready-for-agent

- [ ] With a listed game running, opening other apps causes zero engine wakeups
- [ ] Working set under 1 MB after trim
- [ ] Alt-tabbing out of the game does not exit Gaming mode
- [ ] Rules resume after the game exits
- [ ] Thread-pool cost of the process wait is measured and reported
