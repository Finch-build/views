// The geometry of a Layout's Zones: pure, so that the editor only draws it.
//
// Zones are rectangles of whole cells, and every operation here keeps a Layout that
// tiles its grid tiling it, so a Layout that core accepts is one it still accepts.

import type { Layout, Zone } from "./config";

/** The grid that custom splits are made on. Every preset divides it exactly. */
export const GRID = { columns: 12, rows: 6 } as const;

export type Axis = "x" | "y";

/** The boundary between Zones: the grid line `at` on `axis`, over cells `from` to `to` of the other axis. */
export interface Divider {
  axis: Axis;
  at: number;
  from: number;
  to: number;
}

/** A grid of equal one-cell Zones, named in reading order. */
export interface Preset {
  label: string;
  columns: number;
  rows: number;
  names: readonly string[];
}

export const presets: readonly Preset[] = [
  { label: "Halves", columns: 2, rows: 1, names: ["left", "right"] },
  { label: "Thirds", columns: 3, rows: 1, names: ["left", "middle", "right"] },
  { label: "Quarters", columns: 4, rows: 1, names: ["far-left", "left", "right", "far-right"] },
  { label: "2x2", columns: 2, rows: 2, names: ["top-left", "top-right", "bottom-left", "bottom-right"] },
];

/** The grid and Zones that `preset` gives a Layout. */
export function fromPreset({ columns, rows, names }: Preset): Pick<Layout, "columns" | "rows" | "zones"> {
  const zones = names.map((name, index) => ({
    name,
    column: index % columns,
    row: Math.floor(index / columns),
    width: 1,
    height: 1,
  }));
  return { columns, rows, zones };
}

/** `layout` on the finest grid it divides exactly, which is how the editor shows it: the same Zones, with room to move their edges. */
export function fine(layout: Layout): Layout {
  const factor = (cells: number, finest: number) => (finest % cells === 0 ? finest / cells : 1);
  const [x, y] = [factor(layout.columns, GRID.columns), factor(layout.rows, GRID.rows)];
  return {
    ...layout,
    columns: layout.columns * x,
    rows: layout.rows * y,
    zones: layout.zones.map((zone) => ({
      ...zone,
      column: zone.column * x,
      width: zone.width * x,
      row: zone.row * y,
      height: zone.height * y,
    })),
  };
}

/** Every Divider of `layout`. */
export function dividers(layout: Layout): Divider[] {
  return (["x", "y"] as const).flatMap((axis) => {
    const across = crossing(axis);
    const lines = Array.from({ length: cells(layout, axis) - 1 }, (_, index) => index + 1);
    return lines.flatMap((at) => {
      const touching = layout.zones
        .filter((zone) => start(zone, axis) === at || end(zone, axis) === at)
        .sort((a, b) => start(a, across) - start(b, across));
      // Zones whose edges overlap along the line are moved together; ones that only meet are not.
      const groups: Zone[][] = [];
      for (const zone of touching) {
        const group = groups.at(-1);
        if (group && start(zone, across) < Math.max(...group.map((member) => end(member, across)))) group.push(zone);
        else groups.push([zone]);
      }
      return groups
        .filter((group) => group.some((zone) => start(zone, axis) === at) && group.some((zone) => end(zone, axis) === at))
        .map((group) => ({
          axis,
          at,
          from: Math.min(...group.map((zone) => start(zone, across))),
          to: Math.max(...group.map((zone) => end(zone, across))),
        }));
    });
  });
}

/** The nearest positions, either side of `divider`, that leave every Zone it touches a cell wide. */
export function range(layout: Layout, divider: Divider): [lowest: number, highest: number] {
  const { before, after } = touched(layout, divider);
  return [
    Math.max(...before.map((zone) => start(zone, divider.axis) + 1)),
    Math.min(...after.map((zone) => end(zone, divider.axis) - 1)),
  ];
}

/** Moves `divider` to the grid line `to`, or the nearest it can reach, taking the edges of the Zones it touches with it. */
export function move(layout: Layout, divider: Divider, to: number): { zones: Zone[]; divider: Divider } {
  const { before, after } = touched(layout, divider);
  const [lowest, highest] = range(layout, divider);
  const at = Math.min(Math.max(to, lowest), highest);
  const zones = layout.zones.map((zone) => {
    if (before.includes(zone)) return resized(zone, divider.axis, start(zone, divider.axis), at);
    if (after.includes(zone)) return resized(zone, divider.axis, at, end(zone, divider.axis));
    return zone;
  });
  return { zones, divider: { ...divider, at } };
}

/** `zone` cut in two along `axis`, the second half named `name`; undefined if it is a single cell on that axis. */
export function split(layout: Layout, zone: Zone, axis: Axis, name: string): Zone[] | undefined {
  const length = size(zone, axis);
  if (length < 2) return undefined;
  const middle = start(zone, axis) + Math.ceil(length / 2);
  const second = { ...resized(zone, axis, middle, end(zone, axis)), name };
  return layout.zones.flatMap((member) =>
    member === zone ? [resized(zone, axis, start(zone, axis), middle), second] : [member],
  );
}

/** `zone` joined to the Zone after it along `axis`, which is removed; undefined if no Zone lines up exactly. */
export function merge(layout: Layout, zone: Zone, axis: Axis): Zone[] | undefined {
  const across = crossing(axis);
  const next = layout.zones.find(
    (member) =>
      start(member, axis) === end(zone, axis) &&
      start(member, across) === start(zone, across) &&
      size(member, across) === size(zone, across),
  );
  if (!next) return undefined;
  return layout.zones
    .filter((member) => member !== next)
    .map((member) => (member === zone ? resized(zone, axis, start(zone, axis), end(next, axis)) : member));
}

const along = { x: ["column", "width"], y: ["row", "height"] } as const;

const crossing = (axis: Axis): Axis => (axis === "x" ? "y" : "x");
const start = (zone: Zone, axis: Axis) => zone[along[axis][0]];
const size = (zone: Zone, axis: Axis) => zone[along[axis][1]];
const end = (zone: Zone, axis: Axis) => start(zone, axis) + size(zone, axis);
const cells = (layout: Layout, axis: Axis) => (axis === "x" ? layout.columns : layout.rows);

function resized(zone: Zone, axis: Axis, from: number, to: number): Zone {
  const [position, length] = along[axis];
  return { ...zone, [position]: from, [length]: to - from };
}

/** The Zones whose edges lie on `divider`: those ending at it and those starting at it. */
function touched(layout: Layout, { axis, at, from, to }: Divider): { before: Zone[]; after: Zone[] } {
  const across = crossing(axis);
  const within = layout.zones.filter((zone) => start(zone, across) < to && end(zone, across) > from);
  return {
    before: within.filter((zone) => end(zone, axis) === at),
    after: within.filter((zone) => start(zone, axis) === at),
  };
}
