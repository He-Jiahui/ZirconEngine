---
related_code:
  - zircon_runtime/src/asset/management.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/model/model_asset.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/assets/shader/readiness.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs
  - zircon_runtime/src/core/framework/render/material/management/record_set.rs
  - zircon_runtime/src/core/framework/render/material/management/record_summary.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
implementation_files:
  - zircon_runtime/src/asset/management.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
plan_sources:
  - user: 2026-05-30 continue model material mesh entity shader flow and asset management
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
tests:
  - zircon_runtime/src/asset/tests/assets/scene.rs::scene_asset_management_record_set_sorts_and_summarizes_records (entity record-set assertions)
  - zircon_runtime/src/asset/tests/assets/material.rs::material_asset_management_record_set_sorts_and_summarizes_records
  - zircon_runtime/src/asset/tests/assets/model.rs::model_asset_management_record_set_sorts_and_summarizes_records
  - zircon_runtime/src/asset/tests/assets/management.rs::asset_management_record_sets_summarize_asset_family_lists
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs::project_manager_imports_minimal_gltf_material_shader_mesh_sample
  - cargo test -p zircon_runtime --lib scene_asset_management_record_set_sorts_and_summarizes_records --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 entity record-set slice: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset_management_record_sets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 aggregate entity family counters: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib material_asset_management_record_set --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 material asset-management record set: passed after retrying the first Cargo wrapper timeout; 1 passed, existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset_management_record_sets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 aggregate asset-management material-asset counters: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset_management_record_sets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 aggregate asset-management record set: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset_management_record_sets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 aggregate asset-family summaries: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset_management_record_sets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 aggregate asset-family status index: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset_management_record_sets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 compact asset-management overview: passed, 1 passed; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib asset_management_record_sets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 model mesh-reference aggregate counters: passed, 1 passed, 2194 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-05-31 M6 minimal asset-flow sample with typed facade load-state, primitive binding, and aggregate management assertions: passed, 1 passed, 2211 filtered; existing zircon_runtime lib-test warnings only)
doc_type: module-detail
---

# Asset Management Record Sets

## Purpose

`zircon_runtime::asset::management` owns the cross-asset read model for asset-management panels that need either a compact top-level overview or a single full payload rather than separate model, mesh, scene, entity, material, and shader queries. It does not replace the per-family record sets. It composes them and adds a small header summary plus per-family status rows that can drive top-level badges, tabs, and counters.

The module is intentionally neutral. It has no loading, validation, renderer state mutation, or UI behavior. The per-family modules remain responsible for their own record shapes:

- `ModelAssetManagementRecordSet` summarizes root model primitives, generated mesh-subasset references, geometry counts, and Virtual Geometry presence.
- `MeshAssetManagementRecordSet` keeps valid mesh rows separate from invalid loaded-mesh failure rows.
- `SceneAssetManagementRecordSet` summarizes scene/entity/component binding counts.
- `SceneEntityManagementRecordSet` flattens scene entity overviews into stable `scene_id + entity` rows.
- `MaterialAssetManagementRecordSet` summarizes registered `.zmaterial` asset rows before renderer preparation.
- `RenderMaterialManagementRecordSet` remains the prepared material list/detail/query surface in `core::framework::render::material::management`.
- `ShaderAssetManagementRecordSet` summarizes shader readiness, WGSL availability, dependency/import diagnostics, and pipeline-layout counts.

## Aggregate Summary

`AssetManagementRecordSetSummary` contains only cross-family totals:

- `managed_record_count` is the sum of model, mesh, scene, scene entity, material asset, and shader list counts.
- `degraded_record_count` is the sum of invalid mesh rows, material assets with asset-level issues, and shader rows that are not ready.
- The remaining fields mirror the top-level counters from each family: model count and model mesh-reference counts, mesh valid/invalid counts, scene counts, entity component/reference counts, material asset ready/degraded/issue-row and authoring counts, prepared renderer material counts, and shader ready/not-ready/validation-diagnostic counts.

Mesh invalid rows contribute to both `mesh_count` and `degraded_record_count`, because they are loaded assets that can be displayed but cannot provide a strict `MeshAssetOverview`. Material degradation at the cross-family level uses `MaterialAssetManagementRecordSetSummary::degraded_count()`, which counts registered material assets that have material-local validation errors or importer/authoring diagnostics. Prepared renderer material degradation is still preserved under the `prepared_material_*` summary fields, using `RenderMaterialManagementRecordSummary::degraded_count()`, but it is not double-counted as a second top-level material asset. Shader degradation uses the shader record-set summary's `not_ready_count`.

`AssetManagementOverview` is the lightweight carrier for this summary plus the family rows and status buckets. It intentionally omits the per-family detail record sets, so headers, navigation chrome, and quick badge reads can use it without serializing model, mesh, scene, entity, material, or shader detail rows. `AssetManagementOverview::from_summary(...)` derives family rows and the status index from a precomputed summary, and `AssetManagementRecordSets::overview()` projects the same compact state from the full aggregate payload.

## Family Overview Rows

`AssetManagementFamilySummary` is the compact row used by a top-level asset-management overview. Each row has an `AssetManagementFamilyKind`, an `AssetManagementFamilyStatus`, and four count columns:

- `total_record_count` is the number of records in that family.
- `ready_record_count` is the number of family records that are ready or directly inspectable.
- `degraded_record_count` is the number of family records that need attention.
- `issue_row_count` is the issue-style row count for families that expose it.

The family row order is stable: model, mesh, scene, entity, material, shader. Models, scenes, and scene entities currently have no degraded state in this aggregate because their family record sets only expose inspectable rows. Model summary fields still preserve mesh-reference totals so headers can show how many model roots have been assetized into labeled mesh subassets. Mesh rows are degraded by invalid loaded mesh count, and each invalid mesh also contributes one issue row. Entity rows count flattened `scene_id + entity` records and are ready when the source scene asset can produce an overview. Material rows use asset-level material counts: ready rows are registered `.zmaterial` assets without local validation or diagnostic rows, degraded rows are material assets with those issue rows, and issue rows are the sum of material validation errors and material-local diagnostics. Shader rows use ready/not-ready counts, while issue rows currently reflect validation diagnostic rows.

`AssetManagementFamilyStatusIndex` derives stable status buckets from those rows: `empty`, `ready`, and `degraded`. It also provides helpers for total family count, degraded family count, a degraded-family presence check, and a `families_for_status(...)` lookup. The index is not a replacement for per-family detail records; it is the top-level navigation and badge primitive.

`AssetManagementRecordSets` carries these rows in `families` and the derived status buckets in `family_status_index`. `AssetManagementRecordSets::family_summaries()`, `family_status_index()`, and `overview()` expose compact views for callers that already hold the aggregate payload. `ResourceStreamer::asset_management_overview()`, `asset_management_family_summaries()`, and `asset_management_family_status_index()` provide narrow accessors for callers that only need top-level overview state and not the full record-set payload.

## Streamer Surface

`ResourceStreamer::asset_management_record_sets()` builds the aggregate by calling the already-scoped per-family accessors:

- `model_asset_management_record_set()`
- `mesh_asset_management_record_set()`
- `scene_asset_management_record_set()`
- `scene_entity_management_record_set()`
- `material_asset_management_record_set()`
- `material_management_record_set()`
- `shader_asset_management_record_set()`

This keeps the combined read model deterministic without introducing another registry scan policy. Model, mesh, scene, material asset, and shader rows use `ResourceManager` ids by `ResourceKind`. Entity rows are derived from loaded scene assets and keep the owning `scene_id` beside the authoring entity id. The prepared renderer material record set is carried alongside the material asset record set for panels that need readiness/status/query detail after preparation, but the top-level Material family now represents registered `.zmaterial` assets rather than only already-prepared materials.

`ResourceStreamer::asset_management_overview()`, `asset_management_family_summaries()`, and `asset_management_family_status_index()` derive the same family overview state from the aggregate payload. They are convenience surfaces for management headers and navigation chrome that should not load or serialize all detail records.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/material.rs` covers `MaterialAssetOverview`, `MaterialAssetManagementRecord`, and `MaterialAssetManagementRecordSet` by checking stable id sorting, shader/reference counts, authored texture slot counts, fallback slot counts, diagnostics, ready/degraded summary counts, and direct reference totals.

`zircon_runtime/src/asset/tests/assets/scene.rs` covers `SceneEntityManagementRecordSet` by projecting a scene management record into flattened entity rows and checking stable `(scene_id, entity)` sorting plus entity-row summary totals.

`zircon_runtime/src/asset/tests/assets/model.rs` covers the model record-set summary that feeds this aggregate, including mesh-referenced model totals and primitive mesh-reference totals.

`zircon_runtime/src/asset/tests/assets/management.rs` constructs representative per-family record sets and verifies that `AssetManagementRecordSets::from_record_sets(...)` preserves the nested payloads while deriving the combined managed/degraded counts, model mesh-reference totals, entity totals, material asset issue-row totals, prepared material counters, stable family row order, family statuses, ready/degraded counts, issue-row counts, ready/degraded family status buckets, and compact overview projection from the family summaries.

`zircon_runtime/src/asset/tests/project/asset_flow_sample.rs` is the first project-level cross-family management sample. It loads a glTF scene, root model, mesh model, primitive mesh, scene entity, imported glTF material, authored `.zmaterial`, shader package, and DDS texture, then asserts the per-family management summaries for scene/entity primitive bindings, model mesh references, mesh vertex/index counts, material slot/fallback counts, and texture upload fallback. The sample also derives a compact `AssetManagementRecordSets` payload from those family rows and verifies the aggregate entity direct-mesh and primitive-binding counters, so the top-level management summary matches the imported scene graph. The same sample opens the generated project through `ProjectAssetManager` and checks typed handles plus direct and recursive load states for scene, model, mesh, material, shader, and texture assets, so management rows and facade residency state are verified from one import graph.

Milestone acceptance still requires the broader asset, renderer, importer, and plugin validation from the asset gap plan. These tests lock the aggregate DTO math, wiring boundary, and the M6 minimal project sample.
