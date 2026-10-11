import mark from "../../design/brand/finch-mark.png";
import wordmark from "../../design/brand/finch-wordmark.png";
import { autostart, configPath, features, setAutostart, version } from "../commands";
import { h } from "../dom";
import type { Screen } from "../screens";
import { panel, row, status, toggle } from "../ui";

export const general: Screen = {
  name: "General",
  render() {
    const path = h("span", { className: "path" });
    const release = h("span");
    void configPath().then((found) => (path.textContent = found));
    void version().then((found) => (release.textContent = found));
    return [
      panel("Config file", row("Location", path)),
      startup(),
      panel(
        "About",
        h(
          "div",
          { className: "about" },
          h("img", { className: "mark", src: mark, alt: "" }),
          h(
            "div",
            {},
            h("p", {}, "views ", release),
            h("p", { className: "soft" }, "Moves application windows to a chosen Monitor and Zone when they open."),
            h("p", { className: "soft" }, "finch.dev"),
          ),
          h("img", { className: "wordmark", src: wordmark, alt: "Finch.DEV" }),
        ),
      ),
    ];
  },
};

/** The sign-in toggle. It shows the system's state, not the config's, and changes it at once. Absent where there is no autostart. */
function startup(): HTMLElement {
  const body = h("div");
  const shown = panel("Startup", body);
  shown.hidden = true;

  async function draw(problem = ""): Promise<void> {
    try {
      const on = await autostart();
      body.replaceChildren(
        row("Sign-in", toggle("Start the Engine when I sign in", on, (now) => void change(now))),
        ...(problem ? [status("bad", problem)] : []),
      );
    } catch (reason) {
      body.replaceChildren(status("bad", String(reason)));
    }
    shown.hidden = false;
  }

  async function change(on: boolean): Promise<void> {
    try {
      await setAutostart(on);
    } catch (reason) {
      await draw(String(reason));
      body.querySelector("input")?.focus();
    }
  }

  void features().then(({ autostart: present }) => (present ? draw() : undefined));
  return shown;
}
