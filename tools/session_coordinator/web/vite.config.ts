import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

const consoleRoot = fileURLToPath(new URL(".", import.meta.url));
const hubVisualFiles = [
  "../../../zircon_hub/web/src/theme/tokens.ts",
  "../../../zircon_hub/web/src/theme/muiTheme.ts",
  "../../../zircon_hub/web/src/components/data/HubPanel.tsx",
  "../../../zircon_hub/web/src/components/inputs/HubButton.tsx",
].map((path) => fileURLToPath(new URL(path, import.meta.url)));

export default defineConfig({
  base: "./",
  plugins: [react()],
  resolve: {
    dedupe: ["react", "react-dom", "@emotion/react", "@emotion/styled", "@mui/material"],
  },
  server: {
    host: "127.0.0.1",
    port: 4317,
    strictPort: true,
    fs: { allow: [consoleRoot, ...hubVisualFiles] },
  },
  build: {
    outDir: "dist",
    emptyOutDir: true,
    sourcemap: false,
    assetsDir: "assets",
  },
});
