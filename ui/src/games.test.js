import assert from "node:assert/strict";
import test from "node:test";
import { canAdd } from "./games.ts";

test("a game can be added unless it is blank or listed, whatever its case", () => {
  assert.ok(canAdd(["eldenring.exe"], "doom.exe"));
  assert.ok(!canAdd([], "  "));
  assert.ok(!canAdd(["EldenRing.exe"], " eldenring.EXE "));
});
