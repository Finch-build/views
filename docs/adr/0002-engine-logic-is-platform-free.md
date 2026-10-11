# The Engine's logic is platform-free; a backend is a Desktop and a Host

The Engine is a state machine in `views-engine` that consumes `Event`s and acts through two traits from `views-platform`:

- `Desktop` is stateless: list Monitors and windows, describe a window, place one. The Settings UI and `views-debug` use it without starting anything.
- `Host` is the running Engine's event loop, tray icon and timers.

Each backend crate (`views-windows`, `views-macos`) exports the same four names: `Desktop`, `Host`, `config_path` and `request_reload`. The only `cfg` outside a backend is the line that picks one.

Consequences:

- Rule application, retries, reapply, pause and Gaming mode are tested once, against fakes, with no operating system.
- `Host::run` pushes events into a handler instead of the Engine pulling them, because AppKit owns the main loop on macOS.
- There is no `ipc` crate, though the plan had one. Asking the Engine to reload means posting a message to a window the backend created, so the sender must agree with the backend on names. It lives in the backend as `request_reload`.
