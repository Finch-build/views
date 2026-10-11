# 12: Packaging and README

**What to build:** A per-user Windows installer needing no admin rights, a macOS .app bundle with signing and notarization notes, and a README.

**Blocked by:** 09, 10, 11

**Status:** ready-for-human

- [ ] Windows install and uninstall work without admin
- [x] README notes the unsigned-build Accessibility caveat
- [x] Final budget numbers are recorded

## Comments

- **Windows installer.** `npm run bundle` in `ui` builds the Engine, hands it to Tauri as a binary to ship beside the Settings UI, and produces an NSIS installer in per-user mode (4.7 MB). Hooks in `ui/src-tauri/installer.nsh` ask a running Engine to quit before its file is replaced or removed, and uninstalling deletes the autostart entry. Opening the Settings UI starts the Engine, so the Start menu entry is all a user needs.
- **The open box: the installer builds but has not been run.** Installing on the development machine was not permitted to the agent. To close it, run the setup exe as a standard user, check that views appears in the Start menu and the tray, turn autostart on, uninstall, and check that the install directory and the `views` value under the Run key are gone.
- **macOS.** The same command on a Mac produces `views.app`; signing and notarization are in `docs/macos.md`. The Release workflow builds both.
- **Budgets** are in the README: Engine 533 KB, one thread, 1.2 to 1.7 MB private working set idle, 0.5 MB in Gaming mode, no idle wakeups, 4 to 13 ms from shown to moved. All from the GNU toolchain; MSVC is unmeasured.
