// The app picker: a search over the apps that are open or installed, and a file browser for the rest.

import { listing, matches, type Entry } from "./apps";
import { browse, installedApps, runningApps, type App, type Icon, type Installed } from "./commands";
import { h } from "./dom";
import { button, panel, row, status, textField } from "./ui";

/** Finding the installed apps takes a second or two, so it is done once and begun before it is needed. */
let installed: Promise<Installed[]> | undefined;

/** The icon, one canvas pixel to an icon pixel; the page scales it. Empty where there is none. */
function paint(icon: Icon | null): HTMLElement {
  const slot = h("span", { className: "icon" });
  if (icon) {
    const canvas = h("canvas", { width: icon.width, height: icon.height });
    canvas.getContext("2d")!.putImageData(new ImageData(Uint8ClampedArray.from(icon.rgba), icon.width), 0, 0);
    slot.append(canvas);
  }
  return slot;
}

/**
 * A panel, hidden until opened, from which the user chooses an app. `pick` gets the executable's
 * file name, and the app's open window where it has one. `closed` runs when they choose nothing.
 */
export function appPicker(title: string, pick: (exe: string, from?: App) => void, closed: () => void): { panel: HTMLElement; open: () => void } {
  installed ??= installedApps();
  let entries: Entry[] = [];
  const apps = h("div", { className: "apps" });
  const search = textField("", draw);
  search.type = "search";
  search.placeholder = "Search apps";
  search.ariaLabel = "Search apps";

  function choose(exe: string, from?: App): void {
    frame.hidden = true;
    pick(exe, from);
  }

  function close(): void {
    frame.hidden = true;
    closed();
  }

  function draw(): void {
    const found = entries.filter((entry) => matches(entry, search.value));
    apps.replaceChildren(
      ...found.map((entry) => {
        const add = button("Add", () => choose(entry.exe, entry.open));
        add.ariaLabel = `Add ${entry.exe}`;
        const detail = entry.open ? `${entry.exe} · open now` : entry.exe;
        return h("div", { className: "row app" }, paint(entry.icon), h("div", {}, h("p", {}, entry.name), h("p", { className: "soft" }, detail)), add);
      }),
    );
    if (!found.length) apps.append(status("none", "No app matches. Browse for its executable instead."));
  }

  async function open(): Promise<void> {
    frame.hidden = false;
    search.value = "";
    apps.replaceChildren(status("none", "Looking for apps."));
    search.focus();
    try {
      entries = listing(await runningApps(), await installed!);
      draw();
    } catch (reason) {
      apps.replaceChildren(status("bad", String(reason)));
    }
  }

  const browseButton = button("Browse", () => void browse().then((exe) => (exe ? choose(exe) : browseButton.focus())));
  const frame = panel(
    title,
    search,
    apps,
    row("Not listed", h("div", { className: "actions" }, browseButton)),
    row("", h("div", { className: "actions" }, button("Cancel", close))),
  );
  frame.hidden = true;
  frame.onkeydown = (event) => {
    if (event.key === "Escape") close();
  };
  return { panel: frame, open: () => void open() };
}
