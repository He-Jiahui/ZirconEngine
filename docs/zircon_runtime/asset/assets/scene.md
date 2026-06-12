---
related_code:
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/assets/authoring.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/scene/reflect/fixed/mesh_renderer.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/load_asset.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml.zmeta
  - examples/vampire/assets/navigation/main.navmesh.toml
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
implementation_files:
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/scene/components/scene.rs
  - zircon_runtime/src/scene/world/project_io.rs
  - zircon_runtime/src/script/vm/scene_hook.rs
  - zircon_runtime/src/scene/world/render.rs
  - zircon_runtime/src/scene/world/property_access/entries.rs
  - zircon_runtime/src/scene/world/property_access/write.rs
  - zircon_runtime/src/scene/reflect/fixed/mesh_renderer.rs
  - zircon_runtime/src/core/framework/render/scene_extract.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_plugins/animation/runtime/src/sequence.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/asset/tests/assets/scene.rs
  - zircon_runtime/src/scene/tests/asset_scene.rs
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs
  - examples/vampire/assets/scenes/main.scene.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml
  - examples/vampire/assets/terrain/jungle_clearing.terrain.toml.zmeta
  - examples/vampire/assets/navigation/main.navmesh.toml
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
plan_sources:
  - user: 2026-05-30 continue model material mesh entity shader flow and asset management
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - docs/superpowers/specs/2026-06-09-vampire-dark-content-upgrade-design.md
  - docs/superpowers/plans/2026-06-09-vampire-dark-content-upgrade.md
  - user: 2026-06-10 vampire roguelite animation state-machine follow-up
  - user: 2026-06-10 vampire terrain-backed rugged forest, graphical HUD, and in-scene health bars
tests:
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_toml_roundtrip_preserves_entities_and_bindings
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_toml_roundtrip_preserves_post_process_components
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_toml_roundtrip_preserves_physics_and_animation_components
  - zircon_runtime/src/asset/tests/assets/scene/management.rs::scene_asset_overview_reports_entity_component_and_reference_counts
  - zircon_runtime/src/asset/tests/assets/scene/management.rs::scene_asset_overview_handles_empty_scenes
  - zircon_runtime/src/asset/tests/assets/scene/management.rs::scene_asset_management_record_set_sorts_and_summarizes_records
  - zircon_runtime/src/asset/tests/assets/scene/management.rs::scene_asset_management_record_set_sorts_and_summarizes_records (entity record-set assertions)
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_instantiate_world_with_asset_bound_meshes
  - zircon_runtime/src/scene/tests/asset_scene.rs::render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_roundtrip_primitive_mesh_material_bindings
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_keep_transform_only_hierarchy_nodes
  - zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_selects_mesh_lod_by_camera_distance
  - zircon_runtime/src/scene/tests/render_post_process_extract.rs::scene_asset_post_process_settings_feed_render_extract
  - zircon_runtime/src/scene/tests/world_basics.rs::mesh_renderer_sort_fields_feed_geometry_phase_queue
  - zircon_runtime/src/scene/tests/property_paths.rs::world_resolves_entity_paths_and_mutates_component_properties
  - zircon_runtime/src/scene/tests/inspection.rs::world_inspection_builds_hierarchy_and_reflected_fields
  - zircon_plugins/animation/runtime/src/sequence.rs::tests::sequence_applies_mesh_renderer_morph_weight_track
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_emits_gltf_multi_primitive_material_labels
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_emits_bevy_style_gltf_labeled_subassets
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_emits_gltf_multi_scene_labels
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_emits_multi_primitive_material_labels
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_decodes_triangle_gltf_into_model_asset
  - zircon_plugins/gltf_importer/runtime/src/tests.rs::importer_emits_multi_scene_labels
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs::project_manager_imports_minimal_gltf_material_shader_mesh_sample
  - cargo test -p zircon_runtime --lib scene_asset_management_record_set_sorts_and_summarizes_records --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 entity record-set slice: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib scene_asset_overview --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib asset::tests::assets::scene --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 scene record set: passed, 11 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 M6 minimal asset-flow sample with typed facade load-state, primitive binding, and aggregate management assertions: passed, 1 passed, 2211 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-01 M6 minimal asset-flow sample with glTF morph target/default weight and aggregate management assertions: passed, 1 passed, 2303 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::scene --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-test-splits-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 scene test split and TOML-safe joint constraint: passed, 11 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo test -p zircon_runtime --lib scene_asset_toml_roundtrip_preserves_post_process_components --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo test -p zircon_runtime --lib scene_asset_post_process_settings_feed_render_extract --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=E:\cargo-targets\zircon-vampire-runtime: passed 2026-06-09
  - cargo test -p zircon_runtime --lib vampire_example --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-09; verifies the upgraded vampire scene imports, extracts third-person meshes, and carries post-process state
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app cargo test -p zircon_runtime --lib vampire_example --locked --message-format short -- --nocapture --test-threads=1 (2026-06-10 vampire animation state-machine scene binding: passed, 2 passed)
  - cargo test -p zircon_runtime --lib scene_assets_keep_transform_only_hierarchy_nodes --locked --message-format short -- --nocapture (2026-06-10 transform-only hierarchy nodes: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 after terrain-backed jungle update; validates Baked Jungle Terrain has both mesh and terrain components, loads res://terrain/jungle_clearing.terrain.toml, checks 9x9 height samples, material layer, height range, and navmesh vertical variation
  - cargo test -p zircon_runtime --lib vampire_example_manifest_scene_and_scripts_are_importable --locked --message-format short -- --nocapture --test-threads=1 with CARGO_TARGET_DIR=D:\cargo-targets\zircon-vampire-app: passed 2026-06-10 final verification; confirms terrain-backed jungle scene remains importable after HUD/health-bar code split
doc_type: module-detail
---

# Scene Asset

## Purpose

`SceneAsset` is the serialized scene root used by runtime project loading, prefab payloads, and imported scene entries. It owns ordered `SceneEntityAsset` rows. Each entity row can carry camera, mesh/model/material binding, light, physics, animation, terrain, tilemap, and prefab-instance components.

Scene entities can also carry `script_bindings`. A binding stores a VM package name, module name, enabled flag, per-stage `update` / `fixed_update` flags, and arbitrary JSON properties for authoring metadata. Script-only and transform-only hierarchy entities now instantiate as `NodeKind::Empty` instead of being dropped, which gives gameplay scripts, skeleton node trees, and authoring groups stable scene entity ids without requiring a mesh, light, or camera component.

The vampire example player entity demonstrates the scene-level animation authoring path: the same `SceneEntityAsset` row binds imported Kenney vampire primitive meshes, `SceneAnimationSkeletonAsset`, and `SceneAnimationStateMachinePlayerAsset` with initial `moving=false` and `attacking=false` parameters. Those fields are ordinary scene asset data rather than project-side code, so project scanning, dependency reporting, world instantiation, and runtime tick can all see the same state-machine reference.

The vampire example terrain demonstrates the scene-level terrain authoring path. `Baked Jungle Terrain` keeps the visible `res://models/jungle_terrain.model.toml` mesh for the current renderer, but it also carries a terrain component pointing at `res://terrain/jungle_clearing.terrain.toml`. That terrain asset is a 9x9 heightfield with a `moss_mud_floor` material layer, a ready `.zmeta` artifact record, and a matching height-varying baked navmesh. The scene entity is scaled vertically, so the sample is a rugged terrain-backed jungle clearing rather than only a flat prop mesh with foliage decorations.

Scene cameras can carry `post_process_settings`, and entities can carry `post_process_volume`. The authoring payload covers the current stylistic runtime stack used by scene-authored games: bloom, color grading, tonemapping, vignette, film grain, dithering, chromatic aberration, and fog. Volumes expose active/global/priority/weight/blend-distance controls plus optional profile overrides, so a scene file can seed the base camera look and blend global or local mood profiles without runtime-only setup code.

The scene asset format remains authoring-focused. `SceneMeshInstanceAsset.mesh` is an optional direct `MeshAsset` reference beside the compatibility `model` envelope and material binding. `SceneMeshInstanceAsset.primitives` is the multi-primitive form: each `SceneMeshPrimitiveBindingAsset` pairs one direct mesh reference with the material that should shade that primitive. `SceneMeshInstanceAsset.render_queue`, `material_queue`, `order_in_layer`, and `depth_bias` store authored neutral mesh phase-sort overrides, with zero values skipped during serialization. `SceneMeshInstanceAsset.morph_weights` stores authored per-instance morph target weights beside those mesh bindings. `SceneMeshInstanceAsset.lods` stores conventional mesh LOD levels; each `SceneMeshLodLevelAsset` has a `min_distance` plus the same model, optional direct mesh, material, and primitive binding shape as the base mesh instance. Optional mesh, zero sort overrides, primitive, morph-weight, and LOD fields are skipped when absent or empty, so existing `.scene` assets and prefab scene payloads keep loading without serialized churn.

## Direct References

`SceneEntityAsset::direct_references()` is the per-entity dependency collector. It reports references from camera texture targets, mesh model bindings, optional direct mesh bindings, mesh primitive mesh/material pairs, mesh LOD model/direct mesh/material/primitive bindings, the legacy entity material binding, optional collider physics material bindings, animation skeleton/player/sequence/graph/state-machine bindings, terrain and tilemap bindings, and prefab instances. In `examples/vampire`, this means the same scene row can advertise both the renderer-facing jungle mesh references and the gameplay/authoring-facing terrain asset reference.

`SceneAsset::direct_references()` delegates to the entity collector in entity order. This preserves the previous aggregate behavior while making entity-level dependency counts available to asset management and diagnostics. `PrefabAsset::direct_references()` continues to use the scene aggregate, so prefab dependency reporting follows the same rule.

## Management DTOs

`SceneEntityOverview` is a compact read-only row for one scene entity. It carries the stable entity id, name, parent id, active flag, render layer mask, mobility, direct reference count, direct mesh reference count, primitive mesh/material binding count, morph weight count, and component-presence flags for camera, camera post-process settings, mesh, direct mesh reference, light variants, post-process volume, physics components, animation bindings, terrain, tilemap, and prefab instance. Helper methods derive light, physics, and animation binding counts from those flags.

`SceneAssetOverview` aggregates the entity rows into scene-level counts: total entities, active entities, roots, cameras, mesh instances, direct mesh references, primitive mesh/material bindings, morph weights, mesh material bindings, collider material bindings, lights, physics components, animation bindings, terrain bindings, tilemap bindings, prefab instances, and direct references. `SceneAsset::entity_overviews()` returns ordered entity rows, and `SceneAsset::overview()` returns the aggregate view.

`SceneAssetManagementRecord` wraps a `ResourceId` with the scene overview. `SceneAsset::management_record(...)` is the asset-level constructor, and `ResourceStreamer::scene_asset_overview(...)` / `scene_asset_management_record(...)` load the same read model through the runtime asset manager for renderer-side management panels that already work with resource ids.

`SceneAssetManagementRecordSet` sorts scene records by `ResourceId` and carries `SceneAssetManagementRecordSetSummary`. The summary totals scene count, entity count, active/root entities, direct references, camera/mesh/direct-mesh/primitive-binding/morph-weight/material/collider/light/physics/animation counts, and terrain/tilemap/prefab bindings for scene-list headers. `ResourceStreamer::scene_asset_management_records(...)` and `scene_asset_management_record_set(...)` expose that list-level read model without forcing UI callers to scan scene rows themselves.

`SceneEntityManagementRecord` is the cross-scene row form for entity tables. It pairs the owning scene `ResourceId` with one `SceneEntityOverview`, so a panel can list entities directly while still preserving the scene identity needed for selection, navigation, or stale-row repair. `SceneAssetManagementRecord::entity_management_records(...)` projects one scene record into entity rows, `SceneAsset::entity_management_records(...)` constructs those rows directly from the asset, and `SceneEntityManagementRecordSet` sorts rows by `(scene_id, entity)` with a `SceneEntityManagementRecordSetSummary` that mirrors the scene summary counters at entity-row granularity. `ResourceStreamer::scene_entity_management_records(...)` and `scene_entity_management_record_set(...)` expose that flattened read model across all registered scene assets.

These DTOs do not attempt to instantiate ECS entities, resolve handles, or validate referenced assets. They are deliberately derived from the authoring asset only, making them cheap enough for asset browsers and safe to use when dependencies are still loading or missing.

## Runtime Bridge

`World::from_scene_asset(...)` maps optional direct mesh references into `MeshRenderer.mesh`, primitive mesh/material pairs into `MeshRenderer.primitives`, conventional LOD levels into `MeshRenderer.lods`, authored mesh sort overrides into `MeshRenderer.render_queue`, `MeshRenderer.material_queue`, `MeshRenderer.order_in_layer`, and `MeshRenderer.depth_bias`, and authored morph weights into `MeshRenderer.morph_weights`. `World::to_scene_asset(...)` writes those forms back when persistent locators, non-empty LODs, non-zero sort overrides, or non-empty weights exist. `MeshRenderer.render_queue`, `MeshRenderer.material_queue`, `MeshRenderer.order_in_layer`, and `MeshRenderer.depth_bias` are exposed as editable property paths for render ordering; `MeshRenderer.morph_weights.N` is exposed as an animatable scalar component property path, with writes growing the vector with zeroes so animation tracks can target sparse morph indices. Fixed reflection exposes the sort overrides as editable integer/scalar fields and keeps the morph vector and LOD list as read-only inspection data, while property access exposes `MeshRenderer.lod_level_count` as a read-only count.

When `script_bindings` are present, `World::from_scene_asset(...)` stores them as the dynamic component `script.bindings`. The ZrVM language runtime scene hooks read that component and dispatch `onStart`, `onFixedUpdate`, and `onUpdate` exports for each enabled binding whose stage flag allows the current phase. `World::to_scene_asset(...)` decodes the same dynamic component back into authoring data, so script-bound entities round-trip with the rest of the scene.

Rows without mesh, light, camera, post-process volume, physics, animation, or script payload still instantiate as `NodeKind::Empty`. This is intentional scene hierarchy behavior, not dead-row compatibility: imported glTF node trees, authored grouping transforms, and the vampire actor's `Node1:root` skeleton parent must survive `World::from_scene_asset(...)` so child `parent` links can pass hierarchy normalization and runtime animation hooks can find descendants by node name. `World::to_scene_asset(...)` writes these empty nodes back out with their transform and parent fields, preserving hierarchy-only authoring structure.

When camera `post_process_settings` are present, `World::from_scene_asset(...)` maps them into `PostProcessSettingsComponent` on the camera entity. When `post_process_volume` is present, it maps to `PostProcessVolumeComponent`; volume-only entities instantiate as `NodeKind::Empty` so they remain visible to scene traversal and save back out. `World::to_scene_asset(...)` serializes those runtime components back to scene TOML, preserving the effect values used by render extraction.

Render extraction keeps `RenderMeshSnapshot` single-mesh: it first chooses the active base-or-LOD source for the current scene camera by selecting the highest finite `min_distance` LOD whose threshold is within the camera-to-mesh distance. Base selections keep `RenderMeshSnapshot.mesh_lod` empty; conventional LOD selections set `mesh_lod` to `RenderMeshLodSelection { level_index, min_distance }`, where `level_index` is the authored `MeshRenderer.lods` vector index. When the selected source has primitives, extraction emits one ordinary snapshot per primitive with that primitive's mesh, material, LOD metadata, and the renderer's morph weights; when the primitive list is empty, it emits the selected model/direct-mesh snapshot with the same weight vector. The paired `GeometryPhaseInput` row receives `MeshRenderer.render_queue`, `MeshRenderer.material_queue`, `MeshRenderer.order_in_layer`, and `MeshRenderer.depth_bias`, so the neutral render phase sort key can order same-phase meshes without making draw construction inspect scene components. Keeping primitive, LOD, and sort expansion at extract time avoids adding container behavior to every renderer, Hybrid GI, Virtual Geometry, and submit-path consumer, and keeps conventional mesh LOD metadata separate from Virtual Geometry cluster `lod_level`.

During resource streaming, `ResourceStreamer::ensure_scene_resources(...)` prepares any direct snapshot mesh through `ensure_mesh(...)`; if a snapshot does not carry a direct mesh, the existing model preparation path remains the fallback. The draw builder consumes a prepared direct mesh first and otherwise renders the prepared model. The compatibility model handle remains on scene mesh instances while legacy model paths exist, but imported glTF scenes can now bind every primitive to its labeled `MeshAsset` and material without relying on the root model envelope for normal rendering.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/scene.rs` covers TOML roundtrip for core scene, camera, post-process, light, physics, and animation fields. `scene_asset_toml_roundtrip_preserves_post_process_components` asserts camera settings, global volume profiles, TOML field emission, and overview flags. `zircon_runtime/src/asset/tests/assets/scene/management.rs` owns the scene/entity management read-model regressions: populated scenes with camera, model/direct-mesh/material, collider material, light, physics, animation, terrain, tilemap, and prefab references; empty-scene behavior; stable scene id sorting; list-level totals across populated and empty scene records; projection from scene records into entity rows; stable `(scene_id, entity)` sorting; and entity-row summary totals.

`zircon_runtime/src/scene/tests/asset_scene.rs` covers the runtime bridge from scene asset to world component, render extract, and saved scene asset. The direct-mesh test fixture binds `res://meshes/triangle.zmesh` as the optional direct mesh and asserts the world and render snapshot preserve that mesh handle beside the model and material handles. The primitive-binding test constructs a scene with `SceneMeshPrimitiveBindingAsset`, `SceneMeshLodLevelAsset`, authored render queue, material queue, order-in-layer, depth bias, and morph weights, verifies they become `MeshRenderer.primitives`, `MeshRenderer.lods`, `MeshRenderer.render_queue`, `MeshRenderer.material_queue`, `MeshRenderer.order_in_layer`, `MeshRenderer.depth_bias`, and `MeshRenderer.morph_weights`, verifies render extraction emits a direct mesh/material snapshot carrying those weights, and verifies `World::to_scene_asset(...)` preserves the binding, LOD level, sort overrides, and weight vector. `scene_assets_keep_transform_only_hierarchy_nodes` constructs a pure transform parent plus mesh child and verifies `World::from_scene_asset(...)` keeps the empty parent, preserves the child parent link, and saves the hierarchy-only node back out. `zircon_runtime/src/scene/tests/render_post_process_extract.rs::scene_asset_post_process_settings_feed_render_extract` covers the scene-asset-to-world-to-render-extract path for camera post-process settings and a blended global volume. `zircon_runtime/src/scene/tests/render_extract.rs::render_frame_extract_selects_mesh_lod_by_camera_distance` covers near-camera base source selection with empty `mesh_lod` and far-camera LOD source selection with populated `level_index`/`min_distance` metadata. `zircon_runtime/src/scene/tests/world_basics.rs::mesh_renderer_sort_fields_feed_geometry_phase_queue` covers the same-phase phase queue ordering path.

`zircon_runtime/src/scene/tests/property_paths.rs::world_resolves_entity_paths_and_mutates_component_properties` covers the component-property paths for `MeshRenderer.render_queue`, `MeshRenderer.material_queue`, `MeshRenderer.order_in_layer`, `MeshRenderer.depth_bias`, and `MeshRenderer.morph_weights.N`, including no-op order/depth writes, sparse morph growth, and readback. `zircon_runtime/src/scene/tests/inspection.rs::world_inspection_builds_hierarchy_and_reflected_fields` covers the editable reflected sort fields. `zircon_plugins/animation/runtime/src/sequence.rs::tests::sequence_applies_mesh_renderer_morph_weight_track` covers the animation sequence path by applying a scalar track to `MeshRenderer.morph_weights.1`.

`zircon_runtime/src/asset/tests/assets/gltf_importer.rs` and `zircon_plugins/gltf_importer/runtime/src/tests.rs` cover imported glTF scene bindings. Triangle, multi-scene, and two-primitive/two-material fixtures now assert that scene entities carry `Mesh{n}/Primitive{p}` plus `Material{m}` primitive bindings and that scene dependencies include every primitive mesh and material label.

`zircon_runtime/src/asset/tests/project/asset_flow_sample.rs` covers the imported glTF scene/entity path in a project scan. It asserts `Scene0` and `Node0` ready records, checks the exact `Scene0 -> Node0/Mesh0/Mesh0/Primitive0/Material0` dependency set, verifies the scene entity carries a primitive binding from `Mesh0/Primitive0` to `Material0`, preserves the glTF mesh default weight as `SceneMeshInstanceAsset.morph_weights`, and verifies the scene plus flattened entity management summaries count one direct primitive mesh reference, one primitive binding, and one morph weight. It also loads `Scene0` as a typed `SceneAsset` through `ProjectAssetManager` and verifies root, direct dependency, and recursive dependency load states are all loaded for the full scene graph.

`zircon_runtime/src/asset/tests/project/example_vampire.rs` now additionally covers the authored vampire player animation binding and terrain-backed jungle scene. It verifies the player entity carries an animation skeleton, binds `res://animation/vampire_locomotion.state_machine.zranim`, starts with false `moving` and `attacking` parameters, and that the referenced idle/move/attack graph assets plus locomotion state machine decode from the `examples/vampire/assets/animation` directory. The same test also verifies the scene overview reports terrain, `Baked Jungle Terrain` has both mesh and terrain components, `res://terrain/jungle_clearing.terrain.toml` loads with 81 height samples and a material layer, and the baked navmesh keeps enough Y variation to match the rugged terrain instead of a flat decorative floor.

Broader milestone acceptance still needs the full asset/model/importer and renderer validation from the asset gap plan before the overall model/material/mesh/entity/shader management loop is complete.

Current split-module validation on 2026-06-05 used `D:\cargo-targets\zircon-asset-test-splits-0605` and ran `cargo test -p zircon_runtime --lib asset::tests::assets::scene --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture`. All 11 scene tests passed, including the three `scene/management.rs` regressions and the physics/animation scene TOML round-trip that embeds default joint constraints.
