# 02: Core: config, zone geometry, rule matching

**What to build:** The engine-independent logic: parse and validate the config, turn a Zone into a pixel rectangle, and pick the first matching Rule for a window. Rules reference Zones by name and validation rejects dangling references. A missing Monitor skips the Rule. The config format freezes after this ticket.

**Blocked by:** 01

**Status:** ready-for-agent

- [x] Malformed config returns an error and never panics
- [x] Zones tile thirds, halves, quarters and odd widths with no 1 px gaps or overlaps
- [x] Exe matching is case-insensitive and the first match wins
- [x] All covered by unit tests

## Comments

Done in `crates/core` (17 unit tests).

- **Format.** One TOML file: `version`, `[gaming]`, `[[monitor]]` (friendly names), `[[layout]]`, `[[rule]]`. Unknown keys and unknown versions are errors, reported with their line.
- **Zones are grid cells, not decimal fractions** (ADR 0001). A Layout has `columns` and `rows`; a Zone is `column`, `row`, `width`, `height` in cells. Edges fall on shared grid lines, so tiling is gap-free by construction; the tests check exact cover for halves, thirds, quarters, 2x2 and odd sizes.
- **Matching.** Exe compared case-insensitively, optional exact class, optional title substring. First match wins; a Rule whose Monitor is not attached is skipped and the next one tried.
- **Validation** rejects duplicate Layout or Zone names, Zones outside their grid, and Rules that reference a missing Layout or Zone.
