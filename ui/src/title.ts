import { mount, type Frame, type Piece } from "ascii.rest";
import * as bigText from "ascii.rest/pieces/big-text";
import { h } from "./dom";

/** Draws `text` in block letters into `heading`, which must be in the document: the piece reads its ink from there. */
export function drawTitle(heading: HTMLElement, text: string): void {
  const art = h("span", { role: "img", ariaLabel: text });
  heading.append(art);
  mount(art, cropped(text));
}

/** big-text centres its letters in a field wider than they are. This is the same piece cut down to the letters, so they can sit on a margin. */
function cropped(text: string): Piece {
  const frame = bigText.default({ text });
  const lines = frame(0).split("\n");
  const inked = lines.filter((line) => line.trim());
  const top = lines.findIndex((line) => line.trim());
  const left = Math.min(...inked.map((line) => line.search(/\S/)));
  const right = Math.max(...inked.map((line) => line.trimEnd().length));
  // Solid blocks on either ground: the shaded ones big-text draws on a dark page alias into colour.
  const crop: Frame = (t) =>
    frame(t, { paper: true })
      .split("\n")
      .slice(top, top + inked.length)
      .map((line) => line.slice(left, right))
      .join("\n");
  return { meta: bigText.meta, default: () => crop };
}
