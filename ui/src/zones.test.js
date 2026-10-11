import assert from "node:assert/strict";
import test from "node:test";
import { dividers, fine, fromPreset, merge, move, presets, range, split } from "./zones.ts";

const zone = (name, column, row, width, height) => ({ name, column, row, width, height });

/** How many Zones cover each cell, row by row. Tiling is every cell covered once. */
function coverage({ columns, rows, zones }) {
  const covered = Array.from({ length: rows }, () => Array(columns).fill(0));
  for (const z of zones) {
    for (let row = z.row; row < z.row + z.height; row++) {
      for (let column = z.column; column < z.column + z.width; column++) covered[row][column] += 1;
    }
  }
  return covered.flat();
}

const tiles = (layout) => coverage(layout).every((count) => count === 1);

const layout = (columns, rows, zones) => ({ name: "test", columns, rows, zones });

// A wide centre with two sides, on the fine grid.
const sides = layout(12, 6, [zone("left", 0, 0, 3, 6), zone("centre", 3, 0, 6, 6), zone("right", 9, 0, 3, 6)]);

for (const preset of presets) {
  test(`the ${preset.label} preset tiles its grid with uniquely named Zones`, () => {
    const made = layout(preset.columns, preset.rows, fromPreset(preset).zones);
    assert.ok(tiles(made));
    assert.equal(new Set(made.zones.map((z) => z.name)).size, made.zones.length);
  });

  test(`the ${preset.label} preset keeps tiling on the fine grid`, () => {
    const finer = fine({ name: "test", ...fromPreset(preset) });
    assert.deepEqual([finer.columns, finer.rows], [12, 6]);
    assert.ok(tiles(finer));
  });
}

test("a grid that does not divide the fine grid is shown as it is", () => {
  const odd = layout(5, 1, [zone("all", 0, 0, 5, 1)]);
  assert.deepEqual([fine(odd).columns, fine(odd).rows], [5, 6]);
});

test("thirds have two dividers, one on each line between neighbours", () => {
  const thirds = fine({ name: "t", ...fromPreset(presets[1]) });
  assert.deepEqual(
    dividers(thirds).map(({ axis, at, from, to }) => [axis, at, from, to]),
    [["x", 4, 0, 6], ["x", 8, 0, 6]],
  );
});

test("moving a divider resizes both neighbours and keeps the Layout tiling", () => {
  const [first] = dividers(sides);
  const moved = move(sides, first, 5);
  assert.deepEqual(moved.divider, { ...first, at: 5 });
  assert.deepEqual(moved.zones.map((z) => [z.column, z.width]), [[0, 5], [5, 4], [9, 3]]);
  assert.ok(tiles({ ...sides, zones: moved.zones }));
});

test("a divider stops one cell short of its neighbours", () => {
  const [first] = dividers(sides);
  assert.deepEqual(range(sides, first), [1, 8]);
  assert.equal(move(sides, first, 99).divider.at, 8);
  assert.equal(move(sides, first, -4).divider.at, 1);
});

test("a divider shared by several Zones moves all of them", () => {
  // One tall Zone on the left, two stacked on the right.
  const stacked = layout(12, 6, [zone("a", 0, 0, 6, 6), zone("b", 6, 0, 6, 3), zone("c", 6, 3, 6, 3)]);
  const vertical = dividers(stacked).filter((d) => d.axis === "x");
  assert.equal(vertical.length, 1);
  const moved = move(stacked, vertical[0], 8);
  assert.deepEqual(moved.zones.map((z) => [z.column, z.width]), [[0, 8], [8, 4], [8, 4]]);
  assert.ok(tiles({ ...stacked, zones: moved.zones }));
});

test("dividers that only meet at a point move independently", () => {
  const brick = layout(12, 6, [
    zone("a", 0, 0, 6, 3),
    zone("b", 6, 0, 6, 3),
    zone("c", 0, 3, 4, 3),
    zone("d", 4, 3, 8, 3),
  ]);
  const vertical = dividers(brick).filter((d) => d.axis === "x");
  assert.deepEqual(vertical.map((d) => [d.at, d.from, d.to]), [[4, 3, 6], [6, 0, 3]]);
  assert.ok(tiles({ ...brick, zones: move(brick, vertical[1], 7).zones }));
});

test("splitting cuts a Zone in two cells and names the second half", () => {
  const halves = layout(12, 6, [zone("all", 0, 0, 12, 6)]);
  const cut = split(halves, halves.zones[0], "x", "all-2");
  assert.deepEqual(cut.map((z) => [z.name, z.column, z.width]), [["all", 0, 6], ["all-2", 6, 6]]);
  assert.ok(tiles({ ...halves, zones: cut }));
  assert.equal(split(halves, zone("one", 0, 0, 1, 1), "x", "x"), undefined);
});

test("merging joins a Zone to the neighbour that lines up with it, and only that", () => {
  const [left, centre] = sides.zones;
  const joined = merge(sides, left, "x");
  assert.deepEqual(joined.map((z) => [z.name, z.column, z.width]), [["left", 0, 9], ["right", 9, 3]]);
  assert.ok(tiles({ ...sides, zones: joined }));
  assert.equal(merge(sides, centre, "y"), undefined);
  const stacked = layout(12, 6, [zone("a", 0, 0, 6, 6), zone("b", 6, 0, 6, 3), zone("c", 6, 3, 6, 3)]);
  assert.equal(merge(stacked, stacked.zones[0], "x"), undefined);
});
