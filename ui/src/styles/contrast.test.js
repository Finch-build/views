import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

const TEXT = 4.5;
const COMPONENT = 3;

// Every foreground the UI sets on paper, and paper on ink for inverted controls.
const PAIRS = [
  ["ink", "paper", TEXT],
  ["paper", "ink", TEXT],
  ["soft", "paper", TEXT],
  ["moss", "paper", COMPONENT],
  ["slate", "paper", COMPONENT],
  ["ochre", "paper", COMPONENT],
  ["clay", "paper", COMPONENT],
];

const css = readFileSync(new URL("tokens.css", import.meta.url), "utf8");
const [light, dark] = css.split("@media (prefers-color-scheme: dark)").map(colours);

function colours(block) {
  return Object.fromEntries([...block.matchAll(/--([a-z]+): (#[0-9a-f]{6});/g)].map(([, name, hex]) => [name, hex]));
}

function luminance(hex) {
  const [r, g, b] = [1, 3, 5]
    .map((at) => parseInt(hex.slice(at, at + 2), 16) / 255)
    .map((channel) => (channel <= 0.04045 ? channel / 12.92 : ((channel + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * g + 0.0722 * b;
}

function contrast(a, b) {
  const [lighter, darker] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (lighter + 0.05) / (darker + 0.05);
}

for (const [scheme, palette] of Object.entries({ light, dark })) {
  test(`every ${scheme} pair the UI uses meets WCAG AA`, () => {
    for (const [fore, back, floor] of PAIRS) {
      const ratio = contrast(palette[fore], palette[back]);
      assert.ok(ratio >= floor, `${fore} on ${back} is ${ratio.toFixed(2)}:1, under ${floor}:1`);
    }
  });
}
