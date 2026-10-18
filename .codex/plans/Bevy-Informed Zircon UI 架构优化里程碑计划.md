# Bevy-Informed Zircon UI 架构优化里程碑计划

## Summary
以“渐进融合”为默认路线：保留 Zircon 现有 `.ui.toml`、`UiSurfaceFrame`、runtime/editor 分层和接近 Slate 的声明式架构，但把 Bevy 的清晰 UI 管线、taffy 布局、headless widget 行为、渲染抽取/批处理、窗口事件和调试诊断能力融合进来。

主要参考源：
- Bevy：`bevy_window`、`bevy_ui`、`bevy_ui_render`、`bevy_ui_widgets`，重点文件包括 [bevy_ui/src/lib.rs](/E:/Git/ZirconEngine/dev/bevy/crates/bevy_ui/src/lib.rs)、[layout/convert.rs](/E:/Git/ZirconEngine/dev/bevy/crates/bevy_ui/src/layout/convert.rs)、[focus.rs](/E:/Git/ZirconEngine/dev/bevy/crates/bevy_ui/src/focus.rs)、[bevy_ui_render/src/lib.rs](/E:/Git/ZirconEngine/dev/bevy/crates/bevy_ui_render/src/lib.rs)、[bevy_ui_widgets/src/lib.rs](/E:/Git/ZirconEngine/dev/bevy/crates/bevy_ui_widgets/src/lib.rs)、[bevy_window/src/event.rs](/E:/Git/ZirconEngine/dev/bevy/crates/bevy_window/src/event.rs)。
- Zircon：现有 UI 计划与审计文档，重点是 [Zircon UI 增量布局、增量重绘与控件池优化计划.md](/E:/Git/ZirconEngine/.codex/plans/Zircon%20UI%20增量布局、增量重绘与控件池优化计划.md)、[Editor 绘制与鼠标事件优化计划.md](/E:/Git/ZirconEngine/.codex/plans/Editor%20绘制与鼠标事件优化计划.md)、[Shared Slate-Style UI Layout, Render, And Hit Framework.md](/E:/Git/ZirconEngine/.codex/plans/Shared%20Slate-Style%20UI%20Layout,%20Render,%20And%20Hit%20Framework.md)、[shared-ui-input-events.md](/E:/Git/ZirconEngine/docs/ui-and-layout/shared-ui-input-events.md)。

## Bevy/Zircon Gap Map
- 窗口与事件：Bevy 把窗口实体、cursor、redraw、focus、scale factor、raw handle 统一成平台无关事件；Zircon 目前 runtime winit 与 editor Slint/native host 仍有分裂。目标是在 runtime interface 层收束成统一 window/input pump。
- 管线调度：Bevy `UiSystems` 明确拆分 Focus、Prepare、Content、Layout、PostLayout、Stack、Render extract；Zircon 现在有 `UiSurface::rebuild_dirty` 和 editor invalidation，但阶段边界和性能报告不够稳定。目标是显式 UI pipeline stage。
- 布局：Bevy 基于 `taffy` 支持 flex/grid/block/content size；Zircon 目前是自研递归 measure/arrange。目标是用 `UiLayoutEngine` 抽象引入 taffy-backed flex/grid，同时保留 Overlay、Free、Scrollable、virtualized list 等 Slate 风格能力。
- 交互与控件：Bevy widgets 是 headless/unstyled 行为层，状态外置，事件明确；Zircon 组件体验差异主要来自 hover、tooltip、popup、menu、drag/capture、focus 的状态机不够统一。目标是 runtime 统一 widget behavior registry，editor 只消费行为与样式结果。
- 渲染：Bevy UI render 有 Extract、Queue、Prepare、Render pass、debug overlay、shadow/gradient/text 独立阶段；Zircon 已有 paint/batch DTO，但需要变成后端确认的批处理和统计。目标是 runtime WGPU 为真渲染路径，editor painter 保持同一 `UiPaintElement` 语义投影。
- 诊断：Bevy 有 debug overlay；Zircon 已有部分 diagnostics，但缺少“鼠标体验、tooltip、hover taskbar、批次数、重建次数”的强制验收门槛。

## Key Architecture Changes
- 在 `zircon_runtime::core::framework::ui` 与 `zircon_runtime_interface::ui` 补齐中立 UI 合约：window event、input pump、pipeline stage、stage report、render stats、accessibility/focus metadata。
- 新增 `UiLayoutEngine`：`LegacyLayoutEngine` 保持现状，`TaffyLayoutEngine` 负责 flex/grid/block/content-size；输出仍然是现有 `UiArrangedTree` 和 `UiSurfaceFrame`。
- 新增 headless widget behavior 层：Button、IconButton、Checkbox、Radio、Slider、Scrollbar、TextField、Menu、Popover、Tooltip、Dock/Taskbar hover；行为发 typed events，样式继续由 `.ui.toml`/Material theme 决定。
- 强化 render pipeline：把现有 paint/batch DTO 接入真实后端批处理，输出 draw call、batch split reason、clip/scissor、text atlas、shadow/gradient/image/material stats。
- 统一 editor/runtime UX：editor workbench、menu popup、drawer、floating panel、viewport toolbar、asset preview host 都通过同一 hit grid、input state、widget behavior 和 paint contract。

## Milestones
1. **M0 Baseline Audit**
   - 用 Bevy 文件与 Zircon 现有文档重新建立差距清单，不能直接信任旧计划里的“已完成”状态。
   - 记录当前鼠标移动、hover、tooltip、popup、taskbar、layout 重建、render batch、template reload 的失败样例和基准数据。
   - 使用参考脚本 [search_feature_evidence.py](/E:/Git/ZirconEngine/.codex/skills/zircon-project-skills/zr-language-feature-design/scripts/search_feature_evidence.py) 固化引用来源。

2. **M1 Window/Input Pump Convergence**
   - 参考 Bevy `bevy_window/src/event.rs`，统一 cursor moved/entered/left、focus、scale factor、redraw、IME、close/request redraw。
   - runtime winit 与 editor host 都转换为同一种 `UiInputEvent`/window event。
   - 验收：cursor leave 清 hover，scale factor 只触发布局 metrics dirty，重复 redraw 被合并。

3. **M2 UI Pipeline/Schedule**
   - 建立阶段：InputCollect、FocusInteraction、ContentMeasure、Layout、PostLayoutStack、HitGrid、RenderExtract、BatchPrepare、PaintSubmit、Diagnostics。
   - 每帧输出 stage timing、dirty reason、layout count、hit rebuild count、render command reuse count。
   - 验收：同一控件 100 次 mouse move 不触发 template reload 或全量 layout。

4. **M3 Taffy Layout Engine**
   - 参考 Bevy `layout/convert.rs`，把 Zircon slot、metrics、linear/grid/wrap 语义映射到 taffy style。
   - Free、Overlay、Scrollable、virtualized list 保持 Zircon 自有语义；flex/grid/block 使用 taffy。
   - 验收：现有 runtime/editor `.ui.toml` golden 不退化；新增 flex/grid/wrap/text clipping/DPI 案例通过。

5. **M4 Headless Widget Behavior**
   - 参考 Bevy `bevy_ui_widgets` 的 unstyled widgets，把行为从组件名称和 editor host 特判中抽出。
   - 实现统一状态机：pressed、hovered、captured、focused、disabled、tooltip armed/shown、popup open/close、outside click、ESC close、focus return。
   - 验收：button release-outside 不触发、slider drag capture 稳定、tooltip 延迟/取消正确、hover taskbar 不闪烁、menu popup 层级与焦点正确。

6. **M5 Render Extract/Batch/Pass Hardening**
   - 参考 Bevy `bevy_ui_render/src/lib.rs`，拆分 background/image/border/text/cursor/tooltip/debug/shadow/gradient extract。
   - 批处理 key 必须包含 shader/resource/clip/text atlas/page/material/z-layer，并输出后端确认统计。
   - 验收：clip/scissor、9-slice、image、SVG/material、native/SDF text、popover overlay z-order、shadow/gradient 都有 golden 或像素级检查。

7. **M6 Incremental Dirty, Cache, Pool**
   - 合并现有增量计划：Structure、Layout、TextMeasure、Visual、State、Input、Hit、Render dirty domain。
   - render command cache、damage rect union、control pool、surface session reuse 成为默认路径。
   - 验收：resize/mouse/state 更新不重载 `.ui.toml`；未变节点复用 generation；池化节点不会泄漏 hover/focus/capture 状态。

8. **M7 Editor Host UX Cutover**
   - workbench shell、menu、drawer、floating window、hover taskbar、viewport toolbar、asset preview 全部走共享 `UiSurfaceFrame`、hit grid、input dispatch、widget behavior。
   - 修复鼠标提示、悬浮任务栏、popup outside-click、floating panel focus、cursor icon、DPI snapping。
   - 验收：editor native input 场景、宽窄窗口截图、鼠标快速移动、hover taskbar 延迟、popup 关闭路径全部稳定。

9. **M8 Accessibility, Focus, Navigation**
   - 参考 Bevy `bevy_input_focus`、`accesskit`、auto directional navigation，但修正 Bevy 文档中提到的 z-index 不敏感问题，Zircon 导航必须 layer/surface-aware。
   - 为 `.ui.toml` 节点补 role、name、state、focus policy、navigation group。
   - 验收：Tab/Shift-Tab、方向键、popup focus trap、disabled/hidden 节点、tooltip/accessibility snapshot 通过。

10. **M9 Docs And CI Gates**
   - 新增或刷新 `docs/ui-and-layout/bevy-informed-ui-architecture-plan.md`、`docs/ui-and-layout/shared-ui-input-events.md`、`docs/assets-and-rendering/runtime-ui-graphics-integration.md`、`docs/editor-and-tooling/editor-workbench-shell.md`、`docs/zircon_runtime/ui/surface/input.md`、`docs/zircon_runtime/ui/surface/diagnostics.md`。
   - 建立 acceptance 文档，记录每个里程碑的引用文件、变更文件、测试命令、截图/性能结果。

## Test Plan
- 基础检查：`cargo check --workspace --locked`，重点包为 `zircon_runtime_interface`、`zircon_runtime`、`zircon_editor`。
- Runtime focused tests：layout slots/grid/wrap/text measure、hit grid、event routing、input state、render batch、surface diagnostics、runtime UI golden。
- Editor focused tests：presenter/invalidation、native input、asset preview host、workbench shell/menu/popup/taskbar、runtime UI golden projection。
- 性能验收：固定 fixtures 下记录 layout rebuild 次数、paint command reuse、draw call/batch count、mouse move latency、tooltip open/cancel latency、popup close latency。
- 视觉验收：desktop/mobile-like 窗口尺寸、DPI scale、主题切换、tooltip/popover/taskbar/floating panel z-order 截图对比。

## Assumptions
- 不直接把 Bevy 作为 Zircon UI 框架依赖；只吸收架构设计。唯一可新增的核心第三方依赖是 `taffy`，并需在 M3 通过 workspace/license/CI 验证。
- `.ui.toml`、`UiSurfaceFrame`、runtime/editor 边界继续保留；不新增旧计划中的独立 `server` 命名或过时 root crate。
- runtime WGPU 渲染是最终真路径；editor painter 是同语义投影，不能另起一套交互或 hit-test 规则。
- 旧 UI 文档中的完成状态只作为历史背景，所有性能和体验结论以 M0 重新测量为准。
