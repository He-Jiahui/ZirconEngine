---
related_code:
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_camera.rs
  - zircon_editor/src/scene/viewport/handles/mod.rs
  - zircon_editor/src/scene/viewport/pointer/viewport_pointer_layout.rs
  - zircon_runtime/src/core/framework/render/camera.rs
  - zircon_runtime/src/core/framework/render/camera_stack.rs
  - zircon_runtime/src/scene/render_extract/mod.rs
implementation_files:
  - zircon_editor/src/scene/viewport/render_packet.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - docs/plans/zircon_runtime/render/09-camera-render-ordering.md
  - docs/editor-and-tooling/scene-viewport-gizmo-handle-overlays.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Scene Viewport Render Packet

## Purpose

`zircon_editor::scene::viewport::render_packet` builds the editor Scene viewport render snapshot. Runtime scene extraction owns base scene data, camera payload consumption, render settings, and renderer-facing DTOs. The editor packet layer owns authoring-only overlays: selected-object outlines, selection anchors, grid state, transform handles, camera/light gizmos, preview environment flags, and display mode.

## Related Files

- `zircon_editor/src/scene/viewport/render_packet.rs` builds `RenderSceneSnapshot` from a runtime `Scene` plus editor viewport state.
- `zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs` is the controller entry that supplies camera, settings, selection, viewport size, and handle extracts.
- `zircon_runtime/src/core/framework/render/camera.rs` owns `SceneViewportExtractRequest`.
- `zircon_runtime/src/core/framework/render/camera_stack.rs` owns `CameraRenderDescriptor` and the conversion from `ViewportCameraSnapshot`.

## Behavior Model

The build path first asks runtime scene extraction for the base packet using neutral render settings and viewport size. The editor camera still originates as a `ViewportCameraSnapshot`, but the runtime request now expects `CameraRenderDescriptor`, so the packet builder converts the snapshot before calling `Scene::build_viewport_render_packet(...)`. That keeps target, order, viewport, clear, and layer ownership on the runtime descriptor contract while preserving the editor's single-preview-camera UX.

After runtime extraction returns, the editor overwrites only the editor-owned packet slices. Selection highlights and anchors come from editor selection; grid state comes from `SceneViewportSettings`; handles are supplied by the controller; camera and directional-light scene gizmos are projected from active runtime nodes with editor colors, icons, wire shapes, and pick shapes. Preview lighting, fallback skybox, and clear color remain editor preview controls.

## Edge Cases and Constraints

- `SceneViewportExtractRequest::camera` must receive a descriptor, not a bare snapshot.
- Runtime scene extraction must not learn editor selection, handle, grid, or gizmo policy.
- Gizmos are emitted only for active-in-hierarchy camera and directional light nodes.
- Selection anchors are used for non-mesh selected nodes only when gizmos are disabled.
- Wire-only display keeps selection outlines but omits selection tint.

## Test Coverage

This implementation slice is feature-first per the user's instruction. Current local validation covered `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`. Cargo emits existing warning noise from active runtime/editor work, but the descriptor conversion now compiles through the editor crate.

## Plan Sources

The packet boundary is part of the editor UI architecture implementation because retained-host viewport presentation ultimately consumes this render snapshot. The runtime camera descriptor conversion follows `docs/plans/zircon_runtime/render/09-camera-render-ordering.md`, where `SceneViewportExtractRequest::camera` was moved to descriptor-backed camera ownership.

## Open Issues or Follow-up

- Full Cargo tests remain deferred to the milestone testing stage.
- Future multi-camera editor preview work should add explicit descriptor construction in this module instead of reintroducing snapshot-only runtime requests.
