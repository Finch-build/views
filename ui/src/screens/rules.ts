import { monitors as attached, type App, type Monitor } from "../commands";
import type { Rule } from "../config";
import { h } from "../dom";
import { dangling } from "../names";
import { appPicker } from "../picker";
import { fresh, monitorName, optional, retarget, shadowed, shift, skipped } from "../rules";
import type { Screen } from "../screens";
import { config, edit } from "../store";
import { button, field, panel, row, select, status, textField, type Signal } from "../ui";

/** The Rule being edited, and the app it was made from. They outlast a redraw of the screen. */
let editing: Rule | undefined;
let origin: { rule: Rule; app: App } | undefined;

/** The most a delay can be: core reads it as an unsigned 32-bit number. */
const MAX_DELAY = 2 ** 32 - 1;

/** Gives the focus to the first of the controls with these keys that can take it. */
const focus = (...keys: string[]) =>
  keys.map((key) => document.querySelector<HTMLElement>(`[data-key="${key}"]:enabled`)).find(Boolean)?.focus();

/** `names` as options, with `current` among them even where it is not a name, marked as missing. */
function choices(names: readonly string[], current: string): [string, string][] {
  const found: [string, string][] = names.map((name) => [name, name]);
  return names.includes(current) ? found : [...found, [current, `${current} (missing)`]];
}

export const rules: Screen = {
  name: "Rules",
  render() {
    let present: Monitor[] = [];
    const list = h("div");
    const picker = appPicker("Add Rule", add, () => focus("add"));
    const form = h("div");
    const editor = panel("Rule", form);
    editor.hidden = true;

    void attached().then(
      (found) => {
        present = found;
        drawList();
        drawForm();
      },
      (reason) => list.replaceChildren(status("bad", String(reason))),
    );

    function drawList(): void {
      const lost = new Set(dangling(config()));
      const all = config().rule;
      const why = config().layout.length
        ? present.length
          ? ""
          : "No Monitors were found."
        : "A Rule sends a window to a Zone, and there are no Layouts yet. Create one on the Layouts screen.";

      list.replaceChildren(
        ...all.map((rule, index) => {
          const conditions = [rule.class && `class ${rule.class}`, rule.title_contains && `title contains “${rule.title_contains}”`];
          const notes: [Signal, string][] = [];
          if (skipped(rule, present)) notes.push(["warn", "Skipped: its Monitor is not attached."]);
          if (lost.has(rule)) {
            const known = config().layout.some((layout) => layout.name === rule.layout);
            notes.push(["bad", known ? `Layout “${rule.layout}” has no Zone “${rule.zone}”.` : `There is no Layout “${rule.layout}”.`]);
          }
          if (shadowed(all, index, present)) notes.push(["info", "Never matches: an earlier Rule takes every window of this exe."]);

          const act = (label: string, key: string, onPress: () => void, disabled = false) => {
            const control = button(label, onPress);
            control.ariaLabel = `${label} ${rule.exe}`;
            control.dataset.key = `${key}:${index}`;
            control.disabled = disabled;
            return control;
          };
          const toggle = act("Edit", "edit", () => {
            editing = editing === rule ? undefined : rule;
            drawList();
            drawForm();
            focus(editing ? "exe" : `edit:${index}`);
          });
          toggle.ariaPressed = String(rule === editing);

          return h(
            "div",
            { className: "row rule" },
            h("span", {}, String(index + 1)),
            h(
              "div",
              {},
              h("p", {}, rule.exe),
              h("p", { className: "soft" }, conditions.filter(Boolean).join(" · ") || "Any window"),
              h("p", {}, `→ ${monitorName(rule.monitor, config(), present)} · ${rule.layout} · ${rule.zone}`),
              ...notes.map((note) => status(...note)),
            ),
            h(
              "div",
              { className: "actions" },
              act("Up", "up", () => move(index, -1), index === 0),
              act("Down", "down", () => move(index, 1), index === all.length - 1),
              toggle,
              act("Remove", "remove", () => {
                edit((c) => c.rule.splice(index, 1));
                if (editing === rule) editing = undefined;
                drawList();
                drawForm();
                focus(`remove:${index}`, `remove:${index - 1}`, "add");
              }),
            ),
          );
        }),
      );
      if (!all.length) list.append(status("none", "No Rules."));
      const add = button("Add Rule", picker.open, true);
      add.dataset.key = "add";
      add.disabled = why !== "";
      list.append(row("", h("div", { className: "actions" }, add)));
      if (why) list.append(status("info", why));
    }

    /** Moves the Rule at `index`, and keeps the focus on the control that moved it. */
    function move(index: number, by: -1 | 1): void {
      edit((c) => shift(c.rule, index, by));
      drawList();
      const [moved, other] = by < 0 ? ["up", "down"] : ["down", "up"];
      focus(`${moved}:${index + by}`, `${other}:${index + by}`);
    }

    function drawForm(): void {
      const rule = editing && config().rule.includes(editing) ? editing : undefined;
      editor.hidden = !rule;
      if (!rule) return;
      const seen = origin?.rule === rule ? origin.app : undefined;
      const change = (apply: (rule: Rule) => void) => {
        edit(() => apply(rule));
        drawList();
      };

      /** A text field for an optional condition. Where the Rule came from a window, an empty field offers that window's value. */
      const condition = (label: string, value: string | null, offer: string, store: (rule: Rule, value: string | null) => void) => {
        const input = textField(value ?? "", (text) => change(() => store(rule, optional(text))));
        const offered = value === null && offer !== "";
        input.ariaLabel = label;
        input.placeholder = offered ? offer : "Any";
        input.dataset.key = label;
        const use = button("Use", () => {
          change(() => store(rule, offer));
          drawForm();
          focus(label);
        });
        use.ariaLabel = `Use the window's ${label.toLowerCase()}`;
        return row(label, h("div", { className: "pair" }, input, ...(offered ? [use] : [])));
      };

      const exe = textField(rule.exe, (value) => change((r) => (r.exe = value)));
      exe.dataset.key = "exe";
      const monitors = present.map((found) => found.id);
      if (!monitors.includes(rule.monitor)) monitors.push(rule.monitor);
      const monitor = select(
        monitors.map((id): [string, string] => [id, monitorName(id, config(), present)]),
        rule.monitor,
        (id) => change((r) => (r.monitor = id)),
      );
      monitor.querySelector("select")!.dataset.key = "monitor";
      const layout = select(choices(config().layout.map((found) => found.name), rule.layout), rule.layout, (name) => {
        change((r) => retarget(config(), r, name));
        drawForm();
      });
      const zones = config().layout.find((found) => found.name === rule.layout)?.zones.map((zone) => zone.name) ?? [];
      const zone = select(choices(zones, rule.zone), rule.zone, (name) => change((r) => (r.zone = name)));
      const delay = textField(String(rule.reapply_after_ms), (text) => {
        const valid = /^\d+$/.test(text) && Number(text) <= MAX_DELAY;
        delay.ariaInvalid = valid ? null : "true";
        if (valid) change((r) => (r.reapply_after_ms = Number(text)));
      });
      delay.ariaLabel = "Reapply after";
      delay.onblur = () => {
        delay.value = String(rule.reapply_after_ms);
        delay.ariaInvalid = null;
      };

      form.replaceChildren(
        field("Executable", exe),
        condition("Class", rule.class, seen?.class ?? "", (r, value) => (r.class = value)),
        condition("Title contains", rule.title_contains, seen?.title ?? "", (r, value) => (r.title_contains = value)),
        field("Monitor", monitor),
        field("Layout", layout),
        field("Zone", zone),
        row("Reapply after", h("div", { className: "pair" }, delay, h("span", {}, "ms"))),
      );
    }

    /** Appends a Rule for `exe` and opens it for editing. `from` is the open window it was picked by. */
    function add(exe: string, from?: App): void {
      const rule = fresh(exe, config(), present);
      edit((c) => c.rule.push(rule));
      editing = rule;
      origin = from && { rule, app: from };
      drawList();
      drawForm();
      focus("monitor");
    }

    drawList();
    return [panel("Rules", list), picker.panel, editor];
  },
};
