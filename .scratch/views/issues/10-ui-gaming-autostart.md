# 10: UI: Gaming mode screen and autostart

**What to build:** A Gaming mode screen with on/off, the fullscreen-detection toggle and games list, and an opt-in autostart toggle on General that writes the HKCU Run key.

**Blocked by:** 06, 07

**Status:** ready-for-agent

- [ ] Changes take effect in the engine after save
- [x] Autostart is off by default and never enabled silently

## Comments

Built: the Gaming mode screen (enabled, fullscreen detection, the games list by typed exe name) and a Startup panel on General.

- **Autostart** is the `views` value under `HKCU\Software\Microsoft\Windows\CurrentVersion\Run`, pointing at `views.exe` beside the Settings executable. The toggle shows what the registry holds and changes it only when flipped; it is not part of the config or the save. Turning it on fails with a message if the Engine is not there. Tested against a scratch key, and once against the real Run key, which was left as found (no `views` value).
- **Platform.** One `features` command decides, at compile time, that Gaming mode and autostart exist only on Windows; the navigation and the General screen follow it.
- **Open box.** "Changes take effect in the engine after save" rides the same save and reload as every screen, and the Engine's handling of each setting is unit-tested, but it has not been checked by hand in the built app.
