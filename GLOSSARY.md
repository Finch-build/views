# views

A background utility that moves and resizes application windows to a chosen monitor and tile zone when they open.

## Language

**Engine**:
The always-running background executable that watches for windows and applies rules.
_Avoid_: daemon, service, tiler

**Settings UI**:
The separate on-demand executable where the user edits config. Runs only while open.
_Avoid_: app, frontend, GUI

**Rule**:
A match on a window (exe, optional class, optional title substring) that assigns it a monitor, layout, and zone.
_Avoid_: profile, mapping

**Layout**:
A named set of zones expressed as fractions of a monitor's work area.
_Avoid_: grid, template

**Zone**:
One named rectangle within a layout.
_Avoid_: tile, slot, region

**Sweep**:
One pass applying rules to all already-open windows. Runs on engine start, config reload, and gaming exit.

**Gaming mode**:
Engine state where window watching is fully suspended to cost the running game nothing. Lasts while the game process is alive.
_Avoid_: game mode, quiet mode

**Paused**:
Engine state where the user has manually suspended rule application.

**Tileable window**:
A visible, top-level, titled, un-owned, non-tool, non-cloaked window the engine is allowed to move.
