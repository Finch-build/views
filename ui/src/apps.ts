// What the app picker lists, and which of it a search finds.

import type { App, Icon, Installed } from "./commands";

/** One line of the picker: an executable, the name it is known by, and its window where it has one open. */
export interface Entry {
  name: string;
  exe: string;
  icon: Icon | null;
  open?: App;
}

const fold = (exe: string) => exe.toLowerCase();

/** Every installed app, and every running one that is not among them, by name with the open ones first. */
export function listing(running: readonly App[], installed: readonly Installed[]): Entry[] {
  const open = new Map(running.map((app) => [fold(app.exe), app]));
  const known = new Set(installed.map((app) => fold(app.exe)));
  const entries: Entry[] = [
    ...installed.map((app) => ({ ...app, open: open.get(fold(app.exe)) })),
    ...running.filter((app) => !known.has(fold(app.exe))).map((app) => ({ name: app.title || app.exe, exe: app.exe, icon: app.icon, open: app })),
  ];
  return entries.sort((a, b) => Number(!a.open) - Number(!b.open) || a.name.localeCompare(b.name));
}

/** Whether every word of `query` is in the entry's name or executable, whatever its case. */
export function matches(entry: Entry, query: string): boolean {
  const text = `${entry.name} ${entry.exe}`.toLowerCase();
  return query.toLowerCase().split(/\s+/).every((word) => text.includes(word));
}
