# Settings UI

A Tauri 2 window over the config file: vanilla TypeScript in `src`, a thin Rust side in `src-tauri`. It is its own Cargo workspace, so nothing here can reach the Engine.

```sh
npm ci
npm run tauri dev     # the app, reloading as you edit
npm run tauri build   # src-tauri/target/release/views-settings
npm test              # the palette against WCAG AA
```

**A screen** is one file in `src/screens` that exports a `Screen`, and one entry in the list in `src/screens.ts`. Build it from the primitives in `src/ui.ts`. Read the config with `config()` and change it with `edit()`, both from `src/store.ts`; the shell owns saving.

**A command** is a `#[tauri::command]` in `src-tauri/src/main.rs`, its name in `generate_handler!`, and a typed wrapper in `src/commands.ts`.

**Styles** load in the order tokens, base, primitives, shell, screens. Every colour, face and duration is a custom property in `src/styles/tokens.css`. Sizes are multiples of `--cell`. Signal colours mark state and never colour text.

Nothing is fetched at runtime: fonts and art are bundled, and the CSP in `src-tauri/tauri.conf.json` allows only the app's own origin.
