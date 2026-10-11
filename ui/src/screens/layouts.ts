import { monitors } from "../commands";
import type { Layout, Zone } from "../config";
import { h } from "../dom";
import { dangling, renameLayout, renameZone, isFree, unique } from "../names";
import type { Screen } from "../screens";
import { config, edit } from "../store";
import { button, panel, row, status, textField } from "../ui";
import { dividers, fine, fromPreset, merge, move, presets, split, range, type Divider, type Preset } from "../zones";

/** The Layout being edited and its selected Zone, by name. They outlast a redraw of the screen. */
let chosen = "";
let picked = "";

/** How a Zone operation reads on its button, and the Zones it would leave, or nothing if it cannot be done. */
const operations: [label: string, make: (view: Layout, zone: Zone) => Zone[] | undefined][] = [
  ["Split columns", (view, zone) => split(view, zone, "x", unique(zone.name, names(view)))],
  ["Split rows", (view, zone) => split(view, zone, "y", unique(zone.name, names(view)))],
  ["Merge right", (view, zone) => merge(view, zone, "x")],
  ["Merge below", (view, zone) => merge(view, zone, "y")],
];

const names = (layout: Layout) => layout.zones.map((zone) => zone.name);
const key = ({ axis, at, from }: Divider) => `${axis}${at}:${from}`;
const count = (n: number, noun: string) => `${n} ${noun}${n === 1 ? "" : "s"}`;

export const layouts: Screen = {
  name: "Layouts",
  render() {
    const notice = h("div", { role: "status" });
    const rows = h("div");
    const surface = h("div", { className: "zones" });
    const add = button("Add Layout", addLayout);
    add.dataset.key = "add";
    const presetBar = h("div", { className: "actions" }, ...presets.map((preset) => button(preset.label, () => apply(preset))));
    const stage = row("Grid", surface);
    stage.classList.add("stage");
    const toolBar = h("div", { className: "actions" });
    const editor = panel(
      "Zones",
      row("Preset", presetBar),
      stage,
      row("Zone", toolBar),
    );

    // A failure to read the Monitors leaves the editor at the aspect of a common one.
    void monitors().then(([first]) => {
      if (first) surface.style.setProperty("--aspect", String((first.work_area.right - first.work_area.left) / (first.work_area.bottom - first.work_area.top)));
    }, () => {});

    /** The Divider under the pointer, which outlives the redraws its moves cause. */
    let dragging: Divider | undefined;

    function layout(): Layout | undefined {
      return config().layout.find((found) => found.name === chosen) ?? config().layout[0];
    }

    function addLayout(): void {
      const name = unique("layout", config().layout.map((found) => found.name));
      edit((c) => c.layout.push({ name, columns: 1, rows: 1, zones: [{ name: "full", column: 0, row: 0, width: 1, height: 1 }] }));
      chosen = name;
      picked = "";
      draw(`name:${config().layout.length - 1}`);
    }

    function apply(preset: Preset): void {
      const target = layout();
      if (!target) return;
      edit(() => Object.assign(target, fromPreset(preset)));
      picked = "";
      draw();
    }

    /** A text field for a name. It holds the stored name as its default, and takes a new one only while that is free. */
    function nameField(name: string, taken: () => string[], commit: (name: string, previous: string) => void): HTMLInputElement {
      const input = textField(name, (value) => {
        const free = isFree(value, taken().filter((other) => other !== input.defaultValue));
        input.ariaInvalid = free ? null : "true";
        if (free) {
          commit(value, input.defaultValue);
          input.defaultValue = value;
        }
        announce();
      });
      input.defaultValue = name;
      input.onblur = () => {
        input.value = input.defaultValue;
        input.ariaInvalid = null;
        announce();
      };
      return input;
    }

    function drawRows(): void {
      rows.replaceChildren(
        ...config().layout.map((owner, index) => {
          const field = nameField(
            owner.name,
            () => config().layout.map((other) => other.name),
            (name, previous) =>
              edit((c) => {
                if (chosen === previous) chosen = name;
                renameLayout(c, previous, name);
              }),
          );
          field.ariaLabel = "Layout name";
          field.dataset.key = `name:${index}`;
          const choose = button("Edit", () => {
            chosen = owner.name;
            picked = "";
            draw(`edit:${index}`);
          });
          choose.ariaPressed = String(owner === layout());
          choose.dataset.key = `edit:${index}`;
          return row(
            count(owner.zones.length, "Zone"),
            field,
            choose,
            button("Delete", () => {
              edit((c) => c.layout.splice(index, 1));
              draw("add");
            }),
          );
        }),
      );
      if (!config().layout.length) rows.append(status("none", "No Layouts."));
      rows.append(row("", h("div", { className: "actions" }, add)));
    }

    function announce(): void {
      const lost = dangling(config()).length;
      notice.replaceChildren(
        ...(lost
          ? [status("bad", `${count(lost, "Rule")} ${lost === 1 ? "needs" : "need"} a Layout or Zone that no longer exists. Correct the Rules to save.`)]
          : []),
        ...(document.querySelector('[aria-invalid="true"]') ? [status("warn", "A name cannot be blank or the same as another.")] : []),
      );
    }

    /** Draws `owner` on the fine grid and wires its Zones and Dividers to edits of it. */
    function drawEditor(owner: Layout): void {
      const view = fine(owner);
      let selected: Zone | undefined;
      surface.style.setProperty("--columns", String(view.columns));
      surface.style.setProperty("--rows", String(view.rows));

      const place = <T extends HTMLElement>(element: T, rowStart: number, columnStart: number, rowEnd: number, columnEnd: number): T => {
        element.style.gridArea = `${rowStart + 1} / ${columnStart + 1} / ${rowEnd + 1} / ${columnEnd + 1}`;
        return element;
      };
      const cells = Array.from({ length: view.columns * view.rows }, (_, index) => {
        const [r, c] = [Math.floor(index / view.columns), index % view.columns];
        return place(h("div", { className: "cell" }), r, c, r + 1, c + 1);
      });
      const fields = view.zones.map((zone) => {
        const field = nameField(
          zone.name,
          () => names(owner),
          (name, previous) => {
            edit((c) => renameZone(c, owner.name, previous, name));
            if (picked === previous) picked = name;
          },
        );
        field.ariaLabel = "Zone name";
        const element = place(h("div", { className: "zone" }, field), zone.row, zone.column, zone.row + zone.height, zone.column + zone.width);
        element.addEventListener("focusin", () => select(field.defaultValue));
        element.onclick = () => field.focus();
        return field;
      });
      const lines = dividers(view);
      const handles = lines.map((line) => {
        const [lowest, highest] = range(view, line);
        const across = line.axis === "x";
        const handle = h("div", { className: `divider ${line.axis}`, tabIndex: 0, role: "separator" });
        handle.ariaLabel = `Divider at ${across ? "column" : "row"} ${line.at}`;
        handle.ariaOrientation = across ? "vertical" : "horizontal";
        handle.ariaValueNow = String(line.at);
        handle.ariaValueMin = String(lowest);
        handle.ariaValueMax = String(highest);
        handle.dataset.key = key(line);
        return across ? place(handle, line.from, line.at, line.to, line.at + 1) : place(handle, line.at, line.from, line.at + 1, line.to);
      });
      surface.replaceChildren(...cells, ...fields.map((field) => field.parentElement!), ...handles);

      const tools = operations.map(([label, make], index) => {
        const control = button(label, () => {
          const zones = selected && make(view, selected);
          if (!zones) return;
          commit(zones);
          draw(`tool:${index}`);
        });
        control.dataset.key = `tool:${index}`;
        return { make, control };
      });
      toolBar.replaceChildren(...tools.map(({ control }) => control));

      function select(name: string): void {
        picked = name;
        selected = view.zones.find((zone) => zone.name === name);
        for (const field of fields) field.parentElement!.classList.toggle("picked", field.defaultValue === name);
        for (const { make, control } of tools) control.disabled = !(selected && make(view, selected));
      }

      function commit(zones: Zone[]): void {
        edit(() => Object.assign(owner, { columns: view.columns, rows: view.rows, zones }));
      }

      /** Moves `line` towards `to`, redrawing under the pointer or key that moved it. */
      function slide(line: Divider, to: number): Divider {
        const moved = move(view, line, to);
        if (moved.divider.at !== line.at) {
          commit(moved.zones);
          draw(key(moved.divider));
        }
        return moved.divider;
      }

      const dividerOf = (target: EventTarget | null) =>
        lines.find((line) => target instanceof HTMLElement && target.dataset.key === key(line));
      const steps: Record<string, number> = { ArrowLeft: -1, ArrowUp: -1, ArrowRight: 1, ArrowDown: 1 };
      surface.onkeydown = (event) => {
        const line = dividerOf(event.target);
        const step = steps[event.key];
        if (!line || !step) return;
        event.preventDefault();
        slide(line, line.at + step);
      };
      surface.onpointerdown = (event) => {
        dragging = dividerOf(event.target);
        if (dragging) surface.setPointerCapture(event.pointerId);
      };
      surface.onpointermove = (event) => {
        if (!dragging) return;
        const box = surface.getBoundingClientRect();
        dragging = slide(
          dragging,
          dragging.axis === "x"
            ? Math.round(((event.clientX - box.left) / box.width) * view.columns)
            : Math.round(((event.clientY - box.top) / box.height) * view.rows),
        );
      };
      surface.onlostpointercapture = () => (dragging = undefined);

      select(view.zones.some((zone) => zone.name === picked) ? picked : (view.zones[0]?.name ?? ""));
    }

    /** Redraws everything the config shows, then returns focus to the control with `focus` as its key. */
    function draw(focus?: string): void {
      const target = layout();
      chosen = target?.name ?? "";
      drawRows();
      editor.hidden = !target;
      if (target) drawEditor(target);
      announce();
      // A control that is now disabled cannot keep the focus, so the selected Zone takes it.
      const kept = document.querySelector<HTMLButtonElement>(`[data-key="${focus}"]`);
      if (focus) (kept && !kept.disabled ? kept : document.querySelector<HTMLElement>(".picked input"))?.focus();
    }

    draw();
    return [notice, panel("Layouts", rows), editor];
  },
};
