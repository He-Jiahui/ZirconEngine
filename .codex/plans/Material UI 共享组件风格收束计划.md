# Material UI 共享组件风格收束计划

## Summary
按用户选择的 **B：Shared component style pass** 实施。目标不是全量迁移所有 `.ui.toml` 页面，而是把共享 Material 组件、主题 token、native painter 状态绘制收束到一致的 Material 工具型界面风格，让按钮、输入框、列表、菜单、标签、开关、滑条、面板看起来统一、克制、可读。

默认视觉基线：
- 标准控件高度 `40px`，紧凑控件 `32px`，输入框/突出字段 `56px`。
- 圆角使用 `4/5/8px`，避免尖锐或随意半径。
- 状态优先级固定为 `disabled > pressed > selected/focused > primary/accent > hovered > default`。
- Material 暗色工具界面，不做装饰性渐变、营销式大卡片或全局页面重排。

## Key Changes
- 主题层：调整 `editor_material.ui.toml` 的 Material token，让 `surface / inset / hover / pressed / selected / disabled / outline / focus / text` 形成统一色阶，并补齐 state-layer、disabled opacity、focus ring、radius、density token。
- 组件层：更新 `material_meta_components.ui.toml`，让 Button、IconButton、InputField/TextField、ComboBox、ListItem/TableRow、MenuItem、Tab、Checkbox/Switch、Slider、GroupBox/Panel 默认携带统一 `surface_variant`、`button_variant`、`text_tone`、`corner_radius`、`border_width`、`font_size`、state props 和 layout metrics。
- 投影层：保持现有 `TemplatePaneNodeData` 字段，不新增公开 DTO；只确保组件投影稳定传递 `surface_variant`、`button_variant`、`validation_level`、`selected/hovered/pressed/focused/disabled` 等现有字段。
- 绘制层：更新 native template painter 的 Material 状态颜色、边框/focus ring、文本色和控件内边距规则，使同一组件在 default/hover/pressed/focused/selected/disabled/error 下视觉一致。
- 范围限制：不批量改全部页面资产，不新增完整 Material Design 组件库，不恢复 generated Slint UI，不做全局布局重构。

## Test Plan
- 更新/新增 Material token 和 meta component 合约测试，覆盖必需 token、组件根节点状态透传、交互能力、accessibility/binding forwarding。
- 更新 native painter 像素级测试，验证 default、hover、pressed、selected/focused、disabled、primary/accent、error/warning 的背景、边框、文本色。
- 增加组件投影测试，确保 Material 组件状态字段从 `.ui.toml` 投影到 host contract 后没有丢失。
- 运行验证：
  - `cargo fmt --check`
  - `cargo check -p zircon_runtime --lib`
  - focused editor tests for `material_meta_component_contracts` and `native_material_painter`
  - 如当前 dirty workspace 仍有无关编译错误，记录具体阻塞文件，不顺手修 unrelated asset-editor/resource-ref 问题。

## Assumptions
- 用户选择 B，表示先修共享组件风格，不做 C 的全局 `.ui.toml` conformance。
- 保持现有暗色 Material 方向，参考当前 `editor_material.ui.toml` 和 Slint Material token，而不是切换到浅色主题。
- 不改变业务行为和交互路由；这次只收束外观、状态表达、组件默认风格和相关测试。
- 已知当前工作区存在 unrelated 编译阻塞，实施时只在这些阻塞影响本计划测试时报告，不把它们纳入本计划修复范围。
