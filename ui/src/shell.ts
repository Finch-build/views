// The frame around every screen: title, navigation and the save bar.

import { h } from "./dom";
import { screens, type Screen } from "./screens";
import { dirty, engineHeard, error, load, ready, revert, save, subscribe } from "./store";
import { drawTitle } from "./title";
import { button, panel, status, type Signal } from "./ui";

export function shell(): void {
  let current = screens[0]!;
  const tabs = screens.map((screen) =>
    h("button", { className: "tab", type: "button", onclick: () => show(screen) }, screen.name),
  );
  const main = h("main", { className: "screen" });
  const outcome = h("div", { role: "status" });
  const revertButton = button("Revert", revert);
  const saveButton = button("Save", () => void save(), true);

  function show(screen: Screen): void {
    current = screen;
    tabs.forEach((tab, index) => (tab.ariaCurrent = screens[index] === screen ? "page" : null));
    main.replaceChildren(...(ready() ? screen.render() : unreadable()));
  }

  subscribe((replaced) => {
    if (replaced) show(current);
    outcome.replaceChildren(status(...report()));
    revertButton.disabled = saveButton.disabled = !dirty();
  });

  const heading = h("h1", { className: "title" });
  document.body.append(
    heading,
    h("nav", { className: "tabs" }, ...tabs),
    main,
    h("footer", { className: "bar" }, outcome, revertButton, saveButton),
  );
  drawTitle(heading, "views");
  void load();
}

function report(): [Signal, string] {
  if (!ready()) return ["bad", "The config file could not be read."];
  if (error()) return ["bad", error()];
  if (dirty()) return ["warn", "Unsaved changes"];
  if (engineHeard() === true) return ["good", "Saved. The Engine has reloaded."];
  if (engineHeard() === false) return ["info", "Saved. The Engine is not running."];
  return ["none", "No changes"];
}

/** Stands in for every screen while the file on disk does not parse, so that nothing can overwrite it unread. */
function unreadable(): Node[] {
  return [
    panel(
      "Config file",
      h(
        "div",
        { className: "notice" },
        h("p", { className: "path" }, error()),
        h("p", {}, "Nothing is saved until the file can be read. Correct it, or move it away to start empty, then reload."),
        button("Reload", () => void load()),
      ),
    ),
  ];
}
