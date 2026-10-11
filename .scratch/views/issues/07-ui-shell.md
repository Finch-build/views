# 07: Settings UI shell, design system, save/reload

**What to build:** A Tauri app with vanilla TypeScript that opens from the tray Open settings. It has the Finch design tokens (system light and dark, pixel display font, bundled JetBrains Mono), an ascii.rest title, and a General screen. Config is saved atomically after validation and the engine reloads. The pixel font is chosen with the user before bundling.

**Blocked by:** 02, 05

**Status:** ready-for-agent

- [x] UI process fully exits when the window closes
- [x] No network requests at runtime
- [x] Fonts and ascii.rest are bundled
- [x] Brand palette contrast against WCAG AA is checked and failures reported
- [x] Nothing from the UI stack is linked into the engine
- [x] Reduced motion is honoured and animation pauses when hidden

## Comments

Built in `ui/` (own Cargo workspace; `ui/README.md` says how to run it and add a screen).

- **Fonts.** Display is Pixelify Sans with the owner's pixel digits in front (`unicode-range: U+0030-0039`, sha256 5f21b978…), body is JetBrains Mono; all bundled.
- **ascii.rest** draws the title through `mount` and the `big-text` piece. Its custom-element entry was not used: it hard-codes a remote font URL and injects a style tag, which the no-network CSP forbids.
- **Contrast (WCAG AA).** Used pairs all pass. Brand pairs that fail and are avoided: ink-35 on paper (2.12), ink-55 on ink (3.65), and the dark-surface signal hues on paper (2.06 to 2.62). Light ochre on paper is 3.90: fine as a marker, never as text. `npm test` checks the pairs in use.
- **Verified on Windows:** the built app starts, saves atomically through core, reports whether an Engine heard the reload, refuses an invalid config, leaves no process after close, focuses the first window on a second launch, loads nothing over the network, honours reduced motion and pauses animation while minimised.
- **Not verified:** the macOS build.
