// Layout and Zone names, and the Rules that refer to them.
//
// A Rule names its Layout and Zone, so a rename must follow them there. A deletion
// does not: the Rules stay, and `dangling` finds them.

import type { Config, Rule } from "./config";

/** Whether `name` can be taken: it is not blank and is not one of `others`. */
export const isFree = (name: string, others: readonly string[]) => name.trim() !== "" && !others.includes(name);

/** `name`, or `name-2`, `name-3` and so on, whichever is first not in `taken`. */
export function unique(name: string, taken: readonly string[]): string {
  let candidate = name;
  for (let n = 2; taken.includes(candidate); n++) candidate = `${name}-${n}`;
  return candidate;
}

export function renameLayout(config: Config, from: string, to: string): void {
  for (const layout of config.layout) if (layout.name === from) layout.name = to;
  for (const rule of config.rule) if (rule.layout === from) rule.layout = to;
}

export function renameZone(config: Config, layout: string, from: string, to: string): void {
  for (const zone of config.layout.find((found) => found.name === layout)?.zones ?? []) {
    if (zone.name === from) zone.name = to;
  }
  for (const rule of config.rule) if (rule.layout === layout && rule.zone === from) rule.zone = to;
}

/** The Rules that name a Layout or Zone the config does not have, which core refuses to save. */
export function dangling(config: Config): Rule[] {
  return config.rule.filter(
    (rule) => !config.layout.find((layout) => layout.name === rule.layout)?.zones.some((zone) => zone.name === rule.zone),
  );
}
