import { fileURLToPath, URL } from "node:url";
import react from "@vitejs/plugin-react";
import { defineConfig } from "vite";

// 桌面开发壳与发布构建共用此配置；页面根、资源基址和输出目录须与 Tauri 的入口相合。
const projectRoot = fileURLToPath(new URL(".", import.meta.url));

export default defineConfig({
  root: "web",
  // 发布资源使用相对路径，供桌面壳从本地页面位置加载。
  base: "",
  plugins: [react()],
  clearScreen: false,
  // 开发端口被占用时直接失败，避免桌面壳连接到与前端服务不一致的地址。
  server: {
    port: 1420,
    strictPort: true,
    fs: {
      allow: [projectRoot],
    },
  },
  envPrefix: ["VITE_", "TAURI_"],
  // BUG: [CR-HUBCONFIG-0001] npm run build 调用 Vite 后将前端编译物写入仓内 web/dist，违反当前仅允许 D/E/F 驱动器根 cargo-targets 的产物位置约束；证据：package.json 的 build 脚本、此处输出目录与 tauri.conf.json 的 frontendDist。
  // 前端发布物位于 Web 页面根内，由 Tauri 的资源入口接收。
  build: {
    outDir: "dist",
    emptyOutDir: true,
  },
});
