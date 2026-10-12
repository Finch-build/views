import { canAdd, isListed } from "../games";
import { h } from "../dom";
import { appPicker } from "../picker";
import type { Screen } from "../screens";
import { config, edit } from "../store";
import { button, panel, row, status, textField, toggle } from "../ui";

export const gaming: Screen = {
  name: "Gaming mode",
  requires: "gaming",
  render() {
    const list = h("div");
    const notice = h("div", { role: "status" });
    const input = textField("", check);
    input.placeholder = "eldenring.exe";
    input.ariaLabel = "Executable file name";
    const add = button("Add", addGame);
    const find = button("Find", () => picker.open());
    const picker = appPicker(
      "Find a game",
      (exe) => {
        input.value = exe;
        check();
        addGame();
      },
      () => find.focus(),
    );

    function check(): void {
      const listed = isListed(config().gaming.games, input.value);
      input.ariaInvalid = listed ? "true" : null;
      add.disabled = !canAdd(config().gaming.games, input.value);
      notice.replaceChildren(...(listed ? [status("warn", "That game is already listed.")] : []));
    }

    function addGame(): void {
      if (add.disabled) return;
      edit((c) => c.gaming.games.push(input.value.trim()));
      input.value = "";
      draw();
      input.focus();
    }

    function removeGame(index: number): void {
      edit((c) => c.gaming.games.splice(index, 1));
      draw();
      (list.querySelector("button") ?? input).focus();
    }

    function draw(): void {
      const { games } = config().gaming;
      list.replaceChildren(
        ...games.map((game, index) => row("Game", h("span", {}, game), button("Remove", () => removeGame(index)))),
        ...(games.length ? [] : [status("none", "No games are listed.")]),
      );
      check();
    }

    input.onkeydown = (event) => {
      if (event.key === "Enter") addGame();
    };
    draw();
    const settings = config().gaming;
    return [
      panel(
        "Gaming mode",
        h(
          "p",
          { className: "soft" },
          "While a game runs, the Engine watches nothing and applies no Rules. When the game exits, it runs a Sweep. The tray's Auto, On and Off choice outranks these settings.",
        ),
        row("Enabled", toggle("Suspend the Engine while a game runs", settings.enabled, (on) => edit((c) => (c.gaming.enabled = on)))),
        row(
          "Detection",
          toggle("Treat fullscreen Direct3D apps as games", settings.detect_fullscreen, (on) => edit((c) => (c.gaming.detect_fullscreen = on))),
        ),
      ),
      panel("Games", row("Add", input, add, find), notice, list),
      picker.panel,
    ];
  },
};
