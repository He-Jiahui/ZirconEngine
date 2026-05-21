---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/editor_projection/mod.rs
  - zircon_runtime/src/scene/editor_projection/hierarchy.rs
  - zircon_runtime/src/scene/editor_projection/inspector.rs
  - zircon_runtime/src/scene/editor_projection/projection.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/fixed/active_in_hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/camera_component.rs
  - zircon_runtime/src/scene/reflect/fixed/hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/lights.rs
  - zircon_runtime/src/scene/reflect/fixed/mesh_renderer.rs
  - zircon_runtime/src/scene/reflect/fixed/mobility.rs
  - zircon_runtime/src/scene/world/query.rs
implementation_files:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/editor_projection/mod.rs
  - zircon_runtime/src/scene/editor_projection/hierarchy.rs
  - zircon_runtime/src/scene/editor_projection/inspector.rs
  - zircon_runtime/src/scene/editor_projection/projection.rs
  - zircon_runtime/src/scene/reflect/fixed/active_in_hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/camera_component.rs
  - zircon_runtime/src/scene/reflect/fixed/hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/lights.rs
  - zircon_runtime/src/scene/reflect/fixed/mesh_renderer.rs
  - zircon_runtime/src/scene/reflect/fixed/mobility.rs
plan_sources:
  - user: 2026-05-16 Bevy-grade ECS/reflect/scene/transform completion request
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - dev/bevy/crates/bevy_reflect/src/type_registry.rs
  - dev/bevy/crates/bevy_ecs/src/reflect/component.rs
tests:
  - zircon_runtime/src/scene/tests/editor_projection.rs
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib scene::tests::editor_projection --locked --jobs 1 --message-format short
doc_type: module-detail
---

# Editor Projection

`zircon_runtime::scene::editor_projection` is the first M10 runtime-side support layer for Bevy-grade editor integration. It exposes a read-only projection of scene hierarchy rows and inspector fields from `World` without storing editor selection, panel state, undo data, or viewport state back into runtime scene storage.

The Bevy alignment point is reflection-backed editing: component schemas and reflected component adapters are the authoritative source for inspector fields, matching the role played by Bevy's `TypeRegistry` and `ReflectComponent` plumbing. Zircon's editor can consume this DTO layer before its viewport-specific UI code is migrated away from ad hoc `SceneNode` field projection.

## Public DTOs

- `SceneEditorProjection` contains `selected_entity`, `hierarchy_rows`, and `inspector_fields`.
- `SceneEditorHierarchyRow` contains entity id, parent id, depth, display name, kind label, selected flag, active-in-hierarchy flag, and child presence.
- `SceneEditorInspectorField` contains reflected component type path, component display name, field name, field display name, value type path, reflected value, editable flag, serializable flag, and plugin-owned flag.

`World::editor_projection(selected)` is the convenience entry point. It filters missing selections to `None`, builds hierarchy rows for the current world, and only builds inspector fields for a valid selected entity.

## Inspector Rules

Inspector projection walks `world.type_registry().iter()` and keeps component registrations that are editor-visible, backed by a `ReflectComponent` adapter, and present on the selected entity.

The field list is schema-led:

- only fields marked `editor_visible` are included,
- field names, display names, value type paths, editability, serializability, and plugin ownership come from the reflected registration,
- values come from the adapter `read_fields` result,
- plugin-owned dynamic JSON components and fixed components share the same projection path.

This avoids hard-coding fields such as `Name.value` or plugin component properties in editor UI code.

The M10 editor-consumer slice expands the fixed reflection coverage needed by the viewport inspector. In addition to the M8 fixed adapters, runtime projection now sees hierarchy parent, derived active-in-hierarchy, camera clip/FOV fields, mesh renderer model/material/tint, mobility, and ambient/directional/point/rect/spot light fields. `Hierarchy.parent` is marked non-serializable because `DynamicScene` already carries parent relationships in `NodeRecord`; this prevents scene spawning from reapplying parent changes through runtime mutation rules after records have already been inserted.

## Hierarchy Rules

Hierarchy rows are built from `World::node_records()` and `World::active_in_hierarchy`. Rows are emitted root-first, depth annotated, and guarded by a visited set so malformed imported parent data cannot create infinite traversal. Orphaned or cyclic leftovers that were not reached from roots are still emitted as depth-zero rows, preserving inspectability.

## Boundaries

This module does not own editor UI layout, undo/redo stacks, retained host routing, viewport overlays, pointer handling, or gizmo state. Those remain in `zircon_editor`. The runtime projection only answers: "what would an editor need to display for this world right now?"

The next M10 slice can replace `zircon_editor::scene::viewport::edit_mode_projection::build_inspector_fields` with this runtime projection while keeping editor-only state in the editor crate.

## Validation

`zircon_runtime/src/scene/tests/editor_projection.rs` verifies:

- hierarchy root/child ordering, depth, child presence, and selection flags,
- missing selections are filtered without mutating the world,
- fixed `Name` fields appear through reflection,
- fixed `MeshRenderer` resource fields appear through reflection,
- plugin-owned dynamic component fields appear through the same reflection path,
- editable and read-only field flags are preserved.

Latest local evidence:

- `cargo test -p zircon_runtime --lib scene::tests::editor_projection --locked --jobs 1 --message-format short` passed after the M10 fixed inspector coverage expansion: 2 passed, 0 failed, 1458 filtered out.
- `cargo test -p zircon_editor --lib viewport_edit_mode_projection_consumes_runtime_reflection_inspector_fields --locked --jobs 1 --message-format short` passed: 1 passed, 0 failed, 1341 filtered out.
- `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short` currently stops outside this module in the active rendering lane at `zircon_runtime/src/scene/world/render.rs:425`, where `PostProcessExtract` is missing `graph` and `stack` fields.
