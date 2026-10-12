// Sets the version of the Engine and the Settings UI, which are separate Cargo workspaces
// and must agree. Merging the change to master publishes the release.

import { execSync } from "node:child_process";
import { readFileSync, writeFileSync } from "node:fs";

const version = process.argv[2];
if (!/^\d+\.\d+\.\d+$/.test(version ?? "")) throw new Error("usage: npm run bump <major.minor.patch>");

for (const workspace of ["..", "src-tauri"]) {
  const manifest = `${workspace}/Cargo.toml`;
  writeFileSync(manifest, readFileSync(manifest, "utf8").replace(/^version = ".*"$/m, `version = "${version}"`));
  execSync("cargo update --workspace --offline", { cwd: workspace, stdio: "inherit" });
}
