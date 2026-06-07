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
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/mod.rs
  - zircon_runtime/src/asset/tests/project/asset_flow_sample.rs
  - zircon_runtime/src/core/framework/render/material/management/record_set.rs
  - zircon_runtime/src/core/framework/render/material/management/record_summary.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
implementation_files:
  - zircon_runtime/src/asset/management.rs
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/assets/scene.rs
  - zircon_runtime/src/asset/assets/material/material_asset.rs
  - zircon_runtime/src/asset/assets/material/mod.rs
  - zircon_runtime/src/asset/assets/mod.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/management.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/mod.rs
  - zircon_runtime/src/graphics/scene/mod.rs
  - zircon_runtime/src/graphics/scene/resources/resource_streamer/resource_streamer_accessors.rs
  - zircon_runtime/src/asset/tests/assets/scene/management.rs
plan_sources:
  - user: 2026-05-30 continue model material mesh entity shader flow and asset management
  - .codex/plans/ZirconEngine 资产、Texture、模型、ZShaderZMaterialZMesh 缺口补齐计划.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
tests:
  - zircon_runtime/src/asset/tests/assets/scene/management.rs::scene_asset_management_record_set_sorts_and_summarizes_records (entity record-set assertions)
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
  - cargo test -p zircon_runtime --lib asset_management_record_sets --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-01 aggregate mesh morph target and entity morph-weight counters: passed, 1 passed, 2303 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-01 M6 minimal asset-flow sample with glTF morph target/default weight and aggregate management assertions: passed, 1 passed, 2303 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-01 M6 minimal asset-flow sample with runtime ResourceStreamer asset-management overview/family status assertions after formatting: passed, 1 passed, 2324 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-01 M6 minimal asset-flow sample with pure ProjectAssetManager asset-management aggregate and ResourceStreamer delegation assertions after docs update: passed, 1 passed, 2329 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --target-dir D:\cargo-targets\zircon-mesh-index-format-0530 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-01 M6 minimal asset-flow sample after ResourceStreamer per-family management accessors delegated to ProjectAssetManager: passed, 1 passed, 2335 filtered; existing zircon_runtime lib-test warnings only)
  - cargo test -p zircon_runtime --lib project_manager_imports_minimal_gltf_material_shader_mesh_sample --locked --jobs 1 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-01 M6 sample after workspace/plugin gate fixes: passed in D:\cargo-targets\zircon-asset-m6-ci-0601, 1 passed, 2367 filtered; existing zircon_runtime warnings only)
  - cargo build --workspace --locked --verbose (2026-06-01 M6 root build gate: passed in D:\cargo-targets\zircon-asset-m6-ci-0601)
  - cargo test --workspace --locked --verbose (2026-06-01 M6 root test gate: blocked by active Editor workbench test failures after asset-management and render snapshot compile issues were fixed)
  - cargo build --manifest-path zircon_plugins/Cargo.toml --workspace --locked --verbose (2026-06-01 plugin build gate: passed)
  - cargo test --manifest-path zircon_plugins/Cargo.toml -p zircon_plugin_virtual_geometry_runtime -p zircon_plugin_hybrid_gi_runtime -p zircon_plugin_gltf_importer_runtime -p zircon_plugin_asset_importer_model_runtime -p zircon_plugin_asset_importer_shader_runtime -p zircon_plugin_material_editor_editor -p zircon_plugin_animation_runtime --locked --jobs 1 --message-format short --color never (2026-06-01 focused asset/render plugin gate: passed)
  - cargo test -p zircon_runtime --lib asset::tests::assets::scene --locked --jobs 1 --target-dir D:\cargo-targets\zircon-asset-test-splits-0605 --message-format short --color never -- --test-threads=1 --nocapture (2026-06-05 scene management split: passed, 11 passed including scene/management.rs regressions; existing zircon_runtime lib-test warnings only)
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
- The remaining fields mirror the top-level counters from each family: model count and model mesh-reference counts, mesh valid/invalid counts plus mesh morph target totals, scene counts, entity component/reference counts plus entity morph-weight totals, material asset ready/degraded/issue-row and authoring counts, prepared renderer material counts, and shader ready/not-ready/validation-diagnostic counts.

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

## Project Manager Surface

`ProjectAssetManager` now exposes the same asset-management read model before renderer preparation. Its management methods scan the current `ResourceManager` registry by `ResourceKind`, load typed assets through the existing `load_*_asset(...)` functions, and build the same per-family record sets used by the aggregate:

- `model_asset_management_record_set()`
- `mesh_asset_management_record_set()`
- `scene_asset_management_record_set()`
- `scene_entity_management_record_set()`
- `material_asset_management_record_set()`
- `shader_asset_management_record_set()`

`ProjectAssetManager::asset_management_record_sets()` combines those families with an empty `RenderMaterialManagementRecordSet`, because asset-layer callers do not have renderer-prepared material rows. `asset_management_overview()`, `asset_management_family_summaries()`, and `asset_management_family_status_index()` expose the compact read surfaces without requiring a graphics device. This gives editor/asset tooling a neutral project-level management payload for model, mesh, scene/entity, material asset, and shader state while preserving renderer-prepared material detail as an optional overlay.

## Streamer Surface

`ResourceStreamer` per-family management accessors now delegate model, mesh, scene, flattened entity, material asset, and shader reads to `ProjectAssetManager`. `ResourceStreamer::asset_management_record_sets()` uses `ProjectAssetManager::asset_management_record_sets_with_prepared_materials(...)` and supplies `material_management_record_set()` for renderer-prepared material rows. This keeps project and renderer management summaries aligned while preserving prepared-material readiness/status/query detail when renderer state exists.

This keeps the combined read model deterministic without introducing a second registry scan policy in graphics. Model, mesh, scene, material asset, and shader rows use `ResourceManager` ids by `ResourceKind`. Entity rows are derived from loaded scene assets and keep the owning `scene_id` beside the authoring entity id. The prepared renderer material record set is carried alongside the material asset record set for panels that need readiness/status/query detail after preparation, but the top-level Material family now represents registered `.zmaterial` assets rather than only already-prepared materials.

`ResourceStreamer::asset_management_overview()`, `asset_management_family_summaries()`, and `asset_management_family_status_index()` derive the same family overview state from the aggregate payload. They are convenience surfaces for management headers and navigation chrome that should not load or serialize all detail records.

The M6 project sample now instantiates a test `ResourceStreamer` over the same `ProjectAssetManager` that opened the generated project and verifies the real runtime aggregate, overview, family summaries, and family status index together. That runtime view includes the project records plus built-in runtime defaults: the sample locks 17 managed records, including 4 model rows, 1 mesh row, 2 scene rows, 2 flattened entity rows, 5 material asset rows, 3 shader rows, and 2 degraded built-in material-management rows. The same assertion keeps the morph-sensitive counters visible at the streamer boundary: 1 mesh morph target, 1 morph target attribute, and 2 entity morph weights from the generated `#Node0` and `#Scene0` scene assets.

## Test Coverage

`zircon_runtime/src/asset/tests/assets/material.rs` covers `MaterialAssetOverview`, `MaterialAssetManagementRecord`, and `MaterialAssetManagementRecordSet` by checking stable id sorting, shader/reference counts, authored texture slot counts, fallback slot counts, diagnostics, ready/degraded summary counts, and direct reference totals.

`zircon_runtime/src/asset/tests/assets/scene/management.rs` covers `SceneEntityManagementRecordSet` by projecting a scene management record into flattened entity rows and checking stable `(scene_id, entity)` sorting plus entity-row summary totals.

`zircon_runtime/src/asset/tests/assets/model.rs` covers the model record-set summary that feeds this aggregate, including mesh-referenced model totals and primitive mesh-reference totals.

`zircon_runtime/src/asset/tests/assets/management.rs` constructs representative per-family record sets and verifies that `AssetManagementRecordSets::from_record_sets(...)` preserves the nested payloads while deriving the combined managed/degraded counts, model mesh-reference totals, mesh morph target totals, entity totals, entity morph-weight totals, material asset issue-row totals, prepared material counters, stable family row order, family statuses, ready/degraded counts, issue-row counts, ready/degraded family status buckets, and compact overview projection from the family summaries.

`zircon_runtime/src/asset/tests/project/asset_flow_sample.rs` is the first project-level cross-family management sample. It loads a glTF scene, root model, mesh model, primitive mesh, scene entity, imported glTF material, authored `.zmaterial`, shader package, and DDS texture, then asserts the per-family management summaries for scene/entity primitive bindings and morph weights, model mesh references, mesh vertex/index/morph-target counts, material slot/fallback counts, and texture upload fallback. The sample also derives a compact `AssetManagementRecordSets` payload from those family rows and verifies the aggregate entity direct-mesh, primitive-binding, entity morph-weight, mesh morph-target, and mesh morph-target-attribute counters, so the top-level management summary matches the imported scene graph. The same sample opens the generated project through `ProjectAssetManager`, checks typed handles plus direct and recursive load states for scene, model, mesh, material, shader, and texture assets, verifies the pure project-manager aggregate/overview/family-status surfaces, then instantiates a `ResourceStreamer` over that manager and verifies the renderer surface delegates to the same aggregate when no prepared materials exist. This keeps management rows, facade residency state, built-in runtime defaults, and runtime-streamer summary projection verified from one import graph.

Milestone acceptance still requires the broader asset, renderer, importer, and plugin validation from the asset gap plan. These tests lock the aggregate DTO math, wiring boundary, and the M6 minimal project sample.

On 2026-06-05, the scene-management regressions were validated from their split module through the full `asset::tests::assets::scene` filter in `D:\cargo-targets\zircon-asset-test-splits-0605`. The run passed all 11 scene tests, including populated scene overview counts, empty-scene behavior, scene record-set sorting/summary, and flattened scene-entity record-set assertions.

## M6 Validation Status

The current M6 validation pass uses `D:\cargo-targets\zircon-asset-m6-ci-0601` as the shared target directory. The root workspace build gate passes on the current tree. The minimal project sample also passes there, proving the project-level management aggregate, typed facade load states, ResourceStreamer delegation, morph counters, shader/material dependencies, and compressed texture fallback remain aligned after the downstream render-plugin fixture updates.

The full root workspace test gate is not green yet, but the remaining failures are outside this asset-management surface. After stale Virtual Geometry and Hybrid GI snapshot constructors were updated for the current `RenderMeshSnapshot` and `SceneMeshInstanceAsset` shape, `cargo test --workspace --locked --verbose` reached runtime execution and failed in active Editor workbench tests. The full plugin build gate passes, and the focused asset/render plugin gate passes for animation, model importer, shader importer, glTF importer, Hybrid GI, Material Editor, and Virtual Geometry. The plugin all-targets and full plugin test gates are currently blocked by active Editor test-mode and Navigation plugin-manifest validation work owned by neighboring session notes, so this document records the blocker rather than treating M6 as complete.

On 2026-06-03, the current continuation did not start a new root workspace gate because active Hub,
runtime-layout, and editor command-feedback Cargo lanes were already running. Scoped validation for
the current tree passed instead: `cargo check -p zircon_runtime --lib --locked --jobs 1` in
`E:\cargo-targets\zircon-runtime-plugin-asset-importer-metadata-subgroups` and
`cargo check -p zircon_app --lib --locked --jobs 1` in `E:\cargo-targets\zircon-render-main-chain`.
Editor Workbench focused module/visibility checks also passed, but full M6 acceptance remains open
until a fresh root workspace build/test gate is captured.

A later 2026-06-03 refresh re-ran the M6 project sample on the current tree. The first attempt
exposed a lower runtime render test fixture that still constructed `RenderFeaturePassDescriptor`
with a raw struct literal after the descriptor gained the `compute_workload` field. That fixture now
uses the descriptor constructor, preserving the intended invalid read/storage resource row while
letting default descriptor fields come from one owner path. The focused lower-layer regression
`cargo test -p zircon_runtime --lib pipeline_compile_rejects_storage_write_mode_on_read_access --locked --jobs 1 --target-dir E:\cargo-targets\zircon-runtime-plugin-asset-importer-metadata-subgroups --message-format short --color never -- --test-threads=1`
passed with 1 test / 2520 filtered, and the original
`project_manager_imports_minimal_gltf_material_shader_mesh_sample` sample then passed with 1 test /
2520 filtered in the same target directory. The sample evidence is recorded in
`.codex/tmp/asset_m6_project_sample_refresh_after_descriptor_fixture_fix_20260603_detached.log`.
