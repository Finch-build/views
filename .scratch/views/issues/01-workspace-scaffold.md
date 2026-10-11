# 01: Workspace scaffold and CI

**What to build:** A Cargo workspace with the shared size-optimised release profile, the core, platform, Windows backend, macOS stub, engine and ipc crates, and CI that checks formatting, lints and tests on Windows and macOS.

**Blocked by:** None (can start immediately)

**Status:** ready-for-human

- [x] macOS backend is a stub that compiles
- [x] Core crate has no OS calls and compiles on any target
- [ ] CI green on Windows and macOS (fmt, clippy -D warnings, test)

## Comments

Built the workspace: root `Cargo.toml` with `[profile.release]` (`opt-level = "z"`,
lto, `codegen-units = 1`, `strip`) — Cargo reads profiles only from the workspace
root, so members carry none. Crates: `crates/{core,platform,windows,macos,engine,ipc}`
(`views-core` … `views-ipc`). `views-platform` defines the seam: `PlatformBackend`
(`monitors`, `tileable_windows`, `place_window`) plus `Monitor`, `WindowInfo`,
`MonitorId`, `WindowHandle`, `PlaceError`. `views-core` has `Rect` and no OS calls.
Windows and macOS backends are stubs that compile on every target; `views-engine`
picks one with `#[cfg(windows)]`; `views-ipc` is empty until ticket 05.

Verified locally: `cargo fmt --all --check`, `cargo build`, `cargo clippy --workspace
--all-targets -- -D warnings` all clean. **`cargo test` cannot link on this machine**
— no MSVC linker installed (rustup installed the msvc toolchain without Visual Studio,
so `link.exe` resolves to MSYS's `link`). The two tests compile; they run in CI
(`.github/workflows/ci.yml`), which also needs a remote to exist.
