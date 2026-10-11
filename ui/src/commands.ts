// The Rust side, one function a command. A rejection carries a message fit to show.

import { invoke } from "@tauri-apps/api/core";
import type { Config } from "./config";

export interface Rect {
  left: number;
  top: number;
  right: number;
  bottom: number;
}

export interface Monitor {
  /** Survives restarts and reconnects. */
  id: string;
  /** What the display calls itself. */
  name: string;
  bounds: Rect;
  /** The bounds minus whatever the desktop reserves, such as a taskbar. */
  work_area: Rect;
}

export const configPath = () => invoke<string>("config_path");

export const version = () => invoke<string>("version");

/** The config on disk, or an empty one if there is no file yet. */
export const loadConfig = () => invoke<Config>("load_config");

/** Resolves to whether an Engine was running to reload what was saved. */
export const saveConfig = (config: Config) => invoke<boolean>("save_config", { config });

/** The attached Monitors, in pixels of the desktop's own coordinate space. */
export const monitors = () => invoke<Monitor[]>("monitors");
