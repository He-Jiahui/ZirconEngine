---
related_code:
  - zircon_runtime/src/scene/mod.rs
  - zircon_runtime/src/scene/inspection/mod.rs
  - zircon_runtime/src/scene/inspection/hierarchy.rs
  - zircon_runtime/src/scene/inspection/field.rs
  - zircon_runtime/src/scene/inspection/snapshot.rs
  - zircon_runtime/src/scene/inspection/tests.rs
  - zircon_runtime/src/scene/world/generation.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/world/world.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/project_io/camera.rs
  - zircon_runtime/src/scene/world/project_io/physics.rs
  - zircon_runtime/src/scene/world/project_io/post_process.rs
  - zircon_runtime/src/scene/world/project_io/references.rs
  - zircon_runtime/src/scene/world/project_io/script.rs
  - zircon_runtime/src/scene/world/project_io/transform.rs
  - docs/zircon_runtime/scene/world/project_io.md
  - zircon_runtime/src/scene/dynamic_scene/document/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/entity/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/scene/mod.rs
  - zircon_runtime/src/scene/dynamic_scene/value/mod.rs
  - zircon_runtime/src/asset/assets/scene/mod.rs
  - zircon_runtime/src/scene/reflect/world_reflection.rs
  - zircon_runtime/src/scene/reflect/type_registry.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/active_in_hierarchy.rs
  - zircon_runtime/src/scene/components/scene/mod.rs
  - zircon_runtime/src/scene/components/scene/identity.rs
  - zircon_runtime/src/scene/components/scene/hierarchy.rs
  - zircon_runtime/src/scene/components/scene/transform.rs
  - zircon_runtime/src/scene/components/scene/activation.rs
  - zircon_runtime/src/scene/components/scene/mesh_renderer.rs
  - zircon_runtime/src/scene/components/scene/node.rs
  - zircon_runtime/src/scene/reflect/builtin_reflection/hierarchy.rs
  - zircon_runtime/src/scene/components/scene/lighting.rs
  - zircon_runtime/src/scene/components/scene/reflection/mesh_renderer.rs
  - zircon_runtime/src/core/framework/scene/mobility.rs
  - zircon_runtime/src/scene/world/query.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/asset_scene/mesh_bindings.rs
  - zircon_runtime/src/scene/tests/asset_scene/hierarchy_sources.rs
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/world_basics/world_state.rs
  - zircon_runtime/src/scene/tests/world_basics/render_extract.rs
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs
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
  - zircon_runtime/src/scene/inspection/tests.rs
  - zircon_runtime/src/scene/world/generation.rs
  - zircon_runtime/src/scene/world/generation/tests.rs
  - zircon_runtime/src/scene/world/records.rs
  - zircon_runtime/src/scene/world/typed_api.rs
  - zircon_runtime/src/scene/level_system.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/asset_scene/mesh_bindings.rs
  - zircon_runtime/src/scene/tests/asset_scene/hierarchy_sources.rs
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/world_basics/world_state.rs
  - zircon_runtime/src/scene/tests/world_basics/render_extract.rs
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_editor/src/scene/viewport/edit_mode_projection/build.rs
  - zircon_editor/src/scene/viewport/controller/scene_viewport_controller_handle_interaction.rs
  - zircon_editor/src/ui/workbench/snapshot/data/editor_state_snapshot_build.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/scene_project_serialization_boundary.py
plan_sources:
  - docs/plans/zircon_editor/editor/02-data-sync-and-messaging.md
  - docs/plans/engine-code-structure-convention.md
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
  - zircon_runtime/src/scene/inspection/tests.rs
  - zircon_runtime/src/scene/world/generation/tests.rs
  - zircon_runtime/src/scene/tests/authoring_boundary.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/asset_scene/hierarchy_sources.rs::scene_assets_keep_script_only_entities_as_empty_nodes
  - zircon_runtime/src/scene/tests/asset_scene/mesh_bindings.rs::scene_assets_instantiate_world_with_asset_bound_meshes
  - zircon_runtime/src/scene/tests/asset_scene/product_fields.rs::scene_assets_roundtrip_camera_product_fields
  - zircon_runtime/src/scene/tests/inspection.rs
  - zircon_runtime/src/scene/tests/component_structure.rs
  - zircon_runtime/src/scene/tests/dynamic_scene.rs
  - zircon_runtime/src/scene/tests/world_basics.rs
  - zircon_runtime/src/scene/tests/world_basics/world_state.rs::project_roundtrip_preserves_imported_meshes
  - zircon_runtime/src/scene/tests/world_basics/render_extract.rs::updated_transform_is_reflected_in_render_extract
  - zircon_runtime/src/scene/tests/world_basics/sprites.rs::render_product_sprite_world_frame_extract_filters_by_camera_layers
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

- `WorldInspection` contains the runtime-only `generation` header, `focused_entity`, `hierarchy_rows`, and reflected `fields`. Callers can reject a composed or split-read projection when its captured generation no longer matches the world revision.
- `WorldInspectionHierarchyRow` contains entity id, parent id, depth, display name, kind label, stable recursive `subtree_hash`, focused flag, active-in-hierarchy flag, and child presence.
- `WorldInspectionField` contains reflected component type path, component display name, field name, field display name, value type path, reflected value, writable flag, serializable flag, and plugin-owned flag.

`World::inspect_hierarchy()` is the hierarchy-only entry point and carries no editor selection input. `World::inspect_fields(entity)` is the reflected-field-only entry point and returns an empty list for a missing entity. `World::inspect_world(focused)` is the planned composition façade: it captures `World::world_generation()` into the snapshot header, validates the focused entity, composes those two split reads, and applies the composed snapshot's focused row marker without making the hierarchy query depend on editor state.

Current editor consumers use the split reads directly. Viewport selection validity checks call `World::contains_entity`; the edit-mode projection requests hierarchy and inspector fields independently instead of consuming the composed `WorldInspection` façade. M2 owns cache/diff scheduling that will decide which split projection is recomputed after a watch invalidation; M1 establishes the independent read boundaries but does not yet claim inspector-only UI refresh skips the hierarchy call.

## Reflection Rules

Inspection walks `world.type_registry().iter()` and keeps component registrations that are inspection-visible through the existing reflection metadata, backed by a `ReflectComponent` adapter, and present on the focused entity.

The field list is schema-led:

- visible reflected fields are included,
- field names, display names, value type paths, writability, serializability, and plugin ownership come from the reflected registration,
- values come from adapter `read_fields`,
- plugin-owned dynamic JSON components and built-in derived components share the same inspection path.

This keeps editor UI code from hard-coding fixed fields such as `Name.value`, `MeshRenderer.model`, or plugin component JSON fields. Editor-specific labels, property paths, command routing, and viewport affordances are projected in `zircon_editor`.

## Hierarchy Rules

Hierarchy rows are built from `World::node_records()` and `World::active_in_hierarchy`. Rows are emitted root-first, depth annotated, and guarded by a visited set so malformed imported parent data cannot create infinite traversal. Orphaned or cyclic leftovers not reached from roots are still emitted as depth-zero rows to preserve inspectability.

Each row's `subtree_hash` is a stable FNV-1a digest of its display-name bytes, ordered direct-child entity ids, and each child's post-order subtree hash. Traversal uses an explicit stack so a valid 5k-deep editor hierarchy does not consume the Rust call stack. Every direct edge remains encoded: a cycle/visited or missing child writes its entity id with a zero child hash, matching the former recursive projection instead of silently dropping malformed-edge identity. A descendant rename therefore changes that descendant and all ancestors, a reparent changes the old and new parent chains while preserving the moved subtree hash, and unrelated roots remain unchanged. The constants are named and local to the inspection implementation; the hash is a projection revision, not a persisted asset id or security primitive.

## World Generation

`World::world_generation()` exposes a monotonic runtime-only `u64` revision. Successful entity spawn paths, successful despawn, and effective reparent operations advance it exactly once per structural mutation; the typed component mutation throat advances the same revision for insert/replace/remove and successful mutable access so a renamed or edited row cannot be answered with stale `NotModified`. Rejected/no-op component lookup and rejected record import leave both world state and generation unchanged; multi-record restoration validates on a staged world before replacing the authority. Whole-world replacement advances past the larger of the current and incoming runtime revisions, so deserialized/reloaded worlds cannot move a live level's generation backward. At the practically unreachable saturated revision, the query contract disables `NotModified` and returns rows on every request rather than wrapping to an old identity. The field is skipped by serde and excluded from persistent `World` equality through its private revision wrapper, so project/dynamic-scene data never stores session synchronization state.

Fixed-component presence rebuilding is an internal ECS storage projection, not a second world mutation. `insert_rebuilt_fixed_component_presence` writes the already-authoritative fixed value directly into typed storage and preserves the existing archetype, query-cache, and component Add/Replace/Insert lifecycle reporting. Component-specific derived-state dirty marking happens before synchronous lifecycle callbacks, so observers see the same ready-to-recompute projection state as public `World::insert`; generation advancement remains separated and occurs exactly once at the outer structural boundary. The helper deliberately does not re-enter public `World::insert` or advance `WorldGeneration`; therefore a node spawn/import advances generation once, while serde reconstruction remains at generation zero.

The generation owner is `scene/world/generation.rs`; mutation methods only call its private advance operation. Editor02 M2 will pair this revision with subscription flushing and component-watch invalidation. No editor view id, watch token, gateway object, or message transport is stored in `World`.

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

The 2026-06-24 Runtime 15 M3 scene asset integration test folder split keeps scene asset inspection and serialization-adjacent coverage under the test-file budget. `scene/tests/asset_scene.rs` now owns only shared helpers and child mounts, while `scene/tests/asset_scene/mesh_bindings.rs`, `scene/tests/asset_scene/hierarchy_sources.rs`, and `scene/tests/asset_scene/product_fields.rs` own the moved asset-bound mesh, hierarchy/source, camera, physics, animation, and light product-field tests. Guard `runtime_15_scene_asset_integration_tests_are_folder_backed` locks that boundary and the status anchor `runtime_15_scene_asset_integration_tests_folder_split_static_passed_cargo_deferred`; Cargo remains deferred under the Runtime 15 implementation-slice cadence.

The 2026-06-24 Runtime 15 M3 scene world basics test folder split keeps world project serialization, hierarchy, and render-extract smoke coverage under the test-file budget. `scene/tests/world_basics.rs` now owns only shared imports and child mounts, while `scene/tests/world_basics/world_state.rs`, `scene/tests/world_basics/render_extract.rs`, and `scene/tests/world_basics/sprites.rs` own the 15 world basics tests. Guard `runtime_15_scene_world_basics_tests_are_folder_backed` locks that boundary and the status anchor `runtime_15_scene_world_basics_tests_folder_split_static_passed_cargo_deferred`; the 2026-07-01 follow-up repaired its status/date map reads to `expected_slices/{status,date}/runtime_15/m3_structure_support.rs`. Cargo remains deferred under the Runtime 15 implementation-slice cadence.

## Serialization Guard Matrix

| Runtime outlet | Serialized token guard | Source token guard |
|---|---|---|
| World project JSON | `scene::tests::world_basics::world_state::project_roundtrip_preserves_imported_meshes` | `scene::tests::component_structure::scene_project_serialization_sources_do_not_store_editor_authoring_state` |
| Dynamic scene JSON | `scene::tests::dynamic_scene::dynamic_scene_roundtrips_reflected_components_with_entity_remap` and `scene::tests::dynamic_scene::versioned_json_migrates_legacy_world_project_documents` | `scene::tests::component_structure::scene_project_serialization_sources_do_not_store_editor_authoring_state` |
| Asset scene JSON | `scene::tests::asset_scene::scene_assets_instantiate_world_with_asset_bound_meshes` | `scene::tests::component_structure::scene_project_serialization_sources_do_not_store_editor_authoring_state` |
| World inspection JSON | `scene::tests::inspection::world_inspection_serialization_excludes_editor_authoring_tokens` | Neutral public-surface guard in `scene::tests::component_structure::runtime_scene_exposes_neutral_world_inspection_surface` |

`SERIALIZED_AUTHORING_TOKENS` and `SOURCE_AUTHORING_TOKENS` are intentionally narrow deny lists, not a global ban on words such as `selection` in runtime code. Runtime production code may use those words for platform/module selection or accessibility semantics, but scene/project serialization exits must not persist editor authoring state. The token tables must stay sorted and deduplicated through `scene::tests::authoring_boundary::authoring_token_tables_stay_sorted_and_deduplicated`. When `zircon_editor/src/scene` adds a new authoring state type, overlay, gizmo, viewport tool, preview override, or extract DTO, the matching runtime deny-list token must be added in the same change if it could cross a runtime serialization or inspection boundary.

## Validation

`zircon_runtime/src/scene/inspection/tests.rs` verifies split-entry/composition equivalence, subtree-hash propagation for rename/reparent, cycle-edge versus broken-edge identity, deterministic repeated hashing, and a 5k-deep iterative hierarchy walk. `zircon_runtime/src/scene/world/generation/tests.rs` verifies monotonic structural generation, typed component replacement and no-op behavior, fixed-presence dirty/query/lifecycle ordering, failed mutable lookup, atomic rejected single/batch record import, explicit-id spawn counting, and the serde boundary. `zircon_runtime/src/scene/level_system.rs` verifies generation continuity across whole-world replacement. `zircon_runtime/src/scene/tests/inspection.rs` continues to verify hierarchy order, focus filtering, built-in component reflection, plugin-owned dynamic component reflection, writable/read-only field flags, non-mutating invalid-focus behavior, and serialized inspection snapshots free of editor authoring tokens.

`zircon_runtime/src/scene/tests/component_structure.rs` rejects reintroducing the old production `scene/editor_projection` module, checks that the runtime scene public inspection files do not expose `SceneEditor*` symbols, and guards scene/project serialization source files against editor authoring-state names.

`zircon_runtime/src/scene/tests/world_basics/world_state.rs` keeps project roundtrip JSON free of selection, overlay, gizmo, preview override, editor viewport tool, and display-mode keys, while `zircon_runtime/src/scene/tests/world_basics.rs` remains the shared parent mount.

`zircon_runtime/src/scene/tests/dynamic_scene.rs` and `zircon_runtime/src/scene/tests/asset_scene.rs` keep dynamic scene and scene asset JSON on the same authoring-state boundary as world project JSON.
