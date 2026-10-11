# 06: Gaming mode

**What to build:** The Normal, Gaming and Paused state machine. Gaming mode starts when a listed game or a fullscreen Direct3D app takes the foreground, deregisters the shell hook, trims the working set once, and ends when the game process exits, followed by a Sweep. The tray override wins in both directions.

**Blocked by:** 05

**Status:** ready-for-agent

- [x] With a listed game running, opening other apps causes zero engine wakeups
- [x] Working set under 1 MB after trim
- [x] Alt-tabbing out of the game does not exit Gaming mode
- [x] Rules resume after the game exits
- [x] Thread-pool cost of the process wait is measured and reported

## Comments

- **Wakeups.** With a listed game running and two other apps opened: 0 context switches. Gaming mode forced on from the tray: 0 in 60 s.
- **Trim.** Working set 488 to 496 K after the trim (216 to 224 K private).
- **Lifetime.** Gaming mode survived the game losing the foreground and ended when its process exited, followed by a Sweep.
- **Thread-pool cost: none.** The process handle is waited on in the Engine's own message loop (ADR 0003), so there is no pool thread to measure.
- **Tray override** is Auto / On / Off; On and Off both outrank detection.
- **Not tried:** fullscreen detection against a real exclusive-fullscreen Direct3D app. The Engine's side of it is unit-tested.

Numbers are from the GNU toolchain on the development machine; CI and releases build with MSVC, which has not been measured.
