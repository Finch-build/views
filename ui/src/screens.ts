import { general } from "./screens/general";
import { layouts } from "./screens/layouts";
import { monitors } from "./screens/monitors";

export interface Screen {
  /** As it reads in the navigation. */
  name: string;
  /** Builds the screen from the store's config. Called when the screen is shown and when the config is replaced. */
  render(): Node[];
}

/** Every screen, in navigation order. */
export const screens: readonly Screen[] = [general, monitors, layouts];
