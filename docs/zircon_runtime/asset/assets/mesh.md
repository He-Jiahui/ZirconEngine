---
related_code:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/attribute.rs
  - zircon_runtime/src/asset/assets/mesh/constants.rs
  - zircon_runtime/src/asset/assets/mesh/indices.rs
  - zircon_runtime/src/asset/assets/mesh/metadata.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/mesh/normals.rs
  - zircon_runtime/src/asset/assets/mesh/tangents.rs
  - zircon_runtime/src/asset/assets/mesh/usage.rs
  - zircon_runtime/src/asset/assets/mesh/validation.rs
  - zircon_runtime/src/asset/assets/mesh/zmesh_document.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/import_model.rs
  - zircon_runtime/src/asset/importer/ingest/import_obj.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
implementation_files:
  - zircon_runtime_interface/src/resource/marker.rs
  - zircon_runtime_interface/src/resource/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/attribute.rs
  - zircon_runtime/src/asset/assets/mesh/constants.rs
  - zircon_runtime/src/asset/assets/mesh/indices.rs
  - zircon_runtime/src/asset/assets/mesh/metadata.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/mesh/normals.rs
  - zircon_runtime/src/asset/assets/mesh/tangents.rs
  - zircon_runtime/src/asset/assets/mesh/usage.rs
  - zircon_runtime/src/asset/assets/mesh/validation.rs
  - zircon_runtime/src/asset/assets/mesh/zmesh_document.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_scene_resources.rs
  - zircon_runtime/src/graphics/scene/resources/prepared/prepared_mesh.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_mesh/gpu_mesh_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/extend_pending_draws_for_mesh_instance.rs
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/import_mesh.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
plan_sources:
  - user: 2026-05-20 implement ZirconEngine 资产、Texture、模型、ZShader/ZMaterial/ZMesh 缺口补齐计划
  - dev/bevy/crates/bevy_mesh/src/mesh.rs
  - dev/bevy/crates/bevy_gltf/src/label.rs
tests:
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/assets/mesh/normal_generation.rs
  - zircon_runtime/src/asset/tests/assets/mesh/tangent_generation.rs
  - zircon_runtime/src/asset/tests/assets/importer.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs::project_manager_imports_minimal_gltf_material_shader_mesh_sample
  - zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs::tests::skin_mesh_asset_primitive_converts_direct_mesh_attributes_before_skinning
  - cargo test -p zircon_runtime --lib skin_ --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 direct mesh CPU skinning parity: passed, 3 passed, 2211 filtered; existing zircon_runtime lib-test warnings only)
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_instantiate_world_with_asset_bound_meshes
  - zircon_runtime/src/scene/tests/asset_scene.rs::render_extract_keeps_asset_bound_meshes_without_editor_selection_overlay
  - zircon_runtime/src/scene/tests/asset_scene.rs::scene_assets_roundtrip_primitive_mesh_material_bindings
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - cargo test -p zircon_runtime --lib mesh_asset_rejects_builtin_attribute_format_mismatch --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-attribute-format-0529 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-29 root built-in attribute format regression: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-attribute-format-0529 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-29 mesh module regression: passed, 11 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_bounds_can_be_read_without_render_descriptor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-bounds-0529 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-29 direct mesh bounds regression: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-bounds-0529 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-29 mesh module regression after bounds accessor: passed, 12 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_try_render_descriptor_reports_validation_errors --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-try-descriptor-0529 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-29 strict mesh descriptor validation regression: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-try-descriptor-0529 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-29 mesh module regression after strict descriptor accessor: passed, 13 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_rejects_incomplete_list_topology_elements --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-topology-0529 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-29 fixed-list topology element-count regression: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-topology-0529 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-29 mesh module regression after topology validation: passed, 14 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_reports_index_format_without_expanding_indices --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh index-format read model: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh module regression after index-format read model: passed, 15 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_reports_draw_element_and_primitive_counts_without_descriptor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-counts-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 cold target attempted twice during active external compile lanes; timed out before emitting a Cargo test result)
  - cargo test -p zircon_runtime --lib mesh_asset_reports_draw_element_and_primitive_counts_without_descriptor --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh count read model: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh module regression after count read model: passed, 16 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_reports_attribute_summaries_without_value_inspection --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh attribute summary read model: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_reports_morph_target_attribute_summaries --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh morph target attribute summary read model: passed, 1 passed; package-cache lock wait and existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh module regression after attribute summary read model: passed, 18 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_overview_reports_editor_ready_mesh_summary --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh overview read model: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_management_record --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh management record: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh module regression after overview read model: passed, 19 passed; package-cache lock waits and existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_management_record_set --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 mesh management record set: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 mesh management record set: passed, 31 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 M6 minimal asset-flow sample with typed facade load-state assertions: passed, 1 passed, 2205 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_generates_missing_flat_normals_for_unindexed_triangle_list --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 flat normal generation focused regression: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh module regression after flat normal generation: passed, 22 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_generates_missing_smooth_normals_for_indexed_triangle_list --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 smooth normal generation focused regression: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh module regression after smooth/default normal generation: passed, 25 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib mesh_asset_generates_missing_tangents_for_indexed_triangle_list --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 tangent generation focused regression: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset::tests::assets::mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-30 mesh module regression after tangent generation: passed, 29 passed; existing zircon_runtime lib-test warnings only)
  - cargo check -p zircon_runtime --locked --all-targets
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_asset_importer_model_runtime --locked --jobs 1
doc_type: module-detail
---

# Mesh Asset And ZMesh

## Purpose

`MeshAsset` makes mesh payloads a first-class runtime asset instead of only embedding mesh data inside `ModelAsset.primitives`. This is the M2 foundation for the Bevy-style asset plan: typed `Handle<MeshAsset>`, `Assets<MeshAsset>`, `ImportedAsset::Mesh`, `AssetKind::Mesh`, artifact storage, project loading, and resource registration all use the same facade path as texture, shader, material, and model assets.

The authoring format is `.zmesh`, represented by `ZMeshDocument`. The runtime type remains `MeshAsset`, matching the existing naming split between `MaterialAsset`/`ZMaterialDocument` and `ShaderAsset`/`ZShaderDocument`.

## Behavior Model

`MeshAsset` stores a render topology, a named attribute map, optional `u16` or `u32` indices, Bevy-style main-world/render-world residency intent, morph target metadata, optional skin inverse bind matrices, and optional Virtual Geometry payload. The built-in attribute names are `position`, `normal`, `tangent`, `uv0`, `color`, `joint_index`, and `joint_weight`. Positions are required and must be `float32x3`. Present root built-ins are also format-checked at the asset boundary: `normal` is `float32x3`, `tangent` is `float32x4`, `uv0` is `float32x2`, `color` is `float32x4`, `joint_index` is `uint16x4`, and `joint_weight` is `float32x4`. Custom root attributes may use any supported `MeshAttributeValues` format as long as their lengths match the vertex count. `MeshAttributeFormat` is the stable enum for these value formats, and `MeshAttributeSummary` reports an attribute's name, format, row count, and whether it is one of the built-in names. `MeshAsset::attribute_summaries()` returns those root rows in the same deterministic order as the attribute map, so editor diagnostics and importer reports do not need to inspect every `MeshAttributeValues` variant. Every present root attribute must have the same vertex count. When an index buffer is present, its highest index must reference an existing vertex so malformed `.zmesh` documents and importer outputs fail before render descriptor projection. `MeshAsset::index_format()` exposes whether the preserved index buffer is `u16`, `u32`, or absent without expanding index data. `MeshAsset::draw_element_count()` exposes the render draw element count directly, using the index count when an index buffer exists and otherwise using the position vertex count. `MeshAsset::render_primitive_count()` applies the topology primitive-count rule to that element count and reports incomplete fixed-list topology errors instead of requiring callers to create a full descriptor only to read a count. Fixed-size list topologies are also checked at the same boundary: `TriangleList` requires a multiple of three elements and `LineList` requires a multiple of two elements, using the index count when indices exist and the vertex count otherwise. Strip and point topologies keep their existing compatibility behavior.

Morph targets are stored as named attribute maps on `MeshMorphTargetAsset`. Each morph target attribute must match the root mesh vertex count, but the target is not required to redeclare every root attribute and does not reuse the root built-in format table. This is intentional because glTF morph tangent data is a three-component displacement delta, while the root `tangent` attribute is a four-component tangent plus handedness row. `MeshMorphTargetAttributeSummary` adds target index and optional target name around the same attribute summary row, and `MeshAsset::morph_target_attribute_summaries()` exposes those rows in target order with deterministic per-target attribute ordering. The authoring format stays ready for position/normal/tangent deltas without forcing current importers to synthesize unused channels. `MeshSkinAsset.inverse_bind_matrices` stores skin bind-pose matrices beside the mesh payload so future glTF skin labels and renderer skinning paths can reference the same mesh artifact instead of creating a parallel skin metadata store. The representation follows the core idea in Bevy's `Mesh` attribute map while keeping Zircon's current `RenderMeshDescriptor` and `MeshVertex` compatibility layer.

`.zmesh` TOML imports through `zircon.builtin.zmesh`. A valid document becomes `ImportedAsset::Mesh`; invalid position data or mismatched attribute lengths fail during import with a deterministic parse diagnostic. `MeshAsset::bounds()` computes the render-facing bounding box, center, and radius directly from root positions without requiring callers to allocate or inspect a full render descriptor. The count accessors follow the same direct-read pattern for editor diagnostics, importer reports, and future asset browser rows. `MeshAsset::overview()` is the compact strict read model for those consumers: it validates and then returns `MeshAssetOverview` with URI, topology, bounds, vertex/index/draw/primitive counts, index format, root and morph target attribute summaries, morph target count, skin inverse-bind count, Virtual Geometry presence, and asset usage in one payload. `MeshAssetManagementRecord` wraps a stable `ResourceId` around that strict overview, and `MeshAsset::management_record(...)` preserves validation failure semantics by returning the same `MeshValidationError` when the mesh cannot produce an overview. `MeshAssetManagementRecordSet` carries valid records plus `MeshAssetManagementRecordFailure` rows for loaded meshes that fail strict overview validation; failures keep the mesh id and a display diagnostic string instead of inventing a partial overview. `MeshAssetManagementRecordSetSummary` totals valid/invalid mesh counts, vertex/index/draw/render primitive counts, attribute and morph-target counts, skin inverse-bind rows, and Virtual Geometry mesh count for mesh-list headers. `ResourceStreamer::mesh_asset_overview(...)`, `mesh_asset_management_record(...)`, and `mesh_asset_management_record_set(...)` expose the same load-and-validate path through `ProjectAssetManager`, returning `None` only when a single mesh asset cannot be loaded and preserving strict overview results or failure rows for loaded meshes. `MeshAsset::render_mesh_descriptor()` keeps the legacy compatibility behavior and falls back to an empty position slice when required position data is invalid. `MeshAsset::try_render_mesh_descriptor()` is the strict projection path: it runs full mesh validation and returns `MeshValidationError` before producing the same descriptor shape. Both descriptor paths compute bounds, primitive kind, 2D/3D suitability, vertex/index counts, primitive count, and Virtual Geometry presence from the attribute map and indices.

## Model Subassets

Model import paths now keep the old root `ModelAsset { primitives }` payload for renderer compatibility while also emitting labeled `MeshAsset` subassets. The current labels are `Mesh{n}/Primitive0`, so a single OBJ triangle imported from `res://models/triangle.obj` also produces `res://models/triangle.obj#Mesh0/Primitive0`. The root import entry records those subasset locators as dependencies, and each subasset has its own artifact and `ResourceRecord`.

This is intentionally still a compatibility phase, but renderer preparation now consumes the typed mesh target when it is available. `ResourceStreamer::ensure_model(...)` resolves each `ModelPrimitiveAsset.mesh` locator through the project registry and passes a loader into `GpuModelResource::from_asset_with_mesh_assets(...)`; the GPU model builder converts a loaded `MeshAsset` back into the current `ModelPrimitiveAsset` upload shape before creating `GpuMeshResource`s. Scene extraction can carry an optional direct `MeshMarker` handle beside the compatibility model handle, and it can expand `SceneMeshPrimitiveBindingAsset` rows into one direct mesh/material render snapshot per primitive. `ResourceStreamer::ensure_mesh(...)` prepares those direct meshes into `PreparedMesh`, `ensure_scene_resources(...)` uses the direct mesh path for snapshots that carry one, and mesh draw construction first checks whether the direct mesh instance has an extracted animation pose. When it does, the prepared `MeshAsset` is converted through `skin_mesh_asset_primitive(...)` so its joint index/weight attributes feed the same CPU skinning helper used by legacy model primitives; unresolved skeletons or conversion failures fall back to the prepared static direct mesh, and missing direct meshes still fall back to the prepared model. If a model primitive reference is unresolved, the mesh asset cannot load, or strict conversion fails, model preparation falls back to the legacy primitive embedded in the root model. Built-in and plugin glTF import now emit Bevy-style labels for scenes, nodes, meshes, mesh primitives, materials, textures, and diagnostic animation/skin placeholders. glTF scene entities bind every primitive to its labeled `MeshAsset` and matching material through scene primitive bindings, while retaining the root model and first material fields for compatibility. glTF primitive morph target position, normal, and tangent displacements are copied into `MeshAsset.morph_targets` on each primitive subasset. When a glTF node binds a mesh to a skin with inverse bind matrices, both import paths attach those matrices to the corresponding `MeshAsset.skin` on the primitive subassets. The standalone `Skin{n}` labels still remain diagnostic `DataAsset` placeholders until full skin asset payloads are introduced.

## Design And Rationale

The mesh module is folder-backed because mesh data already has separate concerns: attribute values, index format, validation, authoring document conversion, render descriptor projection, and legacy model primitive conversion. `mod.rs` only re-exports the public surface.

`MeshAsset::from_model_primitive()` is the shared support layer for current model importers. OBJ, glTF, STL, PLY, DXF, and `.model.toml` imports all preserve the legacy primitive and derive a matching `MeshAsset` from the same data, avoiding importer-specific render descriptor construction. The conversion preserves the existing joint index/weight vertex channels, but leaves `morph_targets` empty and `skin` unset because the legacy `ModelPrimitiveAsset` does not own morph target or inverse-bindpose metadata. The built-in and plugin glTF importers layer primitive morph target maps and node-level skin data onto the derived mesh subasset after this shared conversion, keeping the common path simple while preserving glTF metadata where it exists.

`MeshAsset::try_generate_missing_normals()` is the Bevy-style default authoring/import support step for the M5 missing-normal gap. It validates the mesh, returns `Ok(false)` when a valid root `normal` attribute already exists, and otherwise writes a root `float32x3` `normal` attribute. Unindexed `TriangleList` meshes use flat normals through `try_generate_missing_flat_normals()`, duplicating each triangle normal across its three vertices. Indexed `TriangleList` meshes use `try_generate_missing_smooth_normals()`, accumulating angle-weighted triangle normals per shared vertex and normalizing the final sums. Degenerate triangles follow Bevy's `normalize_or_zero` behavior and contribute zero normals or zero corner weights, which keeps importer repair deterministic without inventing arbitrary fallback directions. Non-triangle-list topologies, indexed flat-normal requests, and unindexed smooth-normal requests return structured `MeshValidationError` variants so callers can choose whether to unindex, preserve source data, or report an authoring diagnostic.

`MeshAsset::try_generate_missing_tangents()` is the explicit tangent support step for meshes that already have root `position`, `normal`, and `uv0` attributes. It validates the mesh, returns `Ok(false)` when a valid root `tangent` attribute already exists, and otherwise writes `float32x4` tangent rows. The current implementation follows the same Bevy input contract and output shape but uses Zircon's deterministic UV derivative accumulation and Gram-Schmidt orthogonalization instead of adding a new MikkTSpace dependency to the locked workspace. It supports indexed and unindexed `TriangleList` meshes, accumulates shared-vertex tangents when indices exist, preserves handedness in the fourth component, and falls back to a stable orthogonal tangent when a triangle's UVs are degenerate. Missing normal or uv0 attributes and unsupported topologies return structured `MeshValidationError` variants.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/mesh.rs` covers `.zmesh` roundtrip, morph target and skin inverse-bindpose persistence, missing position, root built-in attribute format mismatch, custom attribute format allowance, root and morph target attribute length mismatch, root and morph target attribute summary rows, editor-ready overview rows, management-record wrapping of mesh identity plus strict overview data, record-set summary/failure rows for mixed valid and invalid meshes, flat normal generation for unindexed triangle lists, smooth normal generation for indexed triangle lists, Bevy-style default normal generation dispatch by index presence, tangent generation for indexed and unindexed triangle lists, no-overwrite behavior for existing normals and tangents, structured rejection for unsupported normal/tangent generation requests, out-of-range index rejection, incomplete list topology element rejection, u16/u32 index format reporting, direct draw element and render primitive count reads, u16/u32 index conversion, direct bounds reads, strict descriptor validation, compatibility descriptor projection, bounds/descriptor projection, Virtual Geometry preservation, model primitive conversion, and default `.zmesh` importer routing.

`zircon_runtime/src/asset/tests/assets/importer.rs` covers mesh subasset emission for model imports, Bevy-style glTF labeled subassets, and propagation of glTF morph target position deltas plus inverse bind matrices into `MeshAsset`. The OBJ, glTF, and model importer plugin package tests also assert that a model import outcome includes a mesh subasset entry, and the glTF plugin test mirrors the built-in morph target and inverse-bind matrix assertions.

`zircon_runtime/src/graphics/scene/scene_renderer/mesh/build_mesh_draws/build/skinning.rs` covers CPU skinning for both legacy `ModelPrimitiveAsset` payloads and direct `MeshAsset` payloads converted through the shared `MeshAsset::to_model_primitive()` bridge, locking that direct mesh joint attributes produce the same skinned vertex and normal as the model path.

`zircon_runtime/src/asset/tests/project/asset_flow_sample.rs` covers the M6 project-level mesh path: a scanned glTF scene emits `Mesh0/Primitive0` as a ready `MeshAsset`, the root and mesh model assets point at that subasset, and the mesh management record set reports one valid mesh with three vertices and three indices. The same sample then loads the primitive mesh through `ProjectAssetManager::load<MeshAsset>(...)` and checks root, direct dependency, and recursive dependency load states through the typed facade.

## Open Issues

Texture container GPU upload, full MikkTSpace tangent parity, glTF morph weights/animation playback, full animation/skin subasset payloads, multiple skin bindings per shared mesh, removing the remaining root model compatibility envelope after renderer/plugin consumers no longer need it, and GPU upload of richer mesh-only channels such as morph targets remain later milestones from the 2026-05-20 plan.
