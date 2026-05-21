---
related_code:
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_edit_mode_projection.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/mod.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_edit_mode_projection.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_hierarchy_row.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field_value.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_stats.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_toolbar_state.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/model/menu/selection_menu.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_items_for_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/tests/editing/editor_projection.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_runtime/src/scene/editor_projection/projection.rs
  - zircon_runtime/src/scene/editor_projection/inspector.rs
  - zircon_runtime/src/scene/reflect/fixed/camera_component.rs
  - zircon_runtime/src/scene/reflect/fixed/hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/lights.rs
  - zircon_runtime/src/scene/reflect/fixed/mesh_renderer.rs
  - zircon_runtime/src/scene/reflect/fixed/mobility.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime_interface/src/reflect/read_write.rs
  - zircon_runtime_interface/src/reflect/schema.rs
  - zircon_runtime/src/scene/components/scene.rs
implementation_files:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_edit_mode_projection.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_render_snapshot.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_pointer_route.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/mod.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_edit_mode_projection.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_hierarchy_row.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field_value.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_stats.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_toolbar_state.rs
  - zircon_editor/src/core/editing/command.rs
  - zircon_editor/src/core/editor_operation.rs
  - zircon_editor/src/ui/workbench/state/editor_state_field_updates.rs
  - zircon_editor/src/ui/workbench/state/editor_state_selection.rs
  - zircon_editor/src/ui/workbench/model/menu/selection_menu.rs
  - zircon_editor/src/ui/workbench/model/menu_item_model.rs
  - zircon_editor/src/ui/retained_host/menu_pointer/menu_items_for_layout.rs
  - zircon_editor/src/ui/layouts/windows/workbench_host_window/chrome_template_projection.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - zircon_editor/src/tests/editing/editor_projection.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy-Style 自研 ECS 与场景编辑模式计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - docs/assets-and-rendering/bevy-rendering-capability-matrix.md
  - docs/superpowers/plans/2026-05-08-reflection-type-registry-implementation.md
tests:
  - zircon_editor/src/tests/editing/viewport.rs
  - zircon_editor/src/tests/editing/editor_projection.rs
  - zircon_editor/src/tests/editing/reflected_command.rs
  - zircon_editor/src/tests/workbench/view_model/shell_projection.rs
  - zircon_editor/src/tests/editor_event/runtime.rs
  - zircon_runtime/src/scene/tests/ecs_reflect/editor_remote.rs
  - zircon_runtime/src/scene/tests/editor_projection.rs
  - tests/acceptance/ecs-to-render-chain.md
  - tests/acceptance/reflection-type-registry.md
  - cargo test -p zircon_editor --lib viewport_edit_mode_projection --locked
  - cargo test -p zircon_editor --lib viewport_edit_mode_projection_consumes_runtime_reflection_inspector_fields --locked --jobs 1 --message-format short
  - cargo test -p zircon_editor --lib reflected_editor_command --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib scene::tests::ecs_reflect --locked --message-format short
doc_type: module-detail
---

# Scene Viewport Edit Mode Projection

## Purpose

`zircon_editor::scene::viewport::edit_mode_projection` builds the editor-only scene edit-mode DTOs from the authoritative runtime `World`. The runtime world still owns entities, hierarchy, component values, active propagation, transforms, and render extract data. The editor projection owns only derived panel state: hierarchy rows, selected inspector fields, toolbar state, and scene statistics.

This keeps the scene/editor boundary aligned with the convergence plan: `zircon_runtime::scene` exposes neutral world state, while `zircon_editor::scene` owns authoring selection, tools, camera override, handles, overlays, and panel projections.

## Data Model

- `SceneEditModeProjection` is the single snapshot consumed by editor scene-mode UI code.
- `SceneHierarchyRow` contains entity id, parent id, display name, kind label, depth, selection state, active-in-hierarchy state, and child presence.
- `SceneInspectorField` contains component group, display label, optional property path, typed value, and editability.
- `SceneInspectorFieldValue` preserves typed values for UI binding without exposing runtime ECS storage internals.
- `SceneViewportToolbarState` mirrors current viewport tool/settings state plus selection-derived affordances.
- `SceneViewportStats` reports real counts from the runtime world: nodes, visible nodes, cameras, mesh renderers, lights, and selected entity.

## Runtime Boundary

The projection reads through stable `World` query surfaces such as `nodes()`, `find_node(...)`, `active_self(...)`, `active_in_hierarchy(...)`, `render_layer_mask(...)`, and `mobility(...)`. It does not mutate runtime world state and does not write editor state back into scene serialization or render extract data.

The projection also sanitizes stale editor selection. If the editor-selected entity no longer exists in the runtime world, `selected_entity` becomes `None`, hierarchy rows are unselected, inspector fields are empty, and toolbar frame-selection affordances are disabled. M10 uses the same runtime-projected selection when building render-snapshot selection highlights, selection anchors, handle overlays, and pointer overlay layouts, so stale editor ids no longer leak into authoring-only viewport overlays.

## Inspector Scope

The M10 consumer slice now takes inspector fields from `World::editor_projection(selected)` instead of rebuilding them from a hand-written `SceneNode` list in the editor crate. The runtime projection is reflection-backed and currently covers:

- fixed scene components registered in `zircon_runtime::scene::reflect::fixed`, including name, hierarchy parent, local transform, active state, active-in-hierarchy, render layer mask, mobility, camera, mesh renderer, lights, and rigid body fields,
- plugin-owned dynamic JSON components registered through `ComponentTypeDescriptor`,
- reflected value metadata such as editability, serializability, component type path, display name, field name, value type, and plugin ownership.

The editor mapping layer remains deliberately thin. It translates runtime `ReflectedValue` into the existing `SceneInspectorFieldValue` enum, including `Vec2` for fields such as `RectLight.size`, and preserves reflected component type paths plus field names as inspector field ids such as `Name.value`, `LocalTransform.translation`, `MeshRenderer.model`, `AmbientLight.intensity`, `RectLight.size`, and `weather.Component.CloudLayer.coverage`.

The Bevy rendering completion slice also makes the editor creation surface match the fixed light model. The Selection menu and retained-host menu fallback now expose camera plus ambient, directional, point, rect, and spot light creation actions. These actions resolve through `MenuAction::CreateNode(...)`, stable `CreateNode.*Light` ids, and undoable `Scene.Node.Create*Light` editor operations, while the runtime `World::spawn_node(...)` still owns the actual default component construction.

Resource reference fields are still projected as typed resource strings, and read-only reflected fields remain non-editable in the editor DTO. Inspector edits now route through `EditorCommand::set_reflected_scene_field(...)`, so command capture, undo, and redo write back through `World::reflect_write(...)`; the legacy editor `ComponentPropertyPath` mutation command has been removed. The compact fixed form maps name, parent, and translation to reflected `Name.value`, `Hierarchy.parent`, and `LocalTransform.translation` updates; generic reflected rows can also submit vector, quaternion, and entity text for fields such as `LocalTransform.scale`; plugin-owned dynamic rows use their projected component type paths such as `weather.Component.CloudLayer.coverage`. The editor-side dynamic field gate also checks `World::reflect_schema(...)` and `World::reflect_read(...)` before accepting draft changes, so unloaded schemas and read-only reflected fields remain protected before command capture.

Workbench `EditorDataSnapshot` plugin component rows also use the runtime reflection source when a dynamic schema is loaded. The snapshot reads field metadata from `World::reflect_schema(...)` and values from `World::reflect_fields(...)`, then overlays editor draft text by reflected field id. Unloaded dynamic JSON falls back to visible read-only rows with a diagnostic, preserving serialized data visibility without allowing mutation against an unknown schema.

## Reflection Field Source Seam

The M8 reflection milestone established the inspector field-source seam. M10 now consumes that seam: runtime `World::editor_projection` produces hierarchy rows and reflected inspector fields, and editor viewport code translates those neutral DTOs into `SceneInspectorField` and `SceneInspectorFieldValue`.

This seam keeps ownership split cleanly:

- `zircon_runtime::scene::World` remains authoritative for component/resource values and exposes them through `WorldReflection`.
- `zircon_runtime_interface::reflect` remains the serializable DTO contract shared by editor and remote/devtools callers.
- `zircon_editor::scene` continues to own selection, viewport tools, command history, undo/redo intent, and authoring-only UI state.

M8.7 intentionally left `zircon_editor/src/scene/viewport/edit_mode_projection/build.rs` untouched while proving `world.reflect_fields(...)` and remote DTO reuse. M10 completes the next step by replacing the editor's hard-coded inspector list with runtime projection consumption while keeping editor-owned selection, viewport settings, toolbar state, stats, command history, and undo/redo outside runtime scene storage.

## Validation

Focused tests in `zircon_editor/src/tests/editing/viewport.rs` assert that edit-mode projection derives hierarchy rows, inspector fields, toolbar state, and scene stats from runtime `World`, and that stale editor selection is ignored without polluting runtime render extract overlays or editor render-snapshot overlays.

`zircon_editor/src/tests/editing/editor_projection.rs` adds the M10 consumer coverage: a plugin-owned dynamic `Cloud Layer` component registered in runtime reflection appears in the viewport inspector with command-compatible property paths, reflected editability, and typed values. The same test also keeps legacy fixed paths such as `Name.value`, `Transform.translation`, and `MeshRenderer.model` alive after the runtime projection cutover, and the Bevy light-authoring follow-up verifies `AmbientLight` plus `RectLight` inspector fields, including `RectLight.size` as `Vec2`.

`zircon_editor/src/tests/workbench/view_model/shell_projection.rs` and `zircon_editor/src/tests/editor_event/runtime.rs` cover the menu and operation surface for the light-authoring follow-up: the Selection menu lists the five light kinds, `Create Rect Light` carries `Scene.Node.CreateRectLight`, and invoking that operation dispatches through the same `MenuAction::CreateNode(NodeKind::RectLight)` path as the UI binding layer.

`zircon_editor/src/tests/editing/reflected_command.rs` covers the M10 write/snapshot side: reflected fixed component edits, dynamic plugin component edits, read-only reflected field rejection, reflected editability gating, workbench snapshot projection from loaded reflection schemas, unloaded-schema protection, vector/entity text parsing, and inspector submission undo/redo through `EditorCommand::set_reflected_scene_field(...)`. The inspector submission cases now cover fixed name, hierarchy parent, local translation, generic local scale, generic entity text, and dynamic plugin updates in reflected command batches.

Latest local evidence:

- `cargo test -p zircon_runtime --lib scene::tests::editor_projection --locked --jobs 1 --message-format short` passed: 2 passed, 0 failed, 1458 filtered out.
- `cargo test -p zircon_editor --lib viewport_edit_mode_projection_consumes_runtime_reflection_inspector_fields --locked --jobs 1 --message-format short` passed: 1 passed, 0 failed, 1341 filtered out.
- Earlier `cargo test -p zircon_editor --lib reflected_editor_command --locked --jobs 1 --message-format short --color never` evidence passed with 4 tests, 0 failed, 1342 filtered out before the fixed-form cutover. Fresh focused Cargo output is pending after the latest test expansion because the shared checkout's active Cargo/Rust queue kept the command queued with empty stdout/stderr.
- 2026-05-20 light-authoring follow-up: `rustfmt --edition 2021 --check` passed on the touched editor files, `git diff --check -- <light-authoring code/docs>` passed with repository CRLF warnings only, and `cargo metadata --locked --format-version 1 --no-deps` passed.
- `CARGO_TARGET_DIR=D:\cargo-targets\zircon-render-editor-light-authoring-0520 cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` passed. It emitted the existing runtime `scene/world/query.rs` unused-method warning and existing SpriteAtlas unused-item warnings from the active editor asset lane.
- A focused `cargo test -p zircon_editor --lib rect_light --locked --jobs 1 --message-format short --color never` attempt timed out while compiling/linking the editor test harness and produced no source diagnostics. The follow-up `cargo check -p zircon_editor --lib --tests --locked --jobs 1 --message-format short --color never` reached `zircon_editor` test checking but is blocked outside this slice by `zircon_editor/src/ui/material_editor/projection.rs`, where the active material lane has not yet handled `RenderMaterialValidationError::MissingRequiredProperty`.
