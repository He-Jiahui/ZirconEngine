---
related_code:
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
implementation_files:
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
plan_sources:
  - user: 2026-05-30 continue model material mesh entity shader flow and asset management
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
tests:
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_toml_roundtrip_preserves_entities_and_bindings
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_toml_roundtrip_preserves_physics_and_animation_components
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_overview_reports_entity_component_and_reference_counts
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_overview_handles_empty_scenes
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_management_record_set_sorts_and_summarizes_records
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_management_record_set_sorts_and_summarizes_records (entity record-set assertions)
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_instantiate_world_with_asset_bound_meshes
  - zircon_runtime/src/scene/tests/asset_scene.rs::render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_roundtrip_primitive_mesh_material_bindings
  - zircon_runtime/src/asset/tests/assets/importer.rs::importer_emits_gltf_multi_primitive_material_labels
  - zircon_runtime/src/asset/tests/assets/importer.rs::importer_emits_bevy_style_gltf_labeled_subassets
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_emits_gltf_multi_scene_labels
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_emits_multi_primitive_material_labels
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_decodes_triangle_gltf_into_model_asset
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_emits_multi_scene_labels
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs::project_manager_imports_minimal_gltf_material_shader_mesh_sample
  - cargo test -p zircon_runtime --lib scene_asset_management_record_set_sorts_and_summarizes_records --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 entity record-set slice: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib scene_asset_overview --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib asset::tests::assets::scene --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 scene record set: passed, 11 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 M6 minimal asset-flow sample with typed facade load-state, primitive binding, and aggregate management assertions: passed, 1 passed, 2211 filtered; existing zircon_runtime lib-test warnings only)
doc_type: module-detail
---

# Scene Asset

## Purpose

`SceneAsset` is the serialized scene root used by runtime project loading, prefab payloads, and imported scene entries. It owns ordered `SceneEntityAsset` rows. Each entity row can carry camera, mesh/model/material binding, light, physics, animation, terrain, tilemap, and prefab-instance components.

The scene asset format remains authoring-focused. `SceneMeshInstanceAsset.mesh` is an optional direct `MeshAsset` reference beside the compatibility `model` envelope and material binding. `SceneMeshInstanceAsset.primitives` is the multi-primitive form: each `SceneMeshPrimitiveBindingAsset` pairs one direct mesh reference with the material that should shade that primitive. Both optional fields are skipped when absent or empty, so existing `.scene` assets and prefab scene payloads keep loading without serialized churn.

## Direct References

`SceneEntityAsset::direct_references()` is the per-entity dependency collector. It reports references from camera texture targets, mesh model bindings, optional direct mesh bindings, mesh primitive mesh/material pairs, the legacy entity material binding, optional collider physics material bindings, animation skeleton/player/sequence/graph/state-machine bindings, terrain and tilemap bindings, and prefab instances.

`SceneAsset::direct_references()` delegates to the entity collector in entity order. This preserves the previous aggregate behavior while making entity-level dependency counts available to asset management and diagnostics. `PrefabAsset::direct_references()` continues to use the scene aggregate, so prefab dependency reporting follows the same rule.

## Management DTOs

`SceneEntityOverview` is a compact read-only row for one scene entity. It carries the stable entity id, name, parent id, active flag, render layer mask, mobility, direct reference count, direct mesh reference count, primitive mesh/material binding count, and component-presence flags for camera, mesh, direct mesh reference, light variants, physics components, animation bindings, terrain, tilemap, and prefab instance. Helper methods derive light, physics, and animation binding counts from those flags.

`SceneAssetOverview` aggregates the entity rows into scene-level counts: total entities, active entities, roots, cameras, mesh instances, direct mesh references, primitive mesh/material bindings, mesh material bindings, collider material bindings, lights, physics components, animation bindings, terrain bindings, tilemap bindings, prefab instances, and direct references. `SceneAsset::entity_overviews()` returns ordered entity rows, and `SceneAsset::overview()` returns the aggregate view.

`SceneAssetManagementRecord` wraps a `ResourceId` with the scene overview. `SceneAsset::management_record(...)` is the asset-level constructor, and `ResourceStreamer::scene_asset_overview(...)` / `scene_asset_management_record(...)` load the same read model through the runtime asset manager for renderer-side management panels that already work with resource ids.

`SceneAssetManagementRecordSet` sorts scene records by `ResourceId` and carries `SceneAssetManagementRecordSetSummary`. The summary totals scene count, entity count, active/root entities, direct references, camera/mesh/direct-mesh/primitive-binding/material/collider/light/physics/animation counts, and terrain/tilemap/prefab bindings for scene-list headers. `ResourceStreamer::scene_asset_management_records(...)` and `scene_asset_management_record_set(...)` expose that list-level read model without forcing UI callers to scan scene rows themselves.

`SceneEntityManagementRecord` is the cross-scene row form for entity tables. It pairs the owning scene `ResourceId` with one `SceneEntityOverview`, so a panel can list entities directly while still preserving the scene identity needed for selection, navigation, or stale-row repair. `SceneAssetManagementRecord::entity_management_records(...)` projects one scene record into entity rows, `SceneAsset::entity_management_records(...)` constructs those rows directly from the asset, and `SceneEntityManagementRecordSet` sorts rows by `(scene_id, entity)` with a `SceneEntityManagementRecordSetSummary` that mirrors the scene summary counters at entity-row granularity. `ResourceStreamer::scene_entity_management_records(...)` and `scene_entity_management_record_set(...)` expose that flattened read model across all registered scene assets.

These DTOs do not attempt to instantiate ECS entities, resolve handles, or validate referenced assets. They are deliberately derived from the authoring asset only, making them cheap enough for asset browsers and safe to use when dependencies are still loading or missing.

## Runtime Bridge

`World::from_scene_asset(...)` maps optional direct mesh references into `MeshRenderer.mesh` and primitive mesh/material pairs into `MeshRenderer.primitives`. `World::to_scene_asset(...)` writes both forms back when persistent locators exist. Render extraction keeps `RenderMeshSnapshot` single-mesh: when `MeshRenderer.primitives` is non-empty, extraction emits one ordinary snapshot per primitive with that primitive's mesh and material; when the primitive list is empty, it emits the legacy model/direct-mesh snapshot. Keeping primitive expansion at extract time avoids adding primitive-list behavior to every renderer, Hybrid GI, Virtual Geometry, and submit-path consumer.

During resource streaming, `ResourceStreamer::ensure_scene_resources(...)` prepares any direct snapshot mesh through `ensure_mesh(...)`; if a snapshot does not carry a direct mesh, the existing model preparation path remains the fallback. The draw builder consumes a prepared direct mesh first and otherwise renders the prepared model. The compatibility model handle remains on scene mesh instances while legacy model paths exist, but imported glTF scenes can now bind every primitive to its labeled `MeshAsset` and material without relying on the root model envelope for normal rendering.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/scene.rs` covers TOML roundtrip for core scene, camera, light, physics, and animation fields. The overview tests lock the new read model for populated scenes with camera, model/direct-mesh/material, collider material, light, physics, animation, terrain, tilemap, and prefab references, plus empty-scene behavior. The record-set regression covers stable scene id sorting, list-level totals across populated and empty scene records, projection from scene records into entity rows, stable `(scene_id, entity)` sorting, and entity-row summary totals.

`zircon_runtime/src/scene/tests/asset_scene.rs` covers the runtime bridge from scene asset to world component, render extract, and saved scene asset. The direct-mesh test fixture binds `res://meshes/triangle.zmesh` as the optional direct mesh and asserts the world and render snapshot preserve that mesh handle beside the model and material handles. The primitive-binding test constructs a scene with `SceneMeshPrimitiveBindingAsset`, verifies it becomes `MeshRenderer.primitives`, verifies render extraction emits a direct mesh/material snapshot for that primitive, and verifies `World::to_scene_asset(...)` preserves the binding.

`zircon_runtime/src/asset/tests/assets/importer.rs`, `zircon_runtime/src/asset/tests/assets/gltf_importer.rs`, and `zircon_plugins/gltf_importer/runtime/src/tests.rs` cover imported glTF scene bindings. Triangle, multi-scene, and two-primitive/two-material fixtures now assert that scene entities carry `Mesh{n}/Primitive{p}` plus `Material{m}` primitive bindings and that scene dependencies include every primitive mesh and material label.

`zircon_runtime/src/asset/tests/project/asset_flow_sample.rs` covers the imported glTF scene/entity path in a project scan. It asserts `Scene0` and `Node0` ready records, checks the exact `Scene0 -> Node0/Mesh0/Mesh0/Primitive0/Material0` dependency set, verifies the scene entity carries a primitive binding from `Mesh0/Primitive0` to `Material0`, and verifies the scene plus flattened entity management summaries count one direct primitive mesh reference and one primitive binding. It also loads `Scene0` as a typed `SceneAsset` through `ProjectAssetManager` and verifies root, direct dependency, and recursive dependency load states are all loaded for the full scene graph.

Broader milestone acceptance still needs the full asset/model/importer and renderer validation from the asset gap plan before the overall model/material/mesh/entity/shader management loop is complete.
