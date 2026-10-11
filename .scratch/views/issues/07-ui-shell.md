# 07: Settings UI shell, design system, save/reload

**What to build:** A Tauri app with vanilla TypeScript that opens from the tray Open settings. It has the Finch design tokens (system light and dark, pixel display font, bundled JetBrains Mono), an ascii.rest title, and a General screen. Config is saved atomically after validation and the engine reloads. The pixel font is chosen with the user before bundling.

**Blocked by:** 02, 05

**Status:** ready-for-agent

- [ ] UI process fully exits when the window closes
- [ ] No network requests at runtime
- [ ] Fonts and ascii.rest are bundled
- [ ] Brand palette contrast against WCAG AA is checked and failures reported
- [ ] Nothing from the UI stack is linked into the engine
- [ ] Reduced motion is honoured and animation pauses when hidden
