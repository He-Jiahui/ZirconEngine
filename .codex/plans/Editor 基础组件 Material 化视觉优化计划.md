# Editor 基础组件 Material 化视觉优化计划

## Summary
先做 Editor 的“基础组件视觉能力”，让上层 workbench、欢迎页、Inspector、Asset Browser、组件陈列复用同一套 Material-inspired 暗色风格。目标是解决当前方正、弱反馈、提示不明显的问题，不恢复 Slint Editor host，不改变业务交互逻辑。

## Key Changes
- 视觉语言：把 Editor 基础组件统一成深色 Material 工具界面风格，默认控件圆角约 `10px`，面板约 `12px`，高强调按钮/活动项可用 pill 圆角；主色从现有偏蓝调整为更接近上图的 teal/blue，保留 amber warning 与 coral error。
- 组件状态：统一 Button、IconButton、ToggleButton、InputField、ListRow、TableRow、MenuItem、Tab、Panel 的 `hovered / pressed / focused / selected / disabled / warning / error` 显示优先级；focus 用清晰描边，pressed 用更深状态层，selected 用主色容器。
- 绘制能力：让 retained host 真正消费已有 `corner_radius`，为软件 painter、recorded command stream、GPU UI surface command 都保留圆角信息，避免 GPU presenter 仍画成方块。
- 反馈组件：补强 `success / info / warning / error` surface/text tone，供状态栏、提示条、校验消息、弹出层复用；第一轮不加复杂动画，只做明显颜色、边框和状态层。
- 上层复用：先改基础主题与共享绘制，再把 workbench chrome、viewport toolbar、pane controls、welcome controls、component showcase 的代表性控件切到同一套 Material 类和状态参数；不做页面级重排。

## Interfaces
- 不新增 `.ui.toml` 业务 schema；继续使用现有 `corner_radius`、`border_width`、`surface_variant`、`button_variant`、`text_tone`、`validation_level` 和交互状态字段。
- 内部绘制 DTO 需要扩展：`HostRecordedPaintKind`、`ChromeCommandKind`、`UiSurfaceCommandKind::{Quad, Border}` 保留 `corner_radius: f32`，GPU geometry 用该值生成圆角 solid vertices。
- `surface_variant` 增加约定值：`success`、`info`、`warning`、`danger/error`、`elevated`；旧值继续可用。

## Test Plan
- Native painter：新增圆角像素测试，验证角落不再填满、边框跟随圆角、hover/pressed/selected/focus/disabled/error/warning 颜色优先级不回退。
- Command stream/GPU：新增 recorded command 到 Chrome/GPU surface 的 radius 传递测试；WGPU headless geometry/batching 验证圆角 solid vertices、clip/damage 后仍正确并且不触发 software fallback。
- UI asset contracts：更新 Material theme/component contract 测试，冻结基础组件的 radius、state、tone、variant 转发。
- Focused validation：跑 `cargo test -p zircon_editor --lib native_material_painter --locked`、`cargo test -p zircon_runtime --lib ui_surface --locked`、`cargo test -p zircon_editor --lib component_showcase --locked`、`cargo check -p zircon_editor --lib --tests --locked`。
- 文档：同步更新 Editor workbench shell 与 UI/material 文档，记录这次基础组件视觉能力、GPU command stream 兼容点和验证证据。

## Assumptions
- 范围只覆盖 Editor retained UI，不改 `zircon_hub` Slint UI。
- 本轮优先“基础组件可复用 + 第一眼观感明显改善”，不做复杂 motion/ripple 动画。
- 保持当前 Editor Rust-owned retained host 边界，不恢复旧 Slint host 或旧生成 UI 路径。
