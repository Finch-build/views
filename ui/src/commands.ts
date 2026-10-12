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

/** The small icon of an executable: `width` × `height` pixels, four bytes each, red first. */
export interface Icon {
  width: number;
  height: number;
  rgba: number[];
}

/** An app with a Tileable window, as one of its windows describes it. */
export interface App {
  exe: string;
  class: string;
  title: string;
  /** Absent where the platform has no icons. */
  icon: Icon | null;
}

/** The apps with a Tileable window now, one each. */
export const runningApps = () => invoke<App[]>("running_apps");

/** One executable of an installed app, under the name the system shows for the app. */
export interface Installed {
  name: string;
  exe: string;
  icon: Icon | null;
}

/** The installed apps, by name. Empty where the platform cannot tell. */
export const installedApps = () => invoke<Installed[]>("installed_apps");

/** The file name of an executable the user picks, or null if they cancel. */
export const browse = () => invoke<string | null>("browse");

/** What this platform has: a screen or control for what it lacks is not shown. */
export interface Features {
  gaming: boolean;
  autostart: boolean;
}

export const features = () => invoke<Features>("features");

/** Whether the Engine starts when the user signs in. */
export const autostart = () => invoke<boolean>("autostart");

export const setAutostart = (on: boolean) => invoke<void>("set_autostart", { on });
