# macOS

The Engine moves other apps' windows through the Accessibility API, so macOS asks the user to allow it under System Settings, Privacy & Security, Accessibility. The Engine asks once at start. Until the grant, it watches nothing and places nothing, and its menu-bar item still works. When the system reports that access changed, the Engine reloads its config and sweeps. If that report does not arrive, quit and start the Engine again.

Development builds that are unsigned, or signed ad hoc, are a new program to the system on every rebuild, so the grant is lost each time. Sign with a stable identity to keep it.

Limits that are documented rather than solved:

- A window cannot be moved to another Space. The Engine places windows only on the Space they are on.
- A window in native fullscreen is not tiled, and neither is a minimized one.
- Gaming mode does not exist: the menu has no entry for it, and no process is watched for exit.
- The Accessibility API is not available to a sandboxed app, so the Engine cannot be sandboxed, and so cannot be sold through the Mac App Store.

The Engine names a window by its app's process id and its CoreGraphics window number. Accessibility offers no such number, so it reads it with `_AXUIElementGetWindow`, a private function. Every macOS window manager relies on it; a future macOS could remove it, and the Engine would then find no windows.

Coordinates are points, with the origin at the top left of the primary display. A Monitor's id is its display UUID.

The config is `~/Library/Application Support/views/config.toml`. The Settings UI is the `views-settings` executable beside the Engine's.
