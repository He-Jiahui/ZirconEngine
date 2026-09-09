---
related_code:
  - zircon_runtime/src/ui/mod.rs
  - zircon_runtime/src/ui/module.rs
  - zircon_runtime/src/ui/prelude.rs
implementation_files:
  - zircon_runtime/src/ui/module.rs
plan_sources:
  - user: 2026-09-09 构建 ZirconEngine UI Wiki
tests:
  - zircon_runtime/src/tests/runtime_absorption/ui_architecture.rs
  - zircon_runtime/src/ui/module/tests.rs
doc_type: category-index
---

# ZirconEngine UI

ZirconEngine 的 UI 是运行时内置模块，负责从模板或 v2 资产建立保留式 UI 树，执行 Taffy 布局、文本排版、输入路由、组件状态绑定，并将表面抽取为渲染帧。模块在 Scene 初始化级别启动，依赖 Input、Scene、Graphics 模块。

## 文档地图

- [架构与生命周期](architecture.md)：模块注册、树/表面/帧的数据流。
- [UI 树与事件路由](tree-and-events.md)：节点、焦点、滚动、命中测试和冒泡。
- [模板、组件与绑定](templates-components.md)：旧模板管线、组件目录、数据绑定与热重载。
- [v2 资产管线](v2-assets.md)：Zui/v2 文档编译、原型缓存、样式解析和表面构建。
- [布局与虚拟化](layout.md)：约束、Taffy 阶段、网格/弹性布局和虚拟列表。
- [表面与渲染](surface-rendering.md)：脏域、重建、渲染抽取、文本图元和调试诊断。
- [文本系统](text.md)：shaping、双向文本、换行、富文本、编辑和命中测试。
- [输入与平台](input-platform.md)：winit 转换、窗口输入泵、IME、剪贴板和平台能力。
- [无障碍](accessibility.md)：语义快照、AccessKit、动作分发和预算。
- [样式与主题](style-theme.md)：级联样式、按钮样式字段、主题 token 与运行时状态。

## 最小 Rust 入口

```rust
use zircon_runtime::ui::prelude::*;
use zircon_runtime_interface::ui::event_ui::UiTreeId;

// 资产加载后编译并建立保留表面；每帧由 UiRuntimeDriver 调度布局、输入和渲染抽取。
let surface = UiV2SurfaceBuilder::build_surface(tree_id, &asset_document)?;
let size = measure_text_size("Hello", &style);
```

所有公开类型均从 `zircon_runtime::ui` 或 `zircon_runtime::ui::prelude` 导出；底层文本 shaping 与部分事务 API 保持 crate-private，由 `UiSurface` 和运行时驱动统一调用。

## 当前状态

UI v2、模板、布局、输入和无障碍代码均已吸收到 `zircon_runtime`。平台后端能力通过 `PlatformCapabilityMatrix` 报告；未启用对应 Cargo feature 时，相关后端不会注册。
