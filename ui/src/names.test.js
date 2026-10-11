import assert from "node:assert/strict";
import test from "node:test";
import { dangling, isFree, renameLayout, renameZone, unique } from "./names.ts";

const rule = (layout, zone) => ({ exe: "a.exe", class: null, title_contains: null, monitor: "M", layout, zone, reapply_after_ms: 0 });
const zone = (name) => ({ name, column: 0, row: 0, width: 1, height: 1 });

function config() {
  const thirds = { name: "thirds", columns: 3, rows: 1, zones: [zone("left"), zone("right")] };
  return { layout: [thirds], rule: [rule("thirds", "left"), rule("thirds", "right"), rule("halves", "left")] };
}

test("a name is free unless it is blank or taken", () => {
  assert.ok(isFree("a", ["b"]));
  assert.ok(!isFree("  ", []));
  assert.ok(!isFree("b", ["b"]));
});

test("unique numbers a name from 2 until it is not taken", () => {
  assert.equal(unique("left", ["right"]), "left");
  assert.equal(unique("left", ["left", "left-2"]), "left-3");
});

test("renaming a Layout renames it in the Rules that use it", () => {
  const c = config();
  renameLayout(c, "thirds", "wide");
  assert.deepEqual(c.rule.map((r) => r.layout), ["wide", "wide", "halves"]);
  assert.equal(c.layout[0].name, "wide");
});

test("renaming a Zone renames it only in the Rules that use that Zone of that Layout", () => {
  const c = config();
  renameZone(c, "thirds", "left", "first");
  assert.deepEqual(c.rule.map((r) => r.zone), ["first", "right", "left"]);
  assert.equal(c.layout[0].zones[0].name, "first");
});

test("a Rule is dangling when its Layout or Zone is gone", () => {
  const c = config();
  assert.deepEqual(dangling(c).map((r) => r.layout), ["halves"]);
  c.layout[0].zones.pop();
  assert.equal(dangling(c).length, 2);
});
