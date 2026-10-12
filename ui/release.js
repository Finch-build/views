// Publishes the installer that `npm run bundle` built as the GitHub release `tag`, with the
// `latest.json` that tells an installed Settings UI there is a newer version to fetch.

import { execFileSync } from "node:child_process";
import { readdirSync, readFileSync, writeFileSync } from "node:fs";

const tag = process.argv[2];
const folder = "src-tauri/target/release/bundle/nsis";
const installer = readdirSync(folder).find((file) => file.endsWith("-setup.exe"));
const url = `https://github.com/${process.env.GITHUB_REPOSITORY}/releases/download/${tag}/${installer}`;
const signature = readFileSync(`${folder}/${installer}.sig`, "utf8");

writeFileSync("latest.json", JSON.stringify({ version: tag.slice(1), platforms: { "windows-x86_64": { signature, url } } }));
execFileSync("gh", ["release", "create", tag, `${folder}/${installer}`, "latest.json", "--generate-notes"], { stdio: "inherit" });
