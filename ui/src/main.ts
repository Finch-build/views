import "./styles/tokens.css";
import "./styles/base.css";
import "./styles/primitives.css";
import "./styles/shell.css";
import "./styles/screens.css";
import { shell } from "./shell";

// Animations take their play state from a token, and the token follows this class.
document.addEventListener("visibilitychange", () =>
  document.documentElement.classList.toggle("hidden", document.hidden),
);

shell();
