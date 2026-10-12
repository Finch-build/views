# views

Moves each application window to the monitor and zone you chose for it, the moment it opens.

views is two programs. The **Engine** sits in the tray and does the moving. It is built to cost nothing beside a game: one thread, no polling, no timers while idle, and it stands down entirely while a game runs. The **Settings UI** edits the config and exits when you close it.

Windows is supported. A macOS backend is written and builds, but [has not yet been run](docs/macos.md).

## Install

Run `views_<version>_x64-setup.exe`. It installs for the current user only and asks for no administrator rights. Open **views** from the Start menu: that starts the Engine and shows the settings. To have the Engine start when you sign in, turn that on under General; it is off until you do.

After that it updates itself: when a newer release exists, the General screen offers it, and one press installs it and reopens the settings.

Uninstalling stops the Engine and removes the sign-in entry. Your config stays in `%APPDATA%iews`.

On macOS, `views.app` is built unsigned. macOS ties the Accessibility permission the Engine needs to a signature, so an unsigned build loses that permission every time it is rebuilt, and other Macs will refuse to open it. [docs/macos.md](docs/macos.md) covers signing.

## How it works

A **Layout** is a grid over a monitor's work area, and a **Zone** is a named rectangle of its cells. A **Rule** matches a window by its executable, and optionally its class or part of its title, and sends it to a Zone of a Layout on a Monitor.

When a window opens, the first Rule that matches it places it. A Rule whose Monitor is not connected is skipped, so one config can serve a laptop docked and undocked. Windows that are already open are placed in a **Sweep**: when the Engine starts, when the config is reloaded, and when a game exits.

**Gaming mode** begins when a game you have listed takes the foreground and lasts until its process exits. For that time the Engine watches nothing and gives its memory back.

The tray menu opens the settings, pauses the Engine, forces Gaming mode on or off, and quits. Its tooltip reports a config that failed to load and any Rules being skipped.

## Config

The Settings UI writes the config; it is also plain TOML you can edit, at `%APPDATA%\views\config.toml`. The Engine reads it at start and when the Settings UI saves.

```toml
version = 1

[gaming]
enabled = true
detect_fullscreen = false          # treat exclusive-fullscreen Direct3D apps as games
games = ["eldenring.exe"]

[[monitor]]                        # your own name for a monitor
id = '\\?\DISPLAY#DELA0EC#5&2f4b5d2&0&UID4357#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}'
name = "Left"

[[layout]]
name = "thirds"
columns = 3
rows = 1
zones = [
    { name = "left",   column = 0, row = 0, width = 1, height = 1 },
    { name = "middle", column = 1, row = 0, width = 1, height = 1 },
    { name = "right",  column = 2, row = 0, width = 1, height = 1 },
]

[[rule]]                           # in priority order: the first match wins
exe = "discord.exe"                # compared without regard to case
title_contains = "Discord"         # optional; `class` is the other optional condition
monitor = '\\?\DISPLAY#DELA0EC#5&2f4b5d2&0&UID4357#{e6f07b5f-ee97-4a90-b076-33f57bf4eaa7}'
layout = "thirds"
zone = "right"
reapply_after_ms = 500             # optional: place again, for apps that resize once loaded
```

Zones are whole cells, so two Zones that share a grid line share a pixel edge on every monitor: no gaps and no overlaps ([ADR 0001](docs/adr/0001-zones-are-grid-cells.md)). An invalid config is never applied; the Engine keeps the last good one and says why in its tooltip.

`views-debug list` prints the monitor ids and the windows the Engine would consider.

## What it costs

Measured on Windows 11 with a release build. The binary size is the MSVC build that ships; the rest were taken on a GNU-toolchain build.

| | Budget | Measured |
|---|---|---|
| Engine binary | under 1 MB | 397 KB |
| Threads | 1 | 1 |
| Private working set, idle | under 3 MB | 1.2 to 1.7 MB |
| Working set in Gaming mode | under 1 MB | 0.5 MB |
| Wakeups, idle desktop, 60 s | 0 | 0 |
| Wakeups while a game runs | 0 | 0 |
| Window shown to window moved | under 50 ms | 4 to 13 ms |

The Engine never runs code inside another process, and never runs elevated, so it leaves the windows of elevated apps where they are.

## Building

Rust (stable) and Node 24.

```sh
cargo build --release              # target/release/views and views-debug
cargo test --workspace

cd ui
npm ci
npm run tauri dev                  # the Settings UI alone
npm run bundle                     # the installer, or views.app, with the Engine inside
```

`npm run bundle` signs the installer for the updater, so it needs the signing key in `TAURI_SIGNING_PRIVATE_KEY`.

## Releasing

```sh
cd ui
npm run bump 0.2.0                 # the version, in both Cargo workspaces
```

Merge that to main. The Release workflow publishes a GitHub release for any version on main that has none, and installed copies offer it the next time their settings are opened ([ADR 0004](docs/adr/0004-the-settings-ui-updates-both-executables.md)).

## Layout

```
crates/core       config, Zone geometry, Rule matching; no OS calls
crates/platform   the two traits a backend implements
crates/windows    the Windows backend
crates/macos      the macOS backend
crates/engine     the Engine's logic, its executable, and views-debug
ui                the Settings UI: Tauri, vanilla TypeScript
docs/adr          the decisions that shaped it
```

`GLOSSARY.md` defines the words used here and in the code.
