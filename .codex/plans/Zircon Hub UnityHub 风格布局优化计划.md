# Zircon Hub UnityHub 风格布局优化计划

## Summary
把 `zircon_hub` 从当前基础三页表单，重构为参考图风格的真实数据仪表盘：无边框自绘窗口、顶部状态栏、完整左侧导航、Projects 首页卡片网格、最近项目表、快捷操作面板，并继续接入现有项目/引擎/设置逻辑。项目封面本轮使用 Slint 绘制占位视觉，不新增图片资源。

## Key Changes
- 重做 Hub 外壳：
  - `HubWindow` 设置 `no-frame: true`、`resize-border-width`，窗口尺寸提升到接近桌面 Hub 体验。
  - 增加自绘标题栏：品牌区、引擎选择胶囊、Running/Success/Warning/Error 状态胶囊、用户区、最小化/最大化/关闭按钮。
  - Rust 侧新增窗口控制回调：最小化、最大化切换、关闭、标题栏拖拽；关闭使用 Slint window hide/事件循环退出语义。
- 重做导航：
  - `HubPage` 扩展为 `Projects / Editor / Assets / Builds / Plugins / Cloud / Team / Learn / Settings`。
  - 所有导航项可切换；`Projects / Editor / Builds / Settings` 接真实功能，其余显示同风格占位页。
  - 原 `Installs` 语义迁移到 `Editor` 页面，显示源码引擎、输出目录、Build、Open Output、Launch Editor。
- 重做 Projects 首页：
  - 顶部标题、搜索框、Import Project、New Project、筛选/排序/视图按钮。
  - 最近项目生成最多 4 张项目卡片；点击卡片直接打开对应项目。
  - 最近项目表展示名称、引擎版本占位、最近打开时间、路径、行操作按钮。
  - Quick Actions 包含 Build Project、Install to Device、Package Project、Open in Editor；当前可实现项接现有 build/open/launch，暂不可实现项显示状态提示。
- 调整数据绑定：
  - 不再只用 `projects-summary` / `installs-summary` 文本摘要。
  - 新增 Slint 数据结构：项目卡片、最近项目行、导航项、状态胶囊、快捷操作项。
  - 新增 `src/app/view_model.rs` 或等价子模块承载格式化和模型构建，保持 `binding.rs` 只负责把快照投到 UI。
- 文档同步：
  - 更新 `docs/zircon_hub/index.md`，记录新外壳、导航、数据模型、窗口控制和仍为占位的页面。

## Interfaces
- Slint `HubWindow` 新增/替换接口：
  - properties: `nav-items`, `project-cards`, `recent-project-rows`, `engine-status-*`, `quick-actions`, `selected-page`
  - callbacks: `show-page(string)`, `open-recent-project(string)`, `quick-action(string)`, `window-minimize()`, `window-toggle-maximize()`, `window-close()`, `window-drag-start(...)`, `window-drag-move(...)`, `window-drag-end()`
- Rust `HubRuntime` 新增行为：
  - 根据字符串导航目标解析 `HubPage`
  - 根据项目路径打开最近项目
  - 根据 quick action id 分派到现有 build/open/launch 或返回“暂未实现”状态

## Test Plan
- `cargo fmt -p zircon_hub --check`
- `cargo check -p zircon_hub --locked`
- `cargo test -p zircon_hub --locked`
- 新增/更新测试覆盖：
  - 最近项目到卡片/表格模型的转换与搜索过滤
  - 导航字符串到 `HubPage` 的解析
  - quick action id 分派到正确状态或功能
- 手动验收：
  - 启动 Hub 后无系统标题栏，窗口按钮可用，标题栏可拖拽。
  - Projects 页在空项目和已有最近项目两种状态下都不重叠、不溢出。
  - Editor/Builds/Settings 页仍能保存设置、构建、打开输出目录、启动编辑器。

## Assumptions
- 本轮优先“真实数据仪表盘”，不是纯静态高保真截图。
- 项目封面使用 Slint 绘制占位封面和品牌标记，不新增图片素材。
- 非核心导航页本轮做可切换占位页，不实现 Assets/Plugins/Cloud/Team/Learn 的真实业务。
- 无边框窗口使用 Slint `no-frame` 和 Rust 回调实现；若某平台拖拽限制明显，仍保留自绘窗口按钮并在文档记录平台限制。
