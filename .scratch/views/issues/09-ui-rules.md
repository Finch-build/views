# 09: UI: Rules screen

**What to build:** A Rules screen where the user adds a Rule by picking a running app (exe and icon) or browsing for an exe, then picks Monitor, Layout and Zone.

**Blocked by:** 08

**Status:** ready-for-agent

- [ ] A Rule created in the UI works in the engine without a restart
- [x] Rules referencing a missing Monitor are shown as skipped
- [x] Deleting or renaming a Zone updates or flags the Rules that use it

## Comments

Built: the Rules in priority order with Up, Down, Edit and Remove; an in-page picker of running apps (with icons) or a file browse; Monitor, Layout and Zone chosen from what exists.

- **Skipped** Rules (Monitor not attached) are marked and stay editable.
- **Renames and deletes.** Renaming a Layout or Zone follows it into the Rules that use it. A Rule left naming a missing one is marked, shows the missing value in its select, and blocks saving until corrected.
- **Shadowed** Rules are marked too: one placed after a Rule for the same exe with no class or title condition and an attached Monitor can never match.
- **Icons** are read in the UI's own crate (`icons.rs`), Windows only; the backends stay free of UI concerns.
- **Open box.** "A Rule created in the UI works in the engine without a restart" is covered piecewise (the UI saves through core and requests a reload; the Engine reloads and Sweeps) but has not been done end to end by hand in the built app.
