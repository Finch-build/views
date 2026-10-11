import type { Features } from "./commands";
import { gaming } from "./screens/gaming";
import { general } from "./screens/general";
import { layouts } from "./screens/layouts";
import { monitors } from "./screens/monitors";
import { rules } from "./screens/rules";

export interface Screen {
  /** As it reads in the navigation. */
  name: string;
  /** Builds the screen from the store's config. Called when the screen is shown and when the config is replaced. */
  render(): Node[];
  /** The platform feature the screen is shown for. Without it, the screen is absent. */
  requires?: keyof Features;
}

/** Every screen, in navigation order. */
export const screens: readonly Screen[] = [general, monitors, layouts, rules, gaming];
