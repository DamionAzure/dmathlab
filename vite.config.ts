import path from "node:path";
import { defineConfig } from "vite";
import react from "@vitejs/plugin-react";
import tailwindcss from "@tailwindcss/vite";

// Tauri expects a fixed port (see src-tauri/tauri.conf.json -> build.devUrl).
export default defineConfig({
  plugins: [react(), tailwindcss()],
  resolve: {
    alias: {
      "@": path.resolve(__dirname, "./src"),
    },
  },
  // Vite options tailored for Tauri development.
  //
  // 1. prevent vite from obscuring rust errors
  clearScreen: false,
  // 2. Tauri expects a fixed port; fail if that port is not available
  server: { port: 1420, strictPort: true },
  // 3. Relative asset URLs so the Tauri webview (custom protocol) resolves them.
  base: "./",
});
