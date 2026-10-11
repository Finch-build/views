# 02: Core: config, zone geometry, rule matching

**What to build:** The engine-independent logic: parse and validate the config, turn a Zone into a pixel rectangle, and pick the first matching Rule for a window. Rules reference Zones by name and validation rejects dangling references. A missing Monitor skips the Rule. The config format freezes after this ticket.

**Blocked by:** 01

**Status:** ready-for-agent

- [ ] Malformed config returns an error and never panics
- [ ] Zones tile thirds, halves, quarters and odd widths with no 1 px gaps or overlaps
- [ ] Exe matching is case-insensitive and the first match wins
- [ ] All covered by unit tests
