---
related_code:
  - zircon_runtime/src/asset/artifact/mod.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/chunk_residency.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/json_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/mesh.rs
  - zircon_runtime/src/asset/artifact/cache_payload/material_shader.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload/toml_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/ui.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_cache_payload.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_script.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
  - examples/vampire/assets/shaders/default_pbr.zmeta
  - examples/vampire/.zircon/cache/assets/shaders/ae3ee5f2-ac09-3b2c-d00c-0fd96cccca44.zasset
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
implementation_files:
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/chunk_residency.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_asset_derived.rs
  - zircon_runtime/src/asset/artifact/ibl_bake_artifact_cache.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/json_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/mesh.rs
  - zircon_runtime/src/asset/artifact/cache_payload/material_shader.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload/toml_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/ui.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_cache_payload.rs
  - examples/vampire/assets/shaders/default_pbr.zmeta
  - examples/vampire/.zircon/cache/assets/shaders/ae3ee5f2-ac09-3b2c-d00c-0fd96cccca44.zasset
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - docs/plans/zircon_runtime/render/11-environment-lighting.md
  - docs/plans/zircon_runtime/shader/06-environment-ibl-and-pbr-correctness.md
tests:
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs::artifact_store_roundtrips_scene_assets_with_mesh_references
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs::artifact_store_roundtrips_scene_assets_with_camera_targets
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_components.rs::artifact_store_roundtrips_scene_assets_with_physics_components
  - zircon_runtime/src/asset/tests/assets/artifact_store/scene_script.rs::artifact_store_roundtrips_scene_assets_with_script_binding_json_values
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata
  - zircon_runtime/src/asset/tests/project/zmeta.rs::project_manager_imports_compound_zshader_package_with_subassets
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_cache_payload.rs::runtime_15_asset_artifact_cache_ui_documents_are_child_owner
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs::runtime_environment_ibl_bake_artifact_runtime_cache_store_reads_current_blob_as_candidate
  - zircon_runtime/tests/runtime_environment_ibl_bake_artifact_contract.rs::runtime_environment_ibl_bake_artifact_runtime_cache_store_reports_missing_and_rejected_blobs
  - zircon_runtime/tests/runtime_environment_ibl_bake_asset_derived_contract.rs
doc_type: module-detail
---

# Asset Artifact Cache

Current Runtime 04 owner sync (2026-07-10): `expected_source_file_count = 25`, `expected_guard_file_count = 22`, `test_anchor_count = 28`, `behavior_test_anchor_count = 24`, `missing_behavior_test_anchors = []`, `mirror_docs_guard_present = true`, and `risks = []`. The artifact scene-component and scene-script child tests are now counted as real guard owners, superseding the earlier 11-owner historical mirror without changing artifact behavior.

`zircon_runtime::asset::artifact` owns imported asset persistence under the explicit project artifact-cache root `.zircon/cache/assets`. It keeps `lib://.../*.zasset` as the runtime resource-locator scheme and reads those cache entries back into `ImportedAsset` values during project open and resource synchronization.

This module is a cache boundary, not an authoring format boundary. Authoring assets may use TOML, JSON, flattened serde fields, internally tagged enums, and custom deserializers. The `.zasset` cache uses bincode plus zstd compression, so every cached payload shape must be compatible with bincode's deserializer capabilities.

The module also owns the IBL bake artifact file entries for Shader 06 / Render 11. Runtime cache blobs and asset-derived companion blobs are separate `.zribl` raw blob families; neither reuses `.zasset`, `ImportedAsset`, bincode, or zstd.

## Store Contract

`ArtifactStore::write(...)` derives the artifact-cache path from the asset kind and resource id and streams the cache wire value through one zstd frame into a bounded staging file. It publishes immutable, BLAKE3-addressed 64 KiB compressed chunks first, then atomically replaces a small `ZRARTM03` manifest below `ProjectPaths::asset_artifact_root()`. The manifest records schema, kind, resource revision, complete compressed-content hash, raw/compressed byte counts, and the ordered chunk inventory. A process interrupted before manifest replacement leaves the last-good manifest generation readable; older `ZRARTM02` and monolithic formats are invalidated and recovered through reimport rather than compatibility decoding.

`ArtifactStore::read(...)` only accepts `lib://` URIs and `.zasset` paths. It validates the bounded manifest before chunk I/O, enforces raw and conservative zstd byte limits, lazily reads each compressed chunk through the store's shared bounded residency, verifies per-chunk and complete-content hashes, streams the zstd decoder into the bounded bincode reader, and checks both decoded byte count and asset kind. The store clone owns the same residency state, so repeated requested-chunk reads return the same `Arc<[u8]>` and perform no second payload disk read while resident.

The compressed-size guard applies Zstd's additional margin only below 128 KiB. The subtraction used to calculate that margin is evaluated lazily, so larger admitted payloads use `raw + raw / 256` without unsigned underflow. Focused large-payload tests lock both the exact-bound acceptance case and the first-byte-above-bound rejection case; the real App02 viewer remains the end-to-end consumer gate.

`ArtifactStore::open_chunk_inventory(...)` exposes immutable generation metadata without loading payload chunks. `read_compressed_chunk(...)` admits a single validated inventory index and returns its shared compressed bytes; diagnostics expose resident chunks/bytes, budget, cache hits, disk reads/bytes, and evictions. The API names the bytes as compressed because v3 chunks are ordered pieces of one zstd frame, not independently decodable texture mip or IBL face sections. Render upload-ready sectioning and the Runtime 11 I/O lane must consume this one inventory rather than create a second content index.

The store deliberately exposes no partial-update API. Importers produce complete `ImportedAsset` values, and the store persists complete manifest generations so resource synchronization can treat artifact cache reads as atomic. Chunk publication is content-addressed and may leave unreachable immutable chunks after an interrupted attempt; those chunks are never current truth until a manifest refers to them.

## IBL Bake Artifact Stores

`ibl_bake_artifact_cache.rs` owns runtime filesystem placement for reusable environment bake blobs. `IblBakeArtifactCacheStore::new(cache_root)` expects the project cache root, normally `.zircon/cache`, and writes raw `IblBakeArtifactBlob` bytes below `render/ibl/v{IBL_BAKE_ALGORITHM_VERSION}/{request_hash}/face_####_mips_##.zribl`.

The request hash is a deterministic BLAKE3 digest of `IblBakeKey`, face size, and mip count. Content bits remain inside the blob descriptor, so a cache hit is accepted only when `IblBakeArtifactBlob::decode_current_for_request(...)` proves the stored descriptor still satisfies the requested PMREM/SH9/IEM contents and current algorithm version.

`read_runtime_cache(...)` reports three cache states: `Hit(blob)`, `Missing`, or `Rejected(error)`. Missing and rejected cache files are non-fatal because Plan 06 §4.7 falls back to runtime compute. `IblBakeArtifactCacheRead::candidate()` converts only a valid hit into `IblBakeArtifactCandidate::runtime_cache(...)`, preserving the derived artifact > runtime cache > runtime compute source priority.

`ibl_bake_artifact_asset_derived.rs` owns asset-derived companion artifact placement for source 1. `IblBakeArtifactAssetDerivedStore::new(cache_root)` writes the same raw `.zribl` blob format below `.zircon/cache/render/ibl-derived/v{IBL_BAKE_ALGORITHM_VERSION}/{request_hash}/face_####_mips_##.zribl`. The staged helper `write_source_cubemap_asset_derived_artifact(...)` builds a current PMREM/SH9(+IEM) blob from a `SourceCubemapMipChain`, and `read_asset_derived_artifact(...)` returns `Hit`, `Missing`, or `Rejected` without making fallback paths fatal.

## Cache Wire Types

`ArtifactCacheAsset` is the bincode-safe wire enum. `cache_payload.rs` remains the variant dispatcher and conversion entry; variants that are already bincode-compatible can clone the imported asset directly. Variants with authoring-oriented serde shapes use dedicated cache structs and explicit conversion methods.

The dedicated cache layer currently protects these known problematic shapes:

- JSON-like data canonicalization, which cannot rely on `serde_json::Value` with bincode.
- Texture, shader, material, prefab, mesh, and physics material fields that use flattened data or authoring helper enums.
- Shader import redirects and texture slots with optional fields that authoring TOML skips when they are `None`.
- Scene mesh instances and LODs that keep authoring files compact while still writing complete field sequences for non-human-readable serializers.
- Scene camera targets and collider shapes that use internally tagged serde enums.
- Scene joint metadata that uses physics constraint metadata with custom deserialization behavior.
- UI v1/v2 document assets whose interface DTOs rely on TOML-like dynamic values and parser-owned validation.

The cache wire owner is folder-backed where the payload family has its own conversion rules. `cache_payload/json_value.rs` owns bincode-safe JSON canonical values for data assets and scene script bindings. `cache_payload/mesh.rs` owns mesh attributes, indices, morph targets, skin, and virtual-geometry wire conversion. `cache_payload/material_shader.rs` owns material and shader cache DTOs, shader render-state/resource/definition/texture-slot conversion, and the TOML value/table bridge consumption for material properties and shader editor metadata. `cache_payload/toml_value.rs` remains the shared TOML table/value conversion utility. `cache_payload/ui.rs` owns the UI v1/v2 document TOML normalization cache boundary.

The scene cache structs in `cache_payload/scene.rs` keep the runtime cache independent from authoring convenience syntax. `SceneCameraTargetAsset`, `SceneColliderShapeAsset`, and `PhysicsJointConstraintMetadata` are converted into cache-local structs or enums before bincode serialization and converted back after cache reads. `SceneMeshInstanceAsset` and `SceneMeshLodLevelAsset` now also protect direct binary serialization by writing all fields whenever `Serializer::is_human_readable()` is false, while their TOML/JSON-style output still omits absent or default authoring fields.

Shader cache payloads follow the same rule. `ShaderImportRedirectAsset` and `ShaderTextureSlotAsset` remain authoring-facing TOML structs, while the `.zasset` cache stores `ArtifactCacheShaderImportRedirectAsset` and `ArtifactCacheShaderTextureSlotAsset`. That keeps `redirect`, `default`, `sampler`, `group`, and `label` present in the bincode stream even when their values are `None`, instead of letting authoring `skip_serializing_if` attributes shift the following bytes into invalid enum tags.

The 2026-07-02 shader v2 follow-up keeps new `ShaderAsset` contract fields in the same cache boundary instead of making editor or importer tests build their own partial shader fixtures. `ArtifactCacheShaderAsset` now round-trips `kind`, `options`, `shading_model`, `render_state`, `queue`, `disabled_passes`, and `resources`; legacy cache reads default absent `kind` to `ShaderAssetKind::Surface` and absent collection fields to empty values. Built-in PBR shader construction also initializes the same fields, so validation gates do not rely on obsolete `ShaderAsset` literals.

UI cache payloads use a normalized text document boundary for the v1 and v2 UI document families. `UiLayoutAsset`, `UiWidgetAsset`, `UiStyleAsset`, `UiV2ViewAsset`, `UiV2ComponentAsset`, and `UiV2StyleAsset` serialize their validated document DTO back to TOML text before bincode caching, then restore through the typed `from_toml_str(...)` parsers after cache reads. `UiThemeAsset` and `UiIconAsset` remain direct cache variants because their payload structs are already bincode-compatible and do not require `deserialize_any`.

Runtime 15 M4 records `Runtime 15 M4 asset artifact cache UI document owner split` with status `runtime_15_asset_artifact_cache_ui_documents_owner_split_static_passed_cargo_deferred`. `asset/artifact/cache_payload.rs` remains the artifact cache dispatcher, while `asset/artifact/cache_payload/ui.rs` owns `ArtifactCacheUiAssetDocument` and `ArtifactCacheUiV2AssetDocument`. The guard `runtime_15_asset_artifact_cache_ui_documents_are_child_owner` keeps the parent and child under the production-file budget and prevents UI document conversion helpers from drifting back into the dispatcher.

Runtime 15 M4 also records `Runtime 15 M4 asset artifact cache material/shader owner split` with status `runtime_15_asset_artifact_cache_material_shader_owner_split_static_passed_cargo_deferred`. `asset/artifact/cache_payload.rs` now stays as the dispatcher and direct-cache owner at 325 lines, while `asset/artifact/cache_payload/material_shader.rs` is the 635-line child owner for `ArtifactCacheMaterialAsset`, `ArtifactCacheShaderAsset`, shader render-state/resource/definition/texture-slot cache helpers, and material/shader TOML cache conversion helpers. The guard `runtime_15_asset_artifact_cache_ui_documents_are_child_owner` now locks the UI and material/shader child owners together so cache DTOs do not drift back into the dispatcher.

Runtime 04 F7 keeps the cache boundary error contract typed. `ArtifactCacheAsset::from_imported(...)`, `ArtifactCacheAsset::into_imported(...)`, and TOML cache conversion now return `AssetImportError` rather than `Result<_, String>`. `AssetImportError::TomlSerialize`, `AssetImportError::TomlDeserialize`, `AssetImportError::CachedTomlDatetime`, `AssetImportError::UiDocument`, `AssetImportError::UiV2Document`, `AssetImportError::ArtifactCacheSerialize`, and `AssetImportError::ArtifactCacheDeserialize` keep the source error visible to callers. Runtime 15 F7 also closes the JSON cache number restore boundary: `ArtifactCacheJsonValue::into_json(...)` now returns `Result<serde_json::Value, AssetImportError>`, `cache_table_to_json(...)` propagates importer errors, and `AssetImportError::CachedJsonNonFiniteNumber` / `AssetImportError::CachedJsonNumberParse` replace the old cached-number `.expect(...)` path. `AssetImportError::Registry(#[from] AssetImporterRegistryError)` also preserves importer registry failures without lossy `error.to_string()` conversion; `asset_import_error_preserves_registry_error_source` and `review_f7_asset_artifact_errors_use_asset_import_error_sources` lock the behavior and structure, including that `asset/artifact/cache_payload/json_value.rs` stays free of `.unwrap()` and `.expect(`.

## Runtime 04 Fix Scope

Runtime 04 watcher validation exposed a lower-layer cache failure while opening projects that contained default scene artifacts:

`deserialize artifact cache: Bincode does not support the serde::Deserializer::deserialize_any method`

The failure happened before watcher assertions because scene `.zasset` files reused authoring serde shapes inside the bincode cache. The remediation keeps `ArtifactStore` unchanged at the public boundary and moves the fix into `cache_payload.rs` plus `cache_payload/scene.rs`, where scene mesh, camera, collider, and joint component payloads now have explicit bincode-safe cache representations.

## Runtime 06 Shader Cache Fix Scope

Runtime 06 real-backend validation exposed the same cache boundary on shader package artifacts. The `vampire_project_session` gate moved past ZrVM setup and failed while reading `lib://shaders/ae3ee5f2-ac09-3b2c-d00c-0fd96cccca44.zasset` because bincode deserialized a skipped optional shader import redirect as an enum tag: `tag for enum is not valid, found 5`.

The remediation is cache-local. Shader import redirects and texture slots now have explicit cache structs, and the vampire `default_pbr` shader metadata plus artifact were regenerated into that cache shape. The public shader authoring structs keep their compact TOML behavior.

## Validation

Focused artifact store tests cover scene artifact round-trips with mesh references, camera targets, physics components, and script binding JSON values. The broader watcher rerun is the acceptance path for the original failure because it opens a project, loads scene artifacts from `lib://scenes/*.zasset`, and then exercises hot-reload watcher behavior.

Static validation for this slice includes rustfmt over `cache_payload.rs`, `cache_payload/scene.rs`, and `artifact_store.rs`, plus conflict-marker checks over the changed runtime, docs, and session files. The first focused Cargo retry passed the camera-target and physics-component scene cache cases, then exposed the existing mesh-reference case as the same cache-wire boundary problem. Scene mesh payloads now use cache-local structs as well. The clean retry `cargo test -p zircon_runtime --lib artifact_store_roundtrips_scene_assets_with --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-04-asset-0612 --message-format short --color never -- --nocapture` passed 4/4; the watcher acceptance rerun `cargo test -p zircon_runtime --lib watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-04-asset-0612 --message-format short --color never -- --test-threads=1 --nocapture` passed 7/7.

On 2026-07-06, the IBL runtime cache store passed `cargo test -p zircon_runtime --test runtime_environment_ibl_bake_artifact_contract --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-ibl-artifact-payload-0706 --message-format short --color never -- --nocapture --test-threads=1`. The log `docs/tests/runtime/render/plan11_ibl_bake_artifact_payload_roundtrip_current_source_cargo_20260706.out.log` reports 9/9 tests passed; the matching `.exit.txt` records elapsed 00:05:41.7209590. The cache cases cover `.zircon/cache/render/ibl` path placement, `.zribl` extension, current blob readback, conversion to a runtime-cache candidate, missing-cache fallback, stale algorithm rejection, and truncated blob rejection.

On 2026-06-16, the shader cache fix passed `cargo test -p zircon_runtime --lib artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata --features backend-zr-vm --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-06-real-backend-0616 -- --nocapture --test-threads=1` 1/1. The same target then passed `project_manager_imports_compound_zshader_package_with_subassets` 1/1, proving compound `.zshader` packages with redirected and unredirected imports plus empty optional texture slots read back from `.zasset`. Full `vampire_project_session` real-backend validation no longer fails at shader artifact parse, but the broader session gate still times out in the runtime session path and remains pending. On 2026-06-21, Runtime 11 full-lib triage exposed `SceneMeshInstanceAsset` direct bincode failure `InvalidTagEncoding(36)` when skipped authoring fields shifted the following `AssetReference` bytes into an enum tag. The mesh asset serializer now writes every mesh instance and LOD field for non-human-readable serializers and keeps compact output only for human-readable formats; `artifact_store_bincode_roundtrips_scene_mesh_instance_asset` covers that direct binary guard. The same triage moved UI v1/v2 document artifacts behind the normalized TOML cache boundary so `project_manager_scans_ui_assets_and_assigns_ui_asset_kinds` and `project_manager_scans_zui_assets_and_restores_component_payloads` restore from `.zasset` without bincode attempting `deserialize_any` on interface document values. The direct core-min asset namespace rerun `target\codex-runtime11-coremin-tasks-0621\debug\deps\zircon_runtime-c339c28ec98a5de7.exe asset::tests:: --test-threads=1 --nocapture` passed 363/363 with 4334 filtered out, covering the mesh cache guard and UI cache restore path together.

The Runtime 04 structural audit split keeps artifact source/count ownership in `asset_pipeline_source_inventory.py`, artifact cache and artifact-store roundtrip anchors in `asset_pipeline_anchor_inventory.py`, audit reading/risk aggregation in the 328-line `asset_pipeline_boundary.py`, and Markdown rendering in the 117-line `asset_pipeline_markdown.py`. Current structural evidence reports `expected_source_file_count = 25`, `expected_guard_file_count = 22`, `worker_diagnostic_count = 7`, `expected_worker_diagnostic_count = 7`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 28`, `behavior_test_anchor_count = 24`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `retired_worker_request_sender_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this artifact doc aligned with Runtime 04, the runtime index, facade/worker/watcher/core-resource docs, M0 review, and runtime-interface convergence; this does not replace the broader pending `asset::` / `worker_pool` Cargo validation gate. Runtime 07 also records `artifact_cache_payload_owner_split_static_passed_cargo_deferred`: `cache_payload/{json_value,mesh,toml_value}.rs` keeps JSON, Mesh, and TOML wire owners folder-backed, `runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed` prevents those types from drifting back into the dispatcher, and the current owner-budget mirror after the render product diagnostics split is `large_file_hotspot_count = 40` / `runtime-other = 15`.

Current Runtime 04 source-owner synchronization (2026-08-14): `asset_pipeline_boundary` reports `expected_source_file_count = 26`; `core/resource/manager/commit.rs` owns reload transaction-state mutation in the current tree. This replaces the previous public-facade-only inventory; broader Cargo gates remain pending.
