---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/Cargo.toml
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_runtime_overlay_ui.rs
  - zircon_editor/src/scene/viewport/interaction/mod.rs
  - zircon_editor/src/scene/viewport/interaction/gizmo_axis.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_input.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_feedback.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_state.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/slint_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/slint_host/viewport/tests/controller_submits_shared_ui_overlay_through_render_framework.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_editor/src/core/editor_event/runtime/accessors.rs
  - zircon_app/src/entry/runtime_entry_app.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/mod.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/accessors.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/tests.rs
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/types/viewport_render_frame.rs
  - zircon_graphics/src/types/viewport_render_frame_from_extract.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render/render.rs
implementation_files:
  - zircon_editor/src/lib.rs
  - zircon_editor/Cargo.toml
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_runtime_overlay_ui.rs
  - zircon_editor/src/scene/viewport/interaction/mod.rs
  - zircon_editor/src/scene/viewport/interaction/gizmo_axis.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_input.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_feedback.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_state.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/ui/workbench/state/editor_state_render.rs
  - zircon_editor/src/ui/slint_host/viewport/submit_extract.rs
  - zircon_editor/src/ui/slint_host/viewport/tests/controller_submits_shared_ui_overlay_through_render_framework.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/render/framework.rs
  - zircon_runtime/src/graphics/runtime/render_framework/submit_frame_extract/submit/submit.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_app/src/entry/runtime_entry_app.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/construct.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/mod.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/accessors.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/new.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/resize.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/orbit.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/pan.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/zoom.rs
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/types/viewport_render_frame.rs
  - zircon_graphics/src/types/viewport_render_frame_from_extract.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render/render.rs
plan_sources:
  - user: 2026-04-17 功能性拆分要求，先找明显错包模块并迁回合理包
  - user: 2026-04-17 Viewport 交互边界重构计划
  - user: 2026-04-17 PLEASE IMPLEMENT THIS PLAN
  - user: 2026-04-20 不要re-export 直接清理core里ui部分
  - user: 2026-04-21 M1 主链收口与文本底座计划，runtime 只负责中性 DTO，editor 成为作者态 overlay 唯一入口
tests:
  - zircon_editor/src/tests/host/render_framework_boundary/mod.rs
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/state.rs
  - zircon_editor/src/ui/slint_host/viewport/tests/controller_submits_shared_ui_overlay_through_render_framework.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_app/src/entry/tests.rs
  - zircon_app/src/entry/runtime_entry_app/camera_controller/tests.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --locked
  - cargo test -p zircon_app --locked
  - cargo test -p zircon_editor editor_viewport_interaction_boundary_lives_in_editor_crate --locked
  - cargo test -p zircon_editor editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings --locked
  - cargo test -p zircon_runtime render_framework_tracks_text_payloads_submitted_with_shared_ui_extracts --locked
  - cargo check -p zircon_editor --lib --locked
  - cargo check --workspace --locked
doc_type: module-detail
---

# Viewport Interaction Boundary Split

## Purpose

这轮把“Scene viewport 交互”从 `zircon_graphics` 明确抽回 editor/runtime 所属边界，避免渲染 crate 继续承载输入解释、gizmo 语义和对象编辑行为。

## Ownership Split

- `zircon_editor` 现在拥有 editor 交互类型：`GizmoAxis`、`ViewportInput`、`ViewportFeedback`、`ViewportState`，以及 `scene/viewport/interaction/` 下的全部定义。
- `zircon_editor::SceneViewportController`、handle tool、pointer route、`EditorState::viewport_state()` 和 `EditorEventRuntime::viewport_state()` 都已经切到 editor-owned 类型，不再从 `zircon_graphics` 拉 viewport interaction API。
- `zircon_editor::scene::viewport::render_packet::build_render_packet(...)` 现在也是 authoring-side owner：它基于 runtime `Scene::build_viewport_render_packet(...)` 的中性 scene extract 叠加 selection / anchor / grid / handle / gizmo / preview 环境。
- `zircon_runtime::graphics` 只保留公开渲染职责：`ViewportFrame`、`ViewportFrameTextureHandle`、`SceneRenderer`、`WgpuRenderFramework` 等入口；`ViewportRenderFrame` 与 `ViewportIconSource` 已收回 graphics crate-private seam。
- `zircon_graphics/src/viewport/**` 与旧的 `types/{gizmo_axis,viewport_input,viewport_feedback,viewport_state}.rs` 已删除，不再提供兼容 re-export。
- `zircon_runtime::scene::world::render` 只输出运行时 world geometry、camera、light 与中性 preview 默认值；默认 overlay packet 为空，不再从 world 派生 selection / gizmo / grid / handle 这类 editor 语义。

## Public API

- 删除：`zircon_graphics::{ViewportController, ViewportState, ViewportInput, ViewportFeedback, GizmoAxis}`。
- 新增 owner path：`zircon_editor::scene::viewport::{ViewportState, ViewportInput, ViewportFeedback, GizmoAxis}`。
- `zircon_editor/src/lib.rs` 不再保留这些类型的根级兼容 re-export；调用方需要直接从 `scene::viewport` 取 editor-owned interaction types。
- `zircon_editor/Cargo.toml` 已去掉对 `zircon_graphics` 的直接依赖，editor 不再通过 graphics crate 获取 viewport interaction 类型。
- graphics 内部的 `ViewportRenderFrame::from_extract(...)` 与公开的 `SceneRenderer::render(...)` 都直接使用 `UVec2` 作为 viewport 尺寸，不再把 interaction-layer `ViewportState` 混进 graphics 公共面。
- `RenderOverlayExtract` 保持留在 `zircon_runtime::core::framework::render` 作为中性 DTO；变化的是生产者所有权，而不是 DTO 位置。

## M1 Render Packet Ownership

本轮 M1 把 viewport render packet 的最终边界进一步锁死成两段：

1. `zircon_runtime::scene::World::build_viewport_render_packet(...)`
   - 只负责 runtime authority：camera、mesh、light、preview 默认值
   - 默认 `selection` / `selection_anchors` / `grid` / `handles` / `scene_gizmos` 全空
2. `zircon_editor::scene::viewport::render_packet::build_render_packet(...)`
   - 把 editor-owned `SceneViewportSettings`、当前选择集、camera snapshot、handle overlays 叠加到 runtime packet 上
   - preview lighting / preview skybox 开关也只在这里改写 packet，不写回 world

这让 runtime world 序列化和 render extract 不再受 editor 状态污染；同一个 world 可以被 runtime 路径和 editor 路径消费，而作者态差异只体现在 editor packet 这一层。

## Runtime-Style Viewport HUD

M1 这里再补了一条 editor viewport 到 runtime 文本底座的正式接线：

- `SceneViewportController::build_runtime_overlay_ui()` 会基于 editor-owned viewport 状态生成一份 shared `UiRenderExtract`
- 当前默认 HUD 是右上角的状态条，显示 `tool | projection | display mode`
- `EditorState::render_frame_submission()` 现在会把 scene `RenderFrameExtract` 和这份 HUD 一起交给宿主
- `SlintViewportController::submit_extract_with_ui(...)` 通过 `RenderFramework::submit_frame_extract_with_ui(...)` 同时提交 scene extract 与 shared UI extract
- graphics 侧仍只消费中性 `RenderFrameExtract + UiRenderExtract`，HUD 文本最终进入的仍是 glyphon-backed runtime text path，而不是 Slint 文本系统

这样 editor 现在至少有一条真实 authoring overlay 走进 runtime 文本底座，满足 M1 对“runtime UI 与 editor viewport/runtime-style overlay 共用同一套文本 backend”的完成线，同时不替换 Slint workbench 主 UI。

## Runtime Preview

- `zircon_app` 不再复用 editor viewport controller，而是在 `entry/runtime_entry_app/camera_controller/` 下持有 crate-private `RuntimeCameraController`。
- runtime 控制器只实现 `resize / orbit / pan / zoom / orbit target sync`，不再依赖 `GizmoAxis` 或任何对象编辑语义。
- runtime 左键拖拽现在是显式 no-op；选中节点的 transform 不会再被 runtime preview 隐式修改。
- 为适配当前 `winit 0.31.0-beta.2` / `softbuffer` 接口，runtime 窗口与 presenter 也同步切到了 `dyn Window` + `SurfaceResized` / `PointerMoved` / `PointerButton` 事件模型。

## Validation

- `cargo test -p zircon_graphics --locked` 通过。
- `cargo test -p zircon_app --locked` 通过，包含新的 `camera_controller` unit tests 与 runtime boundary regression。
- `cargo test -p zircon_editor editor_viewport_interaction_boundary_lives_in_editor_crate --locked` 通过。
- `cargo test -p zircon_editor editor_viewport_sources_route_through_render_framework_without_wgpu_preview_bindings --locked` 通过。
- `cargo test -p zircon_editor viewport_render_snapshot_keeps_authoring_overlay_and_preview_state_in_editor_only`
  - 证明 selection / anchor / grid / gizmo / preview 开关只由 editor packet 生产，runtime packet 仍保持中性
- `cargo test -p zircon_runtime render_framework_tracks_text_payloads_submitted_with_shared_ui_extracts --locked` 通过。
  - 证明 render framework 的 shared `UiRenderExtract` 提交通道已经能记录 text payload 统计，不再只有 runtime fixture 专用路径才能把文本送进 runtime UI pass
- `cargo test -p zircon_runtime world_bootstraps_with_renderable_defaults`
  - 证明 runtime world 默认 render extract 不包含 selection anchor / handle / gizmo / grid
- `cargo check -p zircon_editor --lib --locked` 通过。
  - 证明 viewport HUD、`render_frame_submission()` 与 `submit_extract_with_ui(...)` 已经编进正式 editor lib 路径
- `cargo check --workspace --locked` 通过。
- `cargo test -p zircon_editor --lib controller_submits_shared_ui_overlay_through_render_framework --locked` 通过。
  - 证明 editor viewport 已经能把 scene extract 和 shared `UiRenderExtract` 一起交给 render framework
- `cargo test -p zircon_editor --lib render_frame_submission_carries_editor_owned_viewport_text_overlay --locked` 通过。
  - 证明 `EditorState::render_frame_submission()` 已经稳定携带 editor-owned runtime-style HUD
- `cargo test -p zircon_editor --lib --locked` 通过。
  - 证明 editor lib test 树当前已经覆盖并接受这条 shared UI HUD 提交路径，不再停留在先前的局部 focused green
- `cargo test --workspace --locked` 通过。
  - 证明当前 `M1` viewport/runtime text 收口链已经纳入工作区全量测试闭环
