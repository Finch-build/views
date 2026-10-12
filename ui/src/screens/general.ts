import mark from "../../design/brand/finch-mark.png";
import wordmark from "../../design/brand/finch-wordmark.png";
import { autostart, configPath, features, installUpdate, setAutostart, updateAvailable, version } from "../commands";
import { h } from "../dom";
import type { Screen } from "../screens";
import { dirty } from "../store";
import { button, panel, row, status, toggle } from "../ui";

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
      update(),
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

/** Offers the newest release. Absent while this is it, and where the platform has no updates. */
function update(): HTMLElement {
  const body = h("div");
  const shown = panel("Update", body);
  shown.hidden = true;
  const show = (...lines: HTMLElement[]) => {
    body.replaceChildren(...lines);
    shown.hidden = false;
  };

  async function check(): Promise<void> {
    try {
      const found = await updateAvailable();
      if (!found) return;
      const offer = row(
        "New version",
        `views ${found}`,
        button("Update", () => {
          // Installing restarts the app, and a draft would not survive that.
          if (dirty()) return show(offer, status("warn", "Save or revert your changes first."));
          show(status("info", `Installing views ${found}. This window will close and open again.`));
          installUpdate().catch((reason) => show(offer, status("bad", String(reason))));
        }),
      );
      show(offer);
    } catch (reason) {
      show(status("none", String(reason)));
    }
  }

  void features().then(({ updates }) => (updates ? check() : undefined));
  return shown;
}
