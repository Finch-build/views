import { monitors as attached, type Monitor } from "../commands";
import { h } from "../dom";
import type { Screen } from "../screens";
import { config, edit } from "../store";
import { button, panel, row, status, textField } from "../ui";

export const monitors: Screen = {
  name: "Monitors",
  render() {
    const map = h("div");
    const remembered = h("div");
    const absent = panel("Not connected", remembered);
    absent.hidden = true;

    function drawAbsent(present: Monitor[]): void {
      const gone = config().monitor.filter((entry) => !present.some((monitor) => monitor.id === entry.id));
      remembered.replaceChildren(
        ...gone.map((entry) =>
          row(
            entry.id,
            h("span", {}, entry.name),
            button("Remove", () => {
              edit((c) => (c.monitor = c.monitor.filter((kept) => kept !== entry)));
              drawAbsent(present);
              remembered.querySelector("button")?.focus();
            }),
          ),
        ),
      );
      absent.hidden = gone.length === 0;
    }

    void attached().then(
      (present) => {
        map.replaceChildren(present.length ? arrangement(present) : status("none", "No Monitors were found."));
        drawAbsent(present);
      },
      (reason) => map.replaceChildren(status("bad", String(reason))),
    );
    return [panel("Arrangement", map), absent];
  },
};

/** The Monitors where they sit on the desktop, to scale. */
function arrangement(present: Monitor[]): HTMLElement {
  const left = Math.min(...present.map((monitor) => monitor.bounds.left));
  const top = Math.min(...present.map((monitor) => monitor.bounds.top));
  const width = Math.max(...present.map((monitor) => monitor.bounds.right)) - left;
  const height = Math.max(...present.map((monitor) => monitor.bounds.bottom)) - top;
  const map = h("div", { className: "map" });
  map.style.setProperty("--aspect", String(width / height));
  map.append(
    ...present.map(({ id, name, bounds }) => {
      const field = textField(config().monitor.find((entry) => entry.id === id)?.name ?? "", (value) => rename(id, value));
      field.placeholder = name;
      field.ariaLabel = `Name of ${name}`;
      const monitor = h("div", { className: "monitor" }, field, h("p", {}, `${bounds.right - bounds.left} × ${bounds.bottom - bounds.top}`));
      monitor.style.cssText = [
        `--x: ${(bounds.left - left) / width}`,
        `--y: ${(bounds.top - top) / height}`,
        `--w: ${(bounds.right - bounds.left) / width}`,
        `--h: ${(bounds.bottom - bounds.top) / height}`,
      ].join(";");
      return monitor;
    }),
  );
  return map;
}

/** Gives the Monitor `id` the name `name`; a blank name takes its entry away, so it shows what it reports again. */
function rename(id: string, name: string): void {
  edit((c) => {
    const entry = c.monitor.find((found) => found.id === id);
    if (name.trim() === "") c.monitor = c.monitor.filter((kept) => kept !== entry);
    else if (entry) entry.name = name;
    else c.monitor.push({ id, name });
  });
}
