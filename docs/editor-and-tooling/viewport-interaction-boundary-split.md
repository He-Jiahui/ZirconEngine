---
related_code:
  - zircon_editor/src/lib.rs
  - zircon_editor/Cargo.toml
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/interaction/mod.rs
  - zircon_editor/src/scene/viewport/interaction/gizmo_axis.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_input.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_feedback.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_state.rs
  - zircon_editor/src/core/editor_event/runtime/accessors.rs
  - zircon_entry/src/entry/runtime_entry_app.rs
  - zircon_entry/src/entry/runtime_entry_app/application_handler.rs
  - zircon_entry/src/entry/runtime_entry_app/construct.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/mod.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/accessors.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/tests.rs
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/types/editor_or_runtime_frame.rs
  - zircon_graphics/src/types/editor_or_runtime_frame_from_extract.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render/render.rs
implementation_files:
  - zircon_editor/src/lib.rs
  - zircon_editor/Cargo.toml
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/interaction/mod.rs
  - zircon_editor/src/scene/viewport/interaction/gizmo_axis.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_input.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_feedback.rs
  - zircon_editor/src/scene/viewport/interaction/viewport_state.rs
  - zircon_entry/src/entry/runtime_entry_app.rs
  - zircon_entry/src/entry/runtime_entry_app/application_handler.rs
  - zircon_entry/src/entry/runtime_entry_app/construct.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/mod.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/accessors.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/new.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/resize.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/orbit.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/pan.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/zoom.rs
  - zircon_graphics/src/lib.rs
  - zircon_graphics/src/types/editor_or_runtime_frame.rs
  - zircon_graphics/src/types/editor_or_runtime_frame_from_extract.rs
  - zircon_graphics/src/scene/scene_renderer/core/scene_renderer_render/render.rs
plan_sources:
  - user: 2026-04-17 功能性拆分要求，先找明显错包模块并迁回合理包
  - user: 2026-04-17 Viewport 交互边界重构计划
  - user: 2026-04-17 PLEASE IMPLEMENT THIS PLAN
tests:
  - zircon_editor/src/tests/host/render_server_boundary.rs
  - zircon_entry/src/entry/tests.rs
  - zircon_entry/src/entry/runtime_entry_app/camera_controller/tests.rs
  - zircon_graphics/src/tests/project_render.rs
  - zircon_graphics/src/tests/hybrid_gi_gpu.rs
  - zircon_graphics/src/tests/hybrid_gi_resolve_render.rs
  - zircon_graphics/src/tests/virtual_geometry_gpu.rs
  - zircon_graphics/src/tests/virtual_geometry_prepare_render.rs
  - cargo test -p zircon_graphics --locked
  - cargo test -p zircon_app --locked
  - cargo test -p zircon_editor editor_viewport_interaction_boundary_lives_in_editor_crate --locked
  - cargo test -p zircon_editor editor_viewport_sources_route_through_render_server_without_wgpu_preview_bindings --locked
  - cargo check --workspace --locked
doc_type: module-detail
---

# Viewport Interaction Boundary Split

## Purpose

这轮把“Scene viewport 交互”从 `zircon_graphics` 明确抽回 editor/runtime 所属边界，避免渲染 crate 继续承载输入解释、gizmo 语义和对象编辑行为。

## Ownership Split

- `zircon_editor` 现在拥有 editor 交互类型：`GizmoAxis`、`ViewportInput`、`ViewportFeedback`、`ViewportState`，以及 `editing/viewport/interaction/` 下的全部定义。
- `zircon_editor::SceneViewportController`、handle tool、pointer route、`EditorState::viewport_state()` 和 `EditorEventRuntime::viewport_state()` 都已经切到 editor-owned 类型，不再从 `zircon_graphics` 拉 viewport interaction API。
- `zircon_graphics` 只保留渲染职责：`EditorOrRuntimeFrame`、`ViewportFrame`、`ViewportFrameTextureHandle`、`SceneRenderer`、`RenderService`、`SharedTextureRenderService`、`RuntimePreviewRenderer`、`ViewportIconSource`。
- `zircon_graphics/src/viewport/**` 与旧的 `types/{gizmo_axis,viewport_input,viewport_feedback,viewport_state}.rs` 已删除，不再提供兼容 re-export。

## Public API

- 删除：`zircon_graphics::{ViewportController, ViewportState, ViewportInput, ViewportFeedback, GizmoAxis}`。
- 新增：`zircon_editor::{ViewportState, ViewportInput, ViewportFeedback, GizmoAxis}`。
- `zircon_editor/Cargo.toml` 已去掉对 `zircon_graphics` 的直接依赖，editor 不再通过 graphics crate 获取 viewport interaction 类型。
- `EditorOrRuntimeFrame::from_extract(...)` 与 `SceneRenderer::render(...)` 的 viewport 参数现在直接使用 `UVec2`，不再把 interaction-layer `ViewportState` 混进 graphics 公共面。

## Runtime Preview

- `zircon_entry` 不再复用 editor viewport controller，而是在 `entry/runtime_entry_app/camera_controller/` 下持有 crate-private `RuntimeCameraController`。
- runtime 控制器只实现 `resize / orbit / pan / zoom / orbit target sync`，不再依赖 `GizmoAxis` 或任何对象编辑语义。
- runtime 左键拖拽现在是显式 no-op；选中节点的 transform 不会再被 runtime preview 隐式修改。
- 为适配当前 `winit 0.31.0-beta.2` / `softbuffer` 接口，runtime 窗口与 presenter 也同步切到了 `dyn Window` + `SurfaceResized` / `PointerMoved` / `PointerButton` 事件模型。

## Validation

- `cargo test -p zircon_graphics --locked` 通过。
- `cargo test -p zircon_app --locked` 通过，包含新的 `camera_controller` unit tests 与 runtime boundary regression。
- `cargo test -p zircon_editor editor_viewport_interaction_boundary_lives_in_editor_crate --locked` 通过。
- `cargo test -p zircon_editor editor_viewport_sources_route_through_render_server_without_wgpu_preview_bindings --locked` 通过。
- `cargo check --workspace --locked` 通过。
- `cargo test -p zircon_editor --locked` 当前仍有 15 个失败，集中在 `template_runtime` / `slint_host` / `tab_drag` 的 shared-surface projection 快照断言；这些失败模式与本轮 viewport interaction 拆包无直接代码交集，但会继续影响 editor 全包绿灯。
