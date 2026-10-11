// The one copy of the config that every screen reads and edits.

import { loadConfig, saveConfig } from "./commands";
import type { Config } from "./config";

let draft: Config | undefined;
/** The file's contents as JSON, to compare the draft against and revert to. */
let disk = "";
let failure = "";
let heard: boolean | undefined;
const listeners: ((replaced: boolean) => void)[] = [];

/** The draft config. Screens are drawn only while there is one. */
export function config(): Config {
  if (!draft) throw new Error("no config is loaded");
  return draft;
}

/** False until the file has loaded, and for as long as it cannot be read. */
export const ready = () => draft !== undefined;

export const dirty = () => ready() && JSON.stringify(draft) !== disk;

/** Why the last load or save failed, or empty. */
export const error = () => failure;

/** Whether an Engine reloaded the last save. Undefined once anything has changed since. */
export const engineHeard = () => heard;

/** Calls `listener` after every change. `replaced` is true when the draft is a new object, not an edit of the old one. */
export function subscribe(listener: (replaced: boolean) => void): void {
  listeners.push(listener);
}

export async function load(): Promise<void> {
  draft = undefined;
  await settle(true, async () => {
    draft = await loadConfig();
    disk = JSON.stringify(draft);
  });
}

export function edit(change: (config: Config) => void): void {
  change(config());
  failure = "";
  heard = undefined;
  emit(false);
}

export function revert(): void {
  draft = JSON.parse(disk) as Config;
  failure = "";
  heard = undefined;
  emit(true);
}

/** Has the core validate the draft and write it, then asks the Engine to reload. */
export async function save(): Promise<void> {
  const saving = JSON.stringify(config());
  await settle(false, async () => {
    heard = await saveConfig(config());
    disk = saving;
  });
}

async function settle(replaced: boolean, action: () => Promise<void>): Promise<void> {
  failure = "";
  heard = undefined;
  try {
    await action();
  } catch (reason) {
    failure = String(reason);
  }
  emit(replaced);
}

function emit(replaced: boolean): void {
  for (const listener of listeners) listener(replaced);
}
