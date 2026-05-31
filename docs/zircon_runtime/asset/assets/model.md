---
related_code:
  - zircon_runtime/src/asset/assets/model/mod.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/asset/assets/model/virtual_geometry.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs
  - zircon_runtime/src/asset/tests/assets/gltf_scene_fixtures.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs
  - zircon_runtime/src/asset/tests/assets/obj_importer.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_plugins/gltf_importer/runtime/src/tests.rs
  - zircon_plugins/gltf_importer/runtime/src/test_fixtures.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
implementation_files:
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/model/primitive.rs
  - zircon_runtime/src/asset/assets/imported.rs
  - zircon_runtime/src/asset/importer/ingest/model_mesh_subassets.rs
  - zircon_runtime/src/asset/importer/ingest/import_gltf.rs
  - zircon_runtime/src/asset/importer/ingest/gltf_labeled_subassets.rs
  - zircon_plugins/asset_importers/model/runtime/src/lib.rs
  - zircon_plugins/obj_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/lib.rs
  - zircon_plugins/gltf_importer/runtime/src/subassets.rs
  - zircon_runtime/src/asset/assets/model/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_ensure_model.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
  - zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs
plan_sources:
  - user: 2026-05-30 continue model material mesh entity shader flow and asset management
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
tests:
  - zircon_runtime/src/asset/tests/assets/model.rs::model_asset_toml_roundtrip_preserves_virtual_geometry_payload
  - zircon_runtime/src/asset/tests/assets/model.rs::model_asset_overview_reports_root_and_primitive_mesh_summary
  - zircon_runtime/src/asset/tests/assets/model.rs::model_asset_overview_handles_empty_model_roots
  - zircon_runtime/src/asset/tests/assets/model.rs::model_asset_management_record_wraps_id_and_overview
  - zircon_runtime/src/asset/tests/assets/model.rs::model_asset_management_record_set_sorts_and_summarizes_records
  - zircon_runtime/src/asset/tests/assets/model.rs::model_asset_direct_references_deduplicate_primitive_meshes
  - zircon_runtime/src/asset/tests/assets/importer.rs::importer_emits_mesh_subassets_for_model_imports
  - zircon_runtime/src/asset/tests/assets/importer.rs::importer_emits_bevy_style_gltf_labeled_subassets
  - zircon_runtime/src/asset/tests/assets/gltf_importer.rs::importer_emits_gltf_multi_scene_labels
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs::project_manager_imports_minimal_gltf_material_shader_mesh_sample
  - cargo test -p zircon_runtime model_asset_overview --locked --jobs 1
  - cargo test -p zircon_runtime --lib asset::tests::assets::model --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 model record set: passed, 5 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib model_asset --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 model mesh-reference slice: passed, 9 passed, 2186 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_emits_mesh_subassets_for_model_imports --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 model mesh-reference slice: passed, 1 passed, 2194 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_emits_bevy_style_gltf_labeled_subassets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 model mesh-reference slice: passed, 1 passed, 2194 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_backfills_virtual_geometry_for_model_toml_without_dropping_base_mesh --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 model mesh-reference slice: passed, 1 passed, 2194 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_preserves_gltf_skinning_channels_on_model_vertices --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 model mesh-reference slice: passed, 1 passed, 2194 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split STL/PLY/DXF plugin mesh-reference slice: passed, 5 passed; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split OBJ plugin mesh-reference slice: passed, 3 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_emits_obj_multi_mesh_subassets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 OBJ multi-mesh labels: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib obj --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 runtime OBJ regression: passed, 6 passed; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime obj_importer_emits_multi_mesh_subassets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split OBJ multi-mesh labels: passed, 1 passed; existing zircon_runtime warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_obj_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split OBJ importer regression: passed, 4 passed plus 0 doc tests; existing zircon_runtime warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_asset_importer_model_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split STL/PLY/DXF M4 gate: passed, 5 passed plus 0 doc tests; existing zircon_runtime warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF plugin mesh-reference slice: passed, 3 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_rejects_unsupported_gltf_primitive_mode --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 glTF primitive mode guard: failed before implementation because `LINES` imported as `TriangleList`; passed after adding the mode guard; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_emits_gltf_multi_primitive_material_labels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 multi-primitive glTF labels: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib importer_emits_gltf_multi_scene_labels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 multi-scene glTF labels: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime importer_rejects_unsupported_gltf_primitive_mode --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF primitive mode guard: failed before implementation because `LINES` imported as `TriangleList`; passed after adding the mode guard; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF primitive mode and multi-primitive label regression: passed, 5 passed; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime importer_emits_multi_scene_labels --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF multi-scene labels: passed, 1 passed; existing zircon_runtime warnings only)
  - cargo test -p zircon_runtime --lib gltf --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 glTF multi-scene/external texture/missing buffer regression: passed, 8 passed; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_gltf_importer_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 split glTF multi-scene/external texture/missing buffer regression: passed, 8 passed plus 0 doc tests; existing zircon_runtime warnings only)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_hybrid_gi_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never (2026-05-31 constructor compatibility check: passed; existing warnings only)
  - cargo check --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime --locked --jobs 1 --target-dir D:\cargo-targets\zircon-plugin-model-mesh-ref-0531 --message-format short --color never (2026-05-31 constructor compatibility check: passed; existing warnings only)
  - cargo test -p zircon_runtime --test virtual_geometry_debug_snapshot_contract --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never --no-run (2026-05-31 constructor compatibility check: passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 M6 minimal asset-flow sample with typed facade load-state assertions: passed, 1 passed, 2205 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test --manifest-path Cargo.toml -p zircon_runtime --lib model_render_primitives_keep_legacy_payload_when_mesh_reference_unresolved --locked --offline --jobs 1 --target-dir D:\cargo-targets\zircon-navigation-runtime-metadata --color never --quiet (2026-05-31 support fix for Navigation metadata validation: passed, 1 passed, 2208 filtered; existing zircon_runtime warnings only)
doc_type: module-detail
---

# Model Asset

## Purpose

`ModelAsset` remains the root model asset consumed by the current renderer path. It still stores `primitives: Vec<ModelPrimitiveAsset>` so existing `ResourceStreamer::ensure_model(...)` can prepare GPU meshes without a hard renderer migration.

`GpuModelResource::from_asset_with_mesh_assets(...)` is the renderer-side compatibility hook for the mesh-subasset migration. Its loader closure is explicitly bounded as `FnMut(&AssetReference) -> Option<MeshAsset>`, and the default `from_asset(...)` path uses a typed `None::<MeshAsset>` fallback so legacy primitive rendering still compiles and keeps the old payload when a labeled mesh subasset is not available.

The asset gap plan requires this legacy root to coexist with first-class `MeshAsset` subassets until renderer mesh-handle consumption is complete. The model module therefore now exposes read-only overview DTOs instead of changing the root payload shape.

## Overview DTOs

`ModelPrimitiveAsset` now has an optional `mesh: AssetReference` field. Importers populate it with the labeled `MeshAsset` subasset that mirrors the legacy primitive payload. The field is optional and skipped during serialization when absent, so old model TOML files continue to load while newly imported model roots can expose their generated mesh locators.

`ModelPrimitiveOverview` is derived from `ModelPrimitiveAsset::render_mesh_descriptor()` and records the primitive index, optional mesh subasset reference, topology, bounds, render mesh kind, 2D/3D suitability, vertex count, index count, rendered primitive count, and whether the primitive carries Virtual Geometry data.

`ModelAssetOverview` is the root summary. `ModelAsset::overview()` returns the model URI, aggregate bounds over all primitive vertices, primitive count, total vertex and index counts, mesh reference count, total rendered primitive count, whether any primitive carries Virtual Geometry data, and the ordered primitive overview rows.

`ModelAssetManagementRecord` wraps a stable `ResourceId` around the root overview. `ModelAsset::management_record(...)` produces this row for asset panels and runtime management surfaces that need to keep model identity separate from the model's authoring URI.

`ModelAssetManagementRecordSet` sorts model rows by `ResourceId` and carries `ModelAssetManagementRecordSetSummary`. The summary aggregates model count, primitive count, total vertices, total indices, rendered primitive count, mesh-referenced model count, mesh reference count, and Virtual Geometry model/primitive totals for model-list headers.

`ModelAsset::direct_references()` returns the unique primitive mesh references. `ImportedAsset::Model` now forwards those references, so generic asset dependency readers can see the root model to labeled mesh-subasset edge from the asset payload as well as from importer metadata.

The overview is intentionally read-only. It does not replace `MeshAssetOverview`, and it does not remove `ModelAsset.primitives`. Asset browsers, importer diagnostics, and editor/runtime management panels can use it to inspect model roots while `MeshAsset` subassets continue to represent the future typed mesh-handle target.

## Compatibility

The current compatibility phase has three layers:

- Root model import still emits `ModelAsset { primitives }` for renderer compatibility.
- Model import also emits labeled `MeshAsset` subassets such as `Mesh0/Primitive0` for typed asset management and future renderer migration, and each root primitive records that generated mesh subasset in `ModelPrimitiveAsset.mesh`.
- Renderer model preparation now resolves those primitive mesh references through the project registry and prefers a loaded `MeshAsset` payload when it can be converted to the current GPU primitive shape. Unresolved, load-failing, or conversion-failing mesh references keep the legacy primitive payload as the compatibility fallback.
- Multi-object OBJ imports keep one root primitive and one `Mesh{n}/Primitive0` mesh subasset per parsed OBJ object.
- glTF mesh model subassets such as `Mesh0` also carry primitive mesh references that point at `Mesh0/Primitive0`, keeping the root glTF model, mesh model, and primitive mesh subasset graph inspectable.
- Multi-primitive glTF meshes keep one root primitive and one `Mesh{n}/Primitive{p}` mesh subasset per glTF primitive, and the `Mesh{n}` model subasset records all primitive mesh references and material dependencies.
- Multi-scene glTF imports emit separate `Scene{n}` subassets whose dependency lists stay scoped to each scene's root nodes while scene entities bind back to the shared `Mesh{n}` and material labels.
- Split plugin importers mirror the runtime importer contract: OBJ, STL, PLY, DXF, and glTF plugin imports keep the legacy root payload while assigning each root primitive to the generated labeled mesh subasset.
- glTF import only accepts triangle-list mesh primitives in this compatibility path. Unsupported primitive modes such as lines fail with a parse diagnostic instead of producing a misleading `TriangleList` mesh asset.
- glTF external texture URIs emit decoded `Texture{n}` subassets and material texture locators, while missing external buffers fail before model emission with a diagnostic that names the missing URI and buffer index.
- `ModelAssetOverview` provides a stable root-level management view that can summarize both empty diagnostic roots and populated legacy roots without requiring a GPU device.
- `ResourceStreamer::model_asset_overview(...)`, `model_asset_management_record(...)`, `model_asset_management_records(...)`, and `model_asset_management_record_set(...)` expose the same read model from the renderer resource cache or asset manager. `prepared_model_asset_management_records(...)` still lists already prepared model roots without forcing a project-wide asset scan.

Empty model roots produce zero counts, default bounds, and no primitive rows. This keeps diagnostic-only importer outputs representable without forcing fake geometry.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/model.rs` covers TOML roundtrip with Virtual Geometry and mesh-reference payloads, non-empty root overviews with aggregate and per-primitive counts, mesh reference counts, Virtual Geometry presence, planar versus spatial primitive classification, empty model roots, management-record wrapping of model identity plus overview data, record-set sorting/list-level summary totals, and direct-reference deduplication.

`zircon_runtime/src/asset/tests/assets/importer.rs` verifies that OBJ/model TOML import assigns generated `Mesh0/Primitive0` references to the root model primitive, and that glTF import assigns Bevy-style `Mesh0/Primitive0` references to both the root glTF model and the `Mesh0` model subasset. `zircon_runtime/src/asset/tests/assets/obj_importer.rs` keeps newer OBJ acceptance cases out of the oversized generic importer test and locks a two-object OBJ file to two root primitives, two mesh subassets, and preserved virtual geometry. The same glTF sweep now covers multi-primitive material labels, external texture image decoding, missing external buffer diagnostics, skinning channel preservation, and unsupported primitive-mode rejection. `zircon_runtime/src/asset/tests/assets/gltf_importer.rs` keeps newer glTF acceptance cases out of the oversized generic importer test and currently locks multi-scene `Scene0`/`Scene1` label isolation plus scene entity mesh/material bindings.

`zircon_runtime/src/asset/tests/project/asset_flow_sample.rs` extends those focused importer checks into a minimal project flow: the root glTF model, `Mesh0` model subasset, `Mesh0/Primitive0` mesh subasset, `Node0`, and `Scene0` all register as ready records, and the root/model management summaries prove the primitive mesh reference, vertex count, index count, and scene mesh/material binding survive project scan/import. The sample also opens the project through `ProjectAssetManager` and asserts the root glTF `ModelAsset` typed handle reports root, direct dependency, and recursive dependency load states as loaded.

`zircon_runtime/src/graphics/scene/resources/gpu_model/gpu_model_resource_from_asset.rs` covers the renderer-side selection boundary. `model_render_primitives_use_referenced_mesh_asset_payload_when_available` proves a referenced `MeshAsset` overrides the legacy primitive payload before GPU upload, while `model_render_primitives_keep_legacy_payload_when_mesh_reference_unresolved` locks the compatibility fallback for older or degraded model roots.

The split plugin tests in `zircon_plugins/asset_importers/model/runtime/src/lib.rs`, `zircon_plugins/obj_importer/runtime/src/lib.rs`, and `zircon_plugins/gltf_importer/runtime/src/tests.rs` verify the same root primitive mesh-reference contract for STL/PLY/DXF, OBJ, and glTF plugin imports. The OBJ plugin now mirrors the runtime multi-object acceptance fixture, and the glTF plugin mirrors the runtime multi-scene acceptance fixture. The glTF tests are kept in a separate module so registration/import logic stays below the large-file threshold while fixture writers live in `runtime/src/test_fixtures.rs`.

Milestone validation should still run the broader asset/model and renderer checks from the asset gap plan before M6 is accepted; the overview tests are a focused regression for the new management read model.
