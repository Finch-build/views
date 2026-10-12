import assert from "node:assert/strict";
import test from "node:test";
import { listing, matches } from "./apps.ts";

const running = (exe, title = "") => ({ exe, class: "", title, icon: null });
const installed = (name, exe) => ({ name, exe, icon: null });

test("the open apps come first, and the rest follow by name", () => {
  const notes = running("Notes.exe", "Untitled");
  const entries = listing([notes, running("tool.exe", "A tool")], [installed("Zed", "zed.exe"), installed("Blender", "blender.exe"), installed("Notes", "notes.exe")]);
  assert.deepEqual(
    entries.map((entry) => [entry.name, entry.exe]),
    [
      ["A tool", "tool.exe"],
      ["Notes", "notes.exe"],
      ["Blender", "blender.exe"],
      ["Zed", "zed.exe"],
    ],
  );
  assert.equal(entries[1].open, notes);
  assert.equal(entries[2].open, undefined);
});

test("a search finds an app by any words of its name or executable, whatever their case", () => {
  const entry = { name: "Red Dead Redemption 2", exe: "RDR2.exe", icon: null };
  assert.ok(matches(entry, ""));
  assert.ok(matches(entry, "red  DEAD"));
  assert.ok(matches(entry, "rdr2"));
  assert.ok(matches(entry, "dead rdr"));
  assert.ok(!matches(entry, "red alert"));
});
