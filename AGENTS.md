## Agent skills

### Issue tracker

Issues and specs are local markdown files under `.scratch/`. See `docs/agents/issue-tracker.md`.

### Triage labels

Default vocabulary: needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix. See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: root `GLOSSARY.md` + `docs/adr/`. See `docs/agents/domain.md`.

## Code standards

The bar is code a strict reviewer would not want to change.

- Every item earns its place. No speculative parameters, no wrapper with one caller, no option nobody sets.
- Names come from `GLOSSARY.md`. In prose, domain terms are capitalised (Rule, Zone, Sweep).
- A doc comment says what a thing is or promises, in a sentence or two, and never restates the signature. An inline comment explains why, and only where the code cannot.
- No ticket numbers, history or TODOs in code. A decision that outlives its change goes in `docs/adr/`.
- `core` and `engine` hold no OS calls, and the only `cfg` outside a backend is the line that picks one (ADR 0002). A new Engine dependency needs a size argument.
- An `unsafe` block is as small as the call it wraps, with a `// SAFETY:` line wherever the reason is not plain from the call.
- A test's name is a sentence about behaviour, and it asserts through public behaviour.
- `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass before every commit.
