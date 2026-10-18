# Material UI 元组件与 `.ui.toml` 编辑器布局 Slate 化计划

## Summary
- 目标：把 Material UI 元组件从 `.ui.toml` 资产一路贯通到 runtime layout、hit grid、事件路由、editor native host 和渲染层，让编辑器布局只改 `.ui.toml` 即可获得可命中、可交互、可局部重绘的控件。
- 架构参照：Unreal Slate 负责 arranged geometry、hit grid、widget path、reply/effect、focus/capture、invalidation、fast paint；Slint Material 负责组件语义、状态层和控件结构。
- 当前策略：先收束已有活跃 UI 会话产物，不重做已经存在的 `UiSurfaceFrame`、hit-grid、event dispatch、render DTO、invalidation、text M6 工作，再在其上补齐 Material 组件和 editor 布局接入。

## Milestones
- **M0 基线冻结与会话收束**
  - 核对当前活跃 UI 会话的职责边界：hit test、input events、render Slate contract、invalidation M7、widget behavior、text M6、asset browser。
  - 先修通 editor/runtime 当前编译门槛，保留现有改动，不回滚其他会话文件。
  - 验收门槛：`zircon_editor` 可 check，runtime hit-grid/event/material layout focused tests 可跑。

- **M1 Shared Slate Core 合同稳定**
  - 以 `UiSurfaceFrame` 作为 layout/render/hit/input 的唯一帧事实，继续收敛 `UiArrangedTree`、`UiHitTestGrid`、`UiHitPath`、`UiVisibility`。
  - 输入统一从 hit path 路由到 widget path，再产生 reply/effect、focus/capture、component event envelope。
  - 布局 dirty、render dirty、presentation dirty、pointer-only dirty 分层固定下来，作为后续 Material 控件的性能边界。

- **M2 Material 元组件覆盖**
  - 以 `material_meta_components.ui.toml` 为入口补齐 Material 控件：Button、IconButton、ToggleButton、CheckBox、Switch、ComboBox、LineEdit/TextEdit、Slider、SpinBox、Menu、Tabs、Scroll/Table、Progress、Spinner。
  - 每个 Material 组件必须展开为现有 Runtime UI 控件，而不是纯视觉 wrapper；bindings、focusable、interactive、value property、disabled/visibility、style token 必须透传到 expanded root control。
  - Material layout pass 负责 Slint-like 尺寸语义：padding、icon gap、min size、density、corner radius、border width、row height、popup/list viewport metrics。

- **M3 `.ui.toml` 编辑器布局接管**
  - Component Showcase、Asset Browser、Inspector surface controls、Viewport toolbar、pane template buttons 优先改为 `.ui.toml` + Material meta component 驱动。
  - Rust host projection 只承载投影合同，不再新增坐标表；`TemplatePaneNodeData` 保留 Material/媒体字段：`surface_variant`、`text_tone`、`button_variant`、`font_size`、`corner_radius`、`border_width`、`media_source`、`icon_name`、`has_preview_image`、`preview_image`。
  - Editor native host 必须从同一 arranged tree/hit grid 命中这些控件，并能把交互回传到 `.ui.toml` binding。

- **M4 完整交互闭环**
  - 覆盖 click、hover、press/release、focus/blur、keyboard submit、text live change/commit、scroll、drag/drop、popup/menu option select。
  - Button/toolbar 触发 action；CheckBox/Switch 触发 value change；TextEdit 支持光标、选择、提交；Slider/SpinBox 支持拖动和键盘调整；Combo/Menu 支持 popup path 和 option binding。
  - 所有事件结果统一走 `UiDispatchReply`/effect，不在 editor host 里绕路实现特殊控件逻辑。

- **M5 渲染与 Material 状态层**
  - Native painter 和 runtime renderer 消费同一 render extract：背景、border、rounded rect、icon/media、text、state layer、focus ring、selection、cursor、popup shadow。
  - hover、press、cursor blink、popup open/close、viewport image 更新只产生局部 damage；不得触发完整 presentation rebuild。
  - 补齐 render batch/debug 信息，能看出 Material 控件的批次、资源、dirty 原因。

- **M6 Text/Edit 收敛**
  - LineEdit/TextEdit 使用 shared text layout/shaping/visual-order DTO；Material 文本字段只配置外观和交互，不复制文本引擎。
  - 支持 placeholder、selection、caret、commit、cancel、disabled/read-only、overflow/clip。
  - IME/平台文本输入先保留接口边界，2D editor 输入路径必须先验收。

- **M7 诊断、性能与 Widget Reflector**
  - 增强 Widget Reflector：显示 widget path、arranged rect、hit-test state、visibility、focus/capture、dirty flags、render batch。
  - 性能验收固定：100 次同目标 mouse move 不触发 presentation rebuild；hover/press/text cursor/popup 只产生局部 damage；日志不反复 template load/compile。
  - 将 Component Showcase 作为 Material 控件全链路回归页。

- **M8 Runtime 独立 UI 验收**
  - Editor host 与 runtime UI 共享同一 `.ui.toml`、layout、hit、event、render contracts。
  - 保留 world-space UI 进入同一 hit grid 的接口约束，但本轮验收只覆盖 2D editor/runtime UI。
  - 清理过时 fallback、重复视觉 wrapper、未使用生成式 Slint 路径。

## Public Interfaces / Contracts
- 共享事实类型保持收敛：`UiSurfaceFrame`、`UiArrangedTree`、`UiHitTestGrid`、`UiHitPath`、`UiVisibility`。
- 输入边界固定为 shared event DTO + dispatch reply/effect；editor host 只做平台事件翻译和 native projection。
- Material meta component 的 `.ui.toml` 参数必须包含布局、视觉、状态、绑定、输入能力；bindings 透传到 expanded root control 是硬合同。
- Render contract 只输出中立 paint/debug DTO；Material painter、editor native painter、runtime WGPU renderer 都消费同一抽取结果。

## Test Plan
- 编译门槛：`cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir E:\zircon-build\targets --message-format short --color never`。
- Runtime 底座：运行 hit-grid、event routing、material layout、surface diagnostics、text layout focused tests。
- Material 覆盖：`material_meta_components_cover_slint_material_exports`、`component_showcase_uses_material_meta_component_assets`、`component_showcase_projection_carries_runtime_component_semantics`。
- 交互验收：Material button click、checkbox/switch change、text edit live change/commit、slider/spin drag、combo/menu popup select、scroll/table visible range、drag/drop reference field。
- Editor native 验收：viewport toolbar、pane template buttons、Inspector Apply/Delete、Asset Browser row hover/click、Component Showcase 全控件可命中且产生可见 repaint。
- 性能验收：pointer-only move、hover/press、cursor blink、popup update、viewport image update 均验证 dirty 分层和局部 damage。

## Assumptions
- “类虚幻 Slate”指复刻 Slate 架构语义和行为边界，不复制 Unreal C++ API 或源码结构。
- 本轮优先 2D editor/runtime UI；world-space UI 不进入第一验收主线。
- 继续采用 Rust-owned native host/painter，不恢复 deleted/generated `.slint` UI 源。
- 现有活跃会话产物视为基线输入；实施时只在其上收敛，不做大范围重写或回滚。
