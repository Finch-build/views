import { defineConfig } from "vite";

export default defineConfig({
  clearScreen: false,
  server: { port: 1420, strictPort: true },
  build: {
    // The CSP allows no data: URLs, so even the smallest font stays a file.
    assetsInlineLimit: 0,
    modulePreload: { polyfill: false },
  },
});
