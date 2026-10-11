# 08: UI: Monitors and Layouts screens

**What to build:** A Monitors screen showing displays to scale in their real arrangement with editable friendly names, and a Layouts screen with a Zone editor: presets for halves, thirds, quarters and 2x2, plus custom splits by dragging dividers on a character-cell grid.

**Blocked by:** 07

**Status:** ready-for-agent

- [ ] Edits persist and the engine reloads them
- [x] Presets produce Zones that tile with no gaps
- [ ] Friendly names survive a monitor reconnect (matched by stable ID)

## Comments

Built: the Monitors screen (to scale, in arrangement, names editable and kept by id, detached ones listed) and the Layouts screen with a Zone editor.

- **Dividers.** The editor shows a Layout on a 12x6 grid. A divider is the stretch of a grid line where Zone edges overlap; dragging it, or moving it with the arrow keys, moves all of those edges together, so a Layout that tiles keeps tiling. Split and merge act on the selected Zone.
- **Rules.** Renaming a Layout or Zone renames it in the Rules that use it. Deleting one leaves those Rules in place and says how many now point at nothing; core refuses the save until they are corrected.
- **Tested** (`npm test`): every preset tiles with no gaps on its own grid and on the editor's, plus moving, splitting, merging and renaming.
- **Still to verify by hand** (the first and third boxes): in the built app against the real Windows backend, that an edit persists and the Engine reloads it, and that a friendly name survives unplugging and replugging a monitor.
