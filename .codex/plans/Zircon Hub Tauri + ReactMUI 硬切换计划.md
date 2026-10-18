# Zircon Hub Tauri + React/MUI 硬切换计划

## Summary
- 采用“硬切换”：`zircon_hub` 的桌面入口改为 Tauri v2 + React + TypeScript + Material UI，不长期保留 Slint UI 并行入口。
- 保留现有 Rust 后端能力：Hub runtime、配置、项目/源码引擎管理、构建/打包/安装、view-model 数据投影。
- 前端从 `docs/ui-and-layout/hub-web-reference` 的 HTML/CSS 参考迁移为 React/MUI 组件栈，按 MUI 分类自底向上实现：Inputs、Data display、Feedback、Surface、Navigation、Layout。
- 官方参考：Tauri [Create Project](https://v2.tauri.app/start/create-project/)、[Vite config](https://v2.tauri.app/start/frontend/vite/)、[Calling Rust](https://v2.tauri.app/develop/calling-rust/)，MUI [Installation](https://mui.com/material-ui/getting-started/installation/)、[All Components](https://mui.com/material-ui/all-components/)。

## Key Changes
- `zircon_hub` crate：
  - 移除 Slint 构建路径：`slint`、`i-slint-compiler`、`build.rs` Slint 编译逻辑、`slint::include_modules!()` 绑定入口。
  - 加入 Tauri v2 runtime、`tauri.conf.json`、capabilities/permissions、窗口配置、npm/Vite build hooks。
  - 保持 `zircon_hub` 仍是独立 Hub 包，不引入 `zircon_runtime` / `zircon_editor` 生命周期依赖。
- Rust API：
  - 新增 `hub_state()` command 返回完整 `HubViewModel` JSON。
  - 新增 `hub_action(action: HubActionRequest)` command 承接原 Slint callbacks，例如 page navigation、project selection、settings save、engine select、build/package/install/open。
  - 原 `binding::apply_snapshot` 改为 `view_model -> serde DTO`，前端通过 command 拉取状态；长任务完成后用 Tauri event 推送 `hub-state-changed`。
- React 前端：
  - 新建 `zircon_hub/web`，使用 Vite + React + TypeScript + npm。
  - 安装 `@mui/material @emotion/react @emotion/styled @mui/icons-material @tauri-apps/api`。
  - 组件分层固定为：
    - `theme/tokens`：从 web reference CSS 变量和截图提取颜色、半径、间距、密度。
    - `components/inputs`：Button、IconButton、TextField、SearchBox、Select/ComboBox、Checkbox、Switch、ToggleButton、Tabs。
    - `components/data`：List、TreeView、Table、DataRow、MetricCard、StatusBadge。
    - `components/overlays`：Menu、Popover、Dialog、Snackbar、Drawer。
    - `components/shell`：WindowView、TopBar、NavigationDrawer、PageSurface。
    - `pages`：只组装组件和绑定数据，不写局部控件样式。
- 契约更新：
  - 删除/替换当前禁止 WebView/HTML/CSS/Tauri 的 Slint-only 契约。
  - 新增 React/MUI 契约：组件目录分层、禁止页面内重复定义低级控件、禁止直接使用 AI draft PNG 作为 UI、验证 MUI 包和 theme tokens 被实际消费。

## Test Plan
- 前端：
  - `npm run typecheck`
  - `npm run build`
  - React component/unit tests覆盖低级组件、组合组件、路由状态、command 调用封装。
  - 用 Playwright/Edge 对 React 页面跑现有 web-reference 等价检查：响应式无横向溢出、19 个页面/状态可打开、菜单/弹窗/路由可点击。
- Rust：
  - `cargo fmt --manifest-path zircon_hub/Cargo.toml --check`
  - `cargo test --manifest-path zircon_hub/Cargo.toml --locked --offline --jobs 1`
  - 重点覆盖 `HubViewModel` JSON shape、`HubActionRequest` 分发、设置同步、项目/源码引擎/构建动作状态更新。
- Tauri 集成：
  - `npm run tauri dev` 打开窗口并验证 Dashboard、Browser、Settings、Source Engine popup、User menu。
  - `npm run tauri build` 或等价 Tauri build 验证生产资源嵌入。
  - 实际窗口截图与 `docs/ui-and-layout/hub-ai-drafts` / `hub-web-reference` 对比，至少覆盖 Dashboard、Browser、Detail、Settings、popup、empty/loading/error。

## Assumptions
- 包管理器使用 npm，因为当前机器有 Node `22.13.1` 和 npm `11.1.0`，未安装 pnpm/yarn。
- UI 采用真实 MUI React 组件加 theme/sx/styled 覆盖；不再手写“仿 MUI”的纯 HTML 控件。
- 视觉目标以现有 `docs/ui-and-layout/hub-web-reference` 和 `hub-ai-drafts` 为准；像素一致通过共享 theme tokens 和组件规则逼近，不做逐页逐像素补丁。
- Slint 文件可在迁移完成后删除或移出 `zircon_hub` 构建路径；不会作为长期备用入口维护。