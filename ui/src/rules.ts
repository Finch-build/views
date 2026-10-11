// What is true of a Rule given the Monitors that are attached, and the edits the Rules screen makes.

import type { Monitor } from "./commands";
import type { Config, Rule } from "./config";

/** Empty optional text is stored as absent. */
export const optional = (text: string) => (text.trim() === "" ? null : text);

/** Whether the Engine passes over `rule` because its Monitor is not attached. */
export const skipped = (rule: Rule, present: readonly Monitor[]) => !present.some((monitor) => monitor.id === rule.monitor);

/** The Monitor as the user knows it: by the name they gave it, else the one it reports, else its id. */
export function monitorName(id: string, config: Config, present: readonly Monitor[]): string {
  return config.monitor.find((entry) => entry.id === id)?.name ?? present.find((monitor) => monitor.id === id)?.name ?? id;
}

/** Whether an earlier Rule takes every window that the Rule at `index` could match: the same exe, no further condition, and a Monitor that is attached. */
export function shadowed(rules: readonly Rule[], index: number, present: readonly Monitor[]): boolean {
  const exe = rules[index]!.exe.toLowerCase();
  return rules
    .slice(0, index)
    .some((earlier) => earlier.exe.toLowerCase() === exe && earlier.class === null && earlier.title_contains === null && !skipped(earlier, present));
}

/** Moves the Rule at `index` one place earlier or later, unless it is already at that end. */
export function shift(rules: Rule[], index: number, by: -1 | 1): void {
  if (!rules[index + by]) return;
  rules.splice(index + by, 0, ...rules.splice(index, 1));
}

/** Points `rule` at Layout `layout`, keeping its Zone if the Layout has one of that name and taking the first otherwise. */
export function retarget(config: Config, rule: Rule, layout: string): void {
  const zones = config.layout.find((found) => found.name === layout)?.zones ?? [];
  rule.layout = layout;
  if (!zones.some((zone) => zone.name === rule.zone)) rule.zone = zones[0]?.name ?? "";
}

/** A Rule for `exe` with no other condition, sending it to the first attached Monitor and the first Zone of the first Layout. */
export function fresh(exe: string, config: Config, present: readonly Monitor[]): Rule {
  const rule: Rule = { exe, class: null, title_contains: null, monitor: present[0]!.id, layout: "", zone: "", reapply_after_ms: 0 };
  retarget(config, rule, config.layout[0]!.name);
  return rule;
}
