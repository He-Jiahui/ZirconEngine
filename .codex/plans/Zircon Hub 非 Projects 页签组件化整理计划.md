# Zircon Hub 非 Projects 页签组件化整理计划

## Summary
- 按方案 A 成组推进：先统一轻量列表页，再整理 Cloud/Team，再整理 Editor/Builds；`Settings` 本轮排除。
- 目标是让非 Projects 页签复用 Projects 已验证的页面节奏：统一外边距、面板标题、徽章、行组件、内部滚动和窄窗口换行。
- 不改 Rust 业务数据模型和回调语义；只在 Slint 组件接口确实需要时做最小绑定修正。

## Key Changes
- 在 `zircon_hub/ui/components.slint` 补齐通用布局组件：
  - `PageScrollSurface`：统一页面背景、28px 内容边距、底部留白、页面级滚动。
  - `CatalogListPanel`：统一 Assets / Plugins / Learn 的标题徽章、空态、列表滚动和 `InfoRow` 间距。
  - `ResponsivePanelFlow`：统一 Cloud / Team / Editor / Builds 的双列或多面板换行规则，窄窗口时自然纵向堆叠。
  - 保持现有 `HubPanel`、`PanelHeader`、`InfoRow`、`ActionRow`、`DataTable` 作为主组件，不另起一套视觉风格。
- 第一组：Assets / Plugins / Learn
  - 三页改为共享 catalog shell，去掉各自手写 `ScrollView` 内容高度公式。
  - Row 继续继承 `InfoRow`，但空态、标题栏、count badge、列表滚动由 `CatalogListPanel` 统一。
- 第二组：Cloud / Team
  - Cloud 的指标区从固定 `GridLayout` 改为响应式面板流，服务列表继续用 `InfoRow`。
  - Team 的身份/仓库摘要和成员列表改用同一响应式面板结构，窄窗口不挤压文字或状态栏。
- 第三组：Editor / Builds
  - 两页共享 engine workspace 布局：概览面板、操作面板、详情/列表面板按同一 responsive flow 组合。
  - 保留现有 source engine、build history、build controls 回调；只替换重复手写 panel/list/action 排版。
- 明确不做：
  - 不调整 `settings.slint` 页面布局。
  - 不新增 Hub 业务功能、不改项目/构建/插件/云/团队扫描逻辑。
  - 不回滚当前工作区已有无关改动。

## Interfaces
- Slint 新增布局组件只服务 Hub UI；不新增 Rust public API。
- 若 `CatalogListPanel` 不能直接承载不同 row repeater，则实现为轻量 shell 组件，页面仍保留各自 `for` row，但外层面板、标题、空态和滚动规则统一。
- 现有 callback 名称、property 名称和 Rust binding 投影保持不变，避免扩大 `binding.rs` / `view_model.rs` 修改面。

## Test Plan
- 静态检查：
  - 搜索 `zircon_hub/ui/*.slint`，确认本轮触及页面不再使用页面级手算内容高度。
  - 确认新增/迁移控件继续使用 SVG 图标，不引入字符图标。
- 构建验证：
  - `cargo fmt -p zircon_hub --check`
  - `cargo check -p zircon_hub --locked --offline --jobs 1`
  - 可用时运行 `cargo build -p zircon_hub --locked --offline --jobs 1`
- 视觉验收：
  - 用 Hub 截图脚本分别检查 Assets、Plugins、Learn、Cloud、Team、Editor、Builds。
  - 至少验证 1600x1024 和较窄窗口两档：页面不重叠、不裁剪、不压住底部状态栏，长列表在面板内或页面内按设计滚动。

## Assumptions
- Projects 是本轮视觉和布局基准。
- `Settings` 明确留到后续，不作为本轮完成条件。
- 当前仓库有大量已有改动，实施时只触碰 Hub UI 组件和相关 Hub 文档/必要编译修正。
