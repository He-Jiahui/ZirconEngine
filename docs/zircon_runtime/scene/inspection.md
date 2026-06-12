---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/hierarchy.rs
  - zircon_runtime/src/scene/inspection/field.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/dynamic_scene/document.rs
  - zircon_runtime/src/scene/dynamic_scene/entity.rs
  - zircon_runtime/src/scene/dynamic_scene/scene.rs
  - zircon_runtime/src/scene/dynamic_scene/value.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/fixed/active_in_hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/camera_component.rs
  - zircon_runtime/src/scene/reflect/fixed/hierarchy.rs
  - zircon_runtime/src/scene/reflect/fixed/lights.rs
  - zircon_runtime/src/scene/reflect/fixed/mesh_renderer.rs
  - zircon_runtime/src/scene/reflect/fixed/mobility.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
implementation_files:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/hierarchy.rs
  - zircon_runtime/src/scene/inspection/field.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
plan_sources:
  - user: 2026-05-16 Bevy-grade ECS/reflect/scene/transform completion request
  - user: 2026-06-04 optimize Zircon Engine runtime architecture with breaking changes allowed
  - .codex/plans/ZirconEngine Bevy-Grade ECS Reflect Scene Transform Roadmap.md
  - .codex/plans/Zircon Runtime 架构渐进式 Review 与优化计划.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - dev/Fyrox/editor/src/world/selection.rs
  - dev/Fyrox/editor/src/world/graph.rs
  - dev/bevy/crates/bevy_ecs/src/hierarchy.rs
  - dev/bevy/crates/bevy_ecs/src/world/reflect.rs
  - dev/UnrealEngine/Engine/Source/Editor/AdvancedPreviewScene/Public/SAdvancedPreviewDetailsTab.h
tests:
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py --json
  - python -m py_compile .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/audit_runtime_structure.py .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --message-format short
  - cargo test -p zircon_runtime --lib scene::tests::inspection --locked --jobs 1 --message-format short
doc_type: module-detail
---

# World Inspection

`zircon_runtime::scene::inspection` is the runtime-owned, neutral inspection snapshot for ECS hierarchy and reflected component fields. It replaces the old runtime scene editor projection path and deliberately avoids authoring terms in its public API.

The owner split is:

- `zircon_runtime::scene` owns the authoritative `World`, hierarchy, reflection registry, fixed and dynamic component adapters, and read-only `WorldInspection` snapshot.
- `zircon_editor::scene` owns selection, viewport state, gizmos, overlays, edit commands, undo/redo, and `SceneEditModeProjection`.

This follows the current reference direction: Fyrox keeps graph selection and world viewer adapters in `editor/src/world`, Bevy keeps ECS hierarchy and reflection as neutral world/runtime facilities, and Unreal Details/preview UI lives under `Engine/Source/Editor`.

## Public DTOs

- `WorldInspection` contains `focused_entity`, `hierarchy_rows`, and reflected `fields`.
- `WorldInspectionHierarchyRow` contains entity id, parent id, depth, display name, kind label, focused flag, active-in-hierarchy flag, and child presence.
- `WorldInspectionField` contains reflected component type path, component display name, field name, field display name, value type path, reflected value, writable flag, serializable flag, and plugin-owned flag.

`World::inspect_world(focused)` is the convenience entry point. It filters missing focus entities to `None`, builds hierarchy rows for the current world, and only builds reflected fields for a valid focused entity.

## Reflection Rules

Inspection walks `world.type_registry().iter()` and keeps component registrations that are inspection-visible through the existing reflection metadata, backed by a `ReflectComponent` adapter, and present on the focused entity.

The field list is schema-led:

- visible reflected fields are included,
- field names, display names, value type paths, writability, serializability, and plugin ownership come from the reflected registration,
- values come from adapter `read_fields`,
- plugin-owned dynamic JSON components and fixed components share the same inspection path.

This keeps editor UI code from hard-coding fixed fields such as `Name.value`, `MeshRenderer.model`, or plugin component JSON fields. Editor-specific labels, property paths, command routing, and viewport affordances are projected in `zircon_editor`.

## Hierarchy Rules

Hierarchy rows are built from `World::node_records()` and `World::active_in_hierarchy`. Rows are emitted root-first, depth annotated, and guarded by a visited set so malformed imported parent data cannot create infinite traversal. Orphaned or cyclic leftovers not reached from roots are still emitted as depth-zero rows to preserve inspectability.

## Boundary Rules

This module must not own:

- editor selection storage,
- inspector drafts,
- retained UI routing,
- viewport overlays,
- pointer interaction,
- gizmo/handle state,
- undo/redo commands,
- serialized authoring state.

The runtime snapshot only answers: what hierarchy rows and reflected component fields are inspectable for this world right now?

## Serialization Boundary

Scene inspection is read-only runtime projection, and project serialization is runtime state persistence. Neither path may store editor authoring state.

Allowed scene/project serialization data includes runtime world state, hierarchy, component values, runtime camera identity, render order, and camera render viewport rectangles such as `SceneViewportRectAsset` or `RenderViewportRect`.

Forbidden serialized authoring data includes selection, editor viewport tools, grid/view-orientation controls, overlays, gizmos, preview camera overrides, preview lighting, display modes, and editor pane state. These belong in `zircon_editor` state or editor-session snapshots, not in runtime scene assets or project saves.

The structural audit reports this as `scene_project_serialization_boundary`, with its owner module in `runtime_structure_audits/scene_project_serialization_boundary.py`. The mirrored Rust source guard is `scene::tests::component_structure::scene_project_serialization_sources_do_not_store_editor_authoring_state`, and runtime serialization guards share the explicit authoring-token list in `scene::tests::authoring_boundary`.

## Serialization Guard Matrix

| Runtime outlet | Serialized token guard | Source token guard |
|---|---|---|
| World project JSON | `scene::tests::world_basics::project_roundtrip_preserves_imported_meshes` | `scene::tests::component_structure::scene_project_serialization_sources_do_not_store_editor_authoring_state` |
| Dynamic scene JSON | `scene::tests::dynamic_scene::dynamic_scene_roundtrips_reflected_components_with_entity_remap` and `scene::tests::dynamic_scene::versioned_json_migrates_legacy_world_project_documents` | `scene::tests::component_structure::scene_project_serialization_sources_do_not_store_editor_authoring_state` |
| Asset scene JSON | `scene::tests::asset_scene::scene_assets_instantiate_world_with_asset_bound_meshes` | `scene::tests::component_structure::scene_project_serialization_sources_do_not_store_editor_authoring_state` |
| World inspection JSON | `scene::tests::inspection::world_inspection_serialization_excludes_editor_authoring_tokens` | Neutral public-surface guard in `scene::tests::component_structure::runtime_scene_exposes_neutral_world_inspection_surface` |

`SERIALIZED_AUTHORING_TOKENS` and `SOURCE_AUTHORING_TOKENS` are intentionally narrow deny lists, not a global ban on words such as `selection` in runtime code. Runtime production code may use those words for platform/module selection or accessibility semantics, but scene/project serialization exits must not persist editor authoring state. The token tables must stay sorted and deduplicated through `scene::tests::authoring_boundary::authoring_token_tables_stay_sorted_and_deduplicated`. When `zircon_editor/src/scene` adds a new authoring state type, overlay, gizmo, viewport tool, preview override, or extract DTO, the matching runtime deny-list token must be added in the same change if it could cross a runtime serialization or inspection boundary.

## Validation

`zircon_runtime/src/scene/tests/inspection.rs` verifies hierarchy order, focus filtering, fixed component reflection, plugin-owned dynamic component reflection, writable/read-only field flags, non-mutating invalid-focus behavior, and serialized inspection snapshots free of editor authoring tokens.

`zircon_runtime/src/scene/tests/component_structure.rs` rejects reintroducing the old production `scene/editor_projection` module, checks that the runtime scene public inspection files do not expose `SceneEditor*` symbols, and guards scene/project serialization source files against editor authoring-state names.

`zircon_runtime/src/scene/tests/world_basics.rs` keeps project roundtrip JSON free of selection, overlay, gizmo, preview override, editor viewport tool, and display-mode keys.

`zircon_runtime/src/scene/tests/dynamic_scene.rs` and `zircon_runtime/src/scene/tests/asset_scene.rs` keep dynamic scene and scene asset JSON on the same authoring-state boundary as world project JSON.
