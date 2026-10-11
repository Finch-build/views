// The primitives every screen is built from. Each returns a plain element.

import { h } from "./dom";

/** What a status line's marker says. Each signal has one colour. */
export type Signal = "none" | "good" | "info" | "warn" | "bad";

/** A framed group with its title set into the frame. */
export function panel(title: string, ...children: (Node | string)[]): HTMLElement {
  return h("fieldset", { className: "panel" }, h("legend", {}, title), ...children);
}

/** A line of a list: what it is, what it holds, and any actions on it. */
export function row(label: string, content: Node | string, ...actions: Node[]): HTMLElement {
  return h("div", { className: "row" }, h("span", {}, label), content, ...actions);
}

/** A row whose label names the control it holds. */
export function field(label: string, control: HTMLElement): HTMLElement {
  return h("label", { className: "row" }, h("span", {}, label), control);
}

/** A `primary` button is the action its view exists for. A view has at most one. */
export function button(label: string, onPress: () => void, primary = false): HTMLButtonElement {
  return h("button", { className: primary ? "button primary" : "button", type: "button", onclick: onPress }, label);
}

export function textField(value: string, onInput: (value: string) => void): HTMLInputElement {
  const input = h("input", { className: "text", type: "text", value, spellcheck: false });
  input.oninput = () => onInput(input.value);
  return input;
}

/** A choice among `options`, each a value and the label shown for it. */
export function select(
  options: readonly (readonly [value: string, label: string])[],
  value: string,
  onChange: (value: string) => void,
): HTMLElement {
  const control = h("select", {}, ...options.map(([value, label]) => h("option", { value }, label)));
  control.value = value;
  control.onchange = () => onChange(control.value);
  return h("span", { className: "select" }, control);
}

export function toggle(label: string, on: boolean, onChange: (on: boolean) => void): HTMLElement {
  const input = h("input", { type: "checkbox", role: "switch", checked: on });
  input.onchange = () => onChange(input.checked);
  return h("label", { className: "toggle" }, input, label);
}

/** One line of state: a marker in the signal's colour, and text that says the same. */
export function status(signal: Signal, text: string): HTMLElement {
  return h("p", { className: `status ${signal}` }, text);
}
