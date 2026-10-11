/** The config, in the shape the core crate reads and writes it. */
export interface Config {
  version: number;
  gaming: Gaming;
  monitor: MonitorName[];
  layout: Layout[];
  /** In priority order: the first Rule that matches a window claims it. */
  rule: Rule[];
}

export interface Gaming {
  enabled: boolean;
  detect_fullscreen: boolean;
  games: string[];
}

export interface MonitorName {
  id: string;
  name: string;
}

export interface Layout {
  name: string;
  columns: number;
  rows: number;
  zones: Zone[];
}

export interface Zone {
  name: string;
  column: number;
  row: number;
  width: number;
  height: number;
}

export interface Rule {
  exe: string;
  class: string | null;
  title_contains: string | null;
  /** The Monitor's id. */
  monitor: string;
  layout: string;
  zone: string;
  reapply_after_ms: number;
}
