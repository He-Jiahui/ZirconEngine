---
related_code:
  - zircon_editor/src/scene/viewport/mod.rs
  - zircon_editor/src/scene/viewport/controller/mod.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_edit_mode_projection.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/mod.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_edit_mode_projection.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_hierarchy_row.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field_value.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_stats.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_toolbar_state.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/components/scene.rs
implementation_files:
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_build_edit_mode_projection.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/mod.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_edit_mode_projection.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_hierarchy_row.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_inspector_field_value.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_stats.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/scene_viewport_toolbar_state.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy-Style 自研 ECS 与场景编辑模式计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
tests:
  - zircon_editor/src/tests/editing/viewport.rs
  - tests/acceptance/ecs-to-render-chain.md
  - cargo test -p zircon_editor --lib viewport_edit_mode_projection --locked
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

The projection also sanitizes stale editor selection. If the editor-selected entity no longer exists in the runtime world, `selected_entity` becomes `None`, hierarchy rows are unselected, inspector fields are empty, and toolbar frame-selection affordances are disabled.

## Inspector Scope

The first projection slice covers built-in runtime scene components already visible in `SceneNode` and public world query methods:

- name
- hierarchy parent
- local transform
- active self and derived active-in-hierarchy
- render layer mask
- mobility
- camera
- mesh renderer
- directional, point, and spot lights

Resource reference fields are projected as typed resource strings but marked non-editable in this DTO slice. Editing remains routed through existing command/property paths rather than direct DTO mutation.

## Validation

Focused tests in `zircon_editor/src/tests/editing/viewport.rs` assert that edit-mode projection derives hierarchy rows, inspector fields, toolbar state, and scene stats from runtime `World`, and that stale editor selection is ignored without polluting runtime render extract overlays.

The attempted command was `cargo test -p zircon_editor --lib viewport_edit_mode_projection --locked`. Early runs stopped on unrelated active asset facade and UI host blockers before editor tests ran. A fresh 2026-05-08 12:17 attempt now stops earlier in `zircon_runtime`: `UiSurfaceState` derives serde while `UiSurfaceNodePool` and `UiSurfaceNodePoolReport` in `zircon_runtime/src/ui/surface/node_pool.rs` do not implement `Serialize`/`Deserialize`. This editor projection slice did not edit the asset facade, UI host presentation, or UI surface pooling lanes.
