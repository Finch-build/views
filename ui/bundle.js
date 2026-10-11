// Builds the installer: the Engine in release, placed where Tauri looks for a binary
// to ship beside the Settings UI, then the bundle itself.

import { execSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";

const run = (command) => execSync(command, { stdio: "inherit" });
const exe = process.platform === "win32" ? ".exe" : "";
const target = execSync("rustc --print host-tuple", { encoding: "utf8" }).trim();

run("cargo build --release --locked --manifest-path ../Cargo.toml --bin views");
mkdirSync("src-tauri/binaries", { recursive: true });
copyFileSync(`../target/release/views${exe}`, `src-tauri/binaries/views-${target}${exe}`);
run("npx tauri build --config src-tauri/tauri.bundle.conf.json");
