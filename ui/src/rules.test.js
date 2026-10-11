import assert from "node:assert/strict";
import test from "node:test";
import { fresh, monitorName, optional, retarget, shadowed, shift, skipped } from "./rules.ts";

const rule = (exe, extra = {}) => ({ exe, class: null, title_contains: null, monitor: "A", layout: "halves", zone: "left", reapply_after_ms: 0, ...extra });
const present = [{ id: "A", name: "Dell" }];
const config = {
  monitor: [{ id: "B", name: "Desk" }],
  layout: [
    { name: "halves", zones: [{ name: "left" }, { name: "right" }] },
    { name: "full", zones: [{ name: "all" }] },
  ],
};

test("blank optional text is absent", () => {
  assert.equal(optional("  "), null);
  assert.equal(optional("Docs"), "Docs");
});

test("a Rule is skipped when its Monitor is not attached", () => {
  assert.ok(!skipped(rule("a.exe"), present));
  assert.ok(skipped(rule("a.exe", { monitor: "B" }), present));
});

test("a Monitor is named by the user, else by itself, else by its id", () => {
  assert.equal(monitorName("B", config, present), "Desk");
  assert.equal(monitorName("A", config, present), "Dell");
  assert.equal(monitorName("C", config, present), "C");
});

test("a Rule is shadowed by an earlier Rule for the same exe with no conditions", () => {
  assert.ok(shadowed([rule("a.exe"), rule("A.EXE")], 1, present));
  assert.ok(!shadowed([rule("a.exe"), rule("A.EXE")], 0, present));
});

test("a Rule is not shadowed by a narrower, other or skipped Rule", () => {
  assert.ok(!shadowed([rule("a.exe", { class: "X" }), rule("a.exe")], 1, present));
  assert.ok(!shadowed([rule("a.exe", { title_contains: "x" }), rule("a.exe")], 1, present));
  assert.ok(!shadowed([rule("b.exe"), rule("a.exe")], 1, present));
  assert.ok(!shadowed([rule("a.exe", { monitor: "B" }), rule("a.exe")], 1, present));
});

test("shifting a Rule swaps it with its neighbour and stops at the ends", () => {
  const rules = [rule("a"), rule("b"), rule("c")];
  shift(rules, 2, -1);
  assert.deepEqual(rules.map((r) => r.exe), ["a", "c", "b"]);
  shift(rules, 0, -1);
  shift(rules, 2, 1);
  assert.deepEqual(rules.map((r) => r.exe), ["a", "c", "b"]);
});

test("retargeting keeps the Zone when the new Layout has it and takes the first otherwise", () => {
  const kept = rule("a", { zone: "right" });
  retarget({ ...config, layout: [config.layout[0], { name: "wide", zones: [{ name: "right" }] }] }, kept, "wide");
  assert.deepEqual([kept.layout, kept.zone], ["wide", "right"]);
  retarget(config, kept, "full");
  assert.deepEqual([kept.layout, kept.zone], ["full", "all"]);
});

test("a fresh Rule goes to the first Monitor, Layout and Zone", () => {
  assert.deepEqual(fresh("a.exe", config, present), rule("a.exe"));
});
