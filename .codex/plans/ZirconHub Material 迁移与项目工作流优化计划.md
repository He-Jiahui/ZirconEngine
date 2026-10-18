# ZirconHub Material 迁移与项目工作流优化计划

## Summary

- 以 `E:\Git\ZirconEngine\dev\material-rust-template\material-1.0\material.slint` 作为本地 Material 组件库，不复制整套模板源码；通过 `zircon_hub` 的 `@material` library path 和 `material_bridge.slint` 暴露关键组件。
- 先稳住 Hub 的 Material wrapper 层，再把所有页面的布局统一到 Slint/Taffy `FlexboxLayout` 体系，最后按你选择的“项目工作流”主线优化功能集合。
- 目标体验：Hub 以“当前选中项目 + 当前 Source Engine”为上下文，Projects、Builds、Assets、Plugins、Cloud/Package、Team 页面展示一致的可操作状态。

## Key Changes

- Material 组件收口：
  - 保留 `E:\Git\ZirconEngine\zircon_hub\build.rs` 中的 `@material` 注册。
  - 维护 `E:\Git\ZirconEngine\zircon_hub\ui\material_bridge.slint` 作为纯 re-export 层。
  - 关键迁移范围锁定为：Cards、Buttons/IconButtons、TextField/SearchBar、SegmentedButton、PopupMenu/MenuItem、ListTile、Navigation、Progress、Tabs、Dialog/Overlay、ScrollView。
  - Hub 自有 wrapper 保持稳定 API：`HubPanel`、`PillButton`、`IconButton`、`HubTextField`、`SearchBox`、`ToolbarSelect`、`DropDownButton`、`SegmentButton`、`InfoRow`、`ActionRow`。

- Taffy 响应式布局：
  - `E:\Git\ZirconEngine\zircon_hub\ui\layout.slint` 继续作为布局基础层，所有页面优先使用 `Flow`、`PanelGrid`、`ResponsiveSlot`、`WorkspacePanelSection`。
  - 禁止页面层手写 `parent.width - nav-width` 这类剩余宽度公式；让 Slint/Taffy 分配 shell、nav、content 的空间。
  - 新增或调整断点只放在 `HubTokens`，页面只消费 token。

- 项目工作流优化：
  - `selected_project_path` 成为 Hub 功能页共享上下文。
  - Quick Actions 默认作用于选中项目；无选中项目时才 fallback 到最近项目。
  - Assets 页面优先展示选中项目资产，并保留 Source Engine/Repo 资产分组。
  - Builds 页面显示当前项目绑定的 Source Engine、构建输出、最近构建记录和可执行动作。
  - Plugins 页面区分“引擎可用插件”和“项目相关插件/清单”。
  - Cloud 页面改为本地包/安装/输出状态面板，不做真实云服务实现。
  - Team 页面优先显示选中项目 Git 仓库；没有项目仓库时 fallback 到 Source Engine 仓库。

## Public Interfaces / Types

- Slint-facing types may need additive fields only:
  - `ProjectDetailData`: expose selected project workflow status if missing.
  - `AssetData`: add `scope` or reuse `source` to distinguish `Selected Project` / `Engine` / `Repo`.
  - `PluginData`: add project/engine scope only if UI needs grouping.
  - `CloudSummaryData`: keep local/offline semantics; no account API.
- Rust runtime keeps ownership inside `zircon_hub`:
  - Extend `HubSnapshot` projection, not global engine runtime.
  - Add helper methods around selected project context in `HubRuntime`.
  - Do not introduce new root crate, server facade, or runtime/editor cross-coupling.

## Milestones

- Milestone 1: Material wrapper hardening
  - Audit all Hub UI files for custom-painted controls that should now use Material-backed wrappers.
  - Replace remaining page-local buttons, field rows, menu triggers, and list rows where a wrapper already exists.
  - Update `zircon_hub/tests/ui_contract.rs` so regressions back to custom `TouchArea` controls are caught.

- Milestone 2: Responsive layout completion
  - Normalize Projects secondary pages, Editor, Builds, Settings, Assets, Plugins, Cloud, Learn, and Team to tokenized Taffy primitives.
  - Remove raw page breakpoints and raw pixel literals above `1px` outside `HubTokens` / `MaterialStyleMetrics`.
  - Keep `app.slint` and `projects.slint` as routing/composition files, not implementation-heavy files.

- Milestone 3: Project-centered Hub data model
  - Add selected-project-aware projections for Assets, Plugins, Team, Cloud/package status, and build context.
  - Make page status text and empty states explain whether data is scoped to selected project, active engine, or local repo.
  - Keep persistence limited to existing Hub config/project metadata where needed.

- Milestone 4: Workflow actions and page polish
  - Make Builds, Package, Install, Open Editor, Pin/Remove/Delete project actions consistently target selected project.
  - Align top header, nav, page headers, and bottom status to show current project/engine state without overlap.
  - Update `docs/zircon_hub/index.md` and `docs/zircon_hub/ui/responsive-component-system.md`.

## Test Plan

- Static/component contracts:
  - `cargo test -p zircon_hub --test ui_contract --locked --offline --jobs 1`
- Hub compile/build:
  - `cargo fmt -p zircon_hub --check`
  - `cargo check -p zircon_hub --locked --offline --jobs 1`
  - `cargo test -p zircon_hub --locked --offline --jobs 1`
  - `cargo build -p zircon_hub --bin zircon_hub --locked --offline --jobs 1`
- Visual acceptance:
  - Use the Hub screenshot scripts for Projects dashboard/new/browser/detail at `1600x1024`, `1280x900`, and `1024x768`.
  - Check no clipped text, no overlapping top chrome, no blocked nested controls, and correct selected-project state across pages.

## Assumptions

- “迁移 Material 组件” means direct local template import plus Hub wrappers, not copying the whole template into `zircon_hub`.
- Functional optimization prioritizes selected-project workflows, per your choice.
- Current worktree is heavily dirty; implementation must preserve unrelated runtime/editor/render changes.
- Existing active sessions touch retained presenter and sprite atlas/runtime UI paths, so this plan avoids those modules.
