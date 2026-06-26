---
related_code:
  - zircon_runtime/src/asset/artifact/mod.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/json_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/mesh.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload/toml_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/ui.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_cache_payload.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - examples/vampire/assets/shaders/default_pbr.zmeta
  - examples/vampire/library/shaders/ae3ee5f2-ac09-3b2c-d00c-0fd96cccca44.zasset
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
implementation_files:
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/json_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/mesh.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/artifact/cache_payload/toml_value.rs
  - zircon_runtime/src/asset/artifact/cache_payload/ui.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_cache_payload.rs
  - examples/vampire/assets/shaders/default_pbr.zmeta
  - examples/vampire/library/shaders/ae3ee5f2-ac09-3b2c-d00c-0fd96cccca44.zasset
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_markdown.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_source_inventory.py
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_anchor_inventory.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
  - docs/plans/zircon_runtime/runtime/06-plugin-surface-and-lifecycle.md
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
tests:
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_scene_assets_with_mesh_references
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_scene_assets_with_camera_targets
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_scene_assets_with_physics_components
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_scene_assets_with_script_binding_json_values
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata
  - zircon_runtime/src/asset/tests/project/zmeta.rs::project_manager_imports_compound_zshader_package_with_subassets
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/asset_cache_payload.rs::runtime_15_asset_artifact_cache_ui_documents_are_child_owner
doc_type: module-detail
---

# Asset Artifact Cache

`zircon_runtime::asset::artifact` owns imported asset artifact persistence under the project library root. It writes runtime cache files as `lib://.../*.zasset` URIs and reads those artifacts back into `ImportedAsset` values during project open and resource synchronization.

This module is a cache boundary, not an authoring format boundary. Authoring assets may use TOML, JSON, flattened serde fields, internally tagged enums, and custom deserializers. The `.zasset` cache uses bincode plus zstd compression, so every cached payload shape must be compatible with bincode's deserializer capabilities.

## Store Contract

`ArtifactStore::write(...)` derives the library path from the asset kind and resource id, serializes the imported asset into the cache wire type, prepends the artifact cache magic header, compresses the payload, and writes it below `ProjectPaths::library_root()`.

`ArtifactStore::read(...)` only accepts `lib://` URIs and `.zasset` paths. It validates the cache magic header, decompresses the payload, deserializes the cache wire type, converts it back into `ImportedAsset`, and checks that the artifact path kind matches the decoded asset kind when the path is recognized.

The store deliberately exposes no partial-update API. Importers produce complete `ImportedAsset` values, and the store persists complete cache records so resource synchronization can treat artifact cache reads as atomic.

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

The cache wire owner is folder-backed where the payload family has its own conversion rules. `cache_payload/json_value.rs` owns bincode-safe JSON canonical values for data assets and scene script bindings. `cache_payload/mesh.rs` owns mesh attributes, indices, morph targets, skin, and virtual-geometry wire conversion. `cache_payload/toml_value.rs` owns TOML table/value conversion for material properties and shader editor metadata. `cache_payload/ui.rs` owns the UI v1/v2 document TOML normalization cache boundary.

The scene cache structs in `cache_payload/scene.rs` keep the runtime cache independent from authoring convenience syntax. `SceneCameraTargetAsset`, `SceneColliderShapeAsset`, and `PhysicsJointConstraintMetadata` are converted into cache-local structs or enums before bincode serialization and converted back after cache reads. `SceneMeshInstanceAsset` and `SceneMeshLodLevelAsset` now also protect direct binary serialization by writing all fields whenever `Serializer::is_human_readable()` is false, while their TOML/JSON-style output still omits absent or default authoring fields.

Shader cache payloads follow the same rule. `ShaderImportRedirectAsset` and `ShaderTextureSlotAsset` remain authoring-facing TOML structs, while the `.zasset` cache stores `ArtifactCacheShaderImportRedirectAsset` and `ArtifactCacheShaderTextureSlotAsset`. That keeps `redirect`, `default`, `sampler`, `group`, and `label` present in the bincode stream even when their values are `None`, instead of letting authoring `skip_serializing_if` attributes shift the following bytes into invalid enum tags.

UI cache payloads use a normalized text document boundary for the v1 and v2 UI document families. `UiLayoutAsset`, `UiWidgetAsset`, `UiStyleAsset`, `UiV2ViewAsset`, `UiV2ComponentAsset`, and `UiV2StyleAsset` serialize their validated document DTO back to TOML text before bincode caching, then restore through the typed `from_toml_str(...)` parsers after cache reads. `UiThemeAsset` and `UiIconAsset` remain direct cache variants because their payload structs are already bincode-compatible and do not require `deserialize_any`.

Runtime 15 M4 records `Runtime 15 M4 asset artifact cache UI document owner split` with status `runtime_15_asset_artifact_cache_ui_documents_owner_split_static_passed_cargo_deferred`. `asset/artifact/cache_payload.rs` remains the artifact cache dispatcher, while `asset/artifact/cache_payload/ui.rs` owns `ArtifactCacheUiAssetDocument` and `ArtifactCacheUiV2AssetDocument`. The guard `runtime_15_asset_artifact_cache_ui_documents_are_child_owner` keeps the parent and child under the production-file budget and prevents UI document conversion helpers from drifting back into the dispatcher.

Runtime 04 F7 keeps the cache boundary error contract typed. `ArtifactCacheAsset::from_imported(...)`, `ArtifactCacheAsset::into_imported(...)`, and TOML cache conversion now return `AssetImportError` rather than `Result<_, String>`. `AssetImportError::TomlSerialize`, `AssetImportError::TomlDeserialize`, `AssetImportError::CachedTomlDatetime`, `AssetImportError::UiDocument`, `AssetImportError::UiV2Document`, `AssetImportError::ArtifactCacheSerialize`, and `AssetImportError::ArtifactCacheDeserialize` keep the source error visible to callers. `AssetImportError::Registry(#[from] AssetImporterRegistryError)` also preserves importer registry failures without lossy `error.to_string()` conversion; `asset_import_error_preserves_registry_error_source` and `review_f7_asset_artifact_errors_use_asset_import_error_sources` lock the behavior and structure.

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

On 2026-06-16, the shader cache fix passed `cargo test -p zircon_runtime --lib artifact_store_roundtrips_shader_assets_with_cache_safe_toml_metadata --features zr-vm-real-backend --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-06-real-backend-0616 -- --nocapture --test-threads=1` 1/1. The same target then passed `project_manager_imports_compound_zshader_package_with_subassets` 1/1, proving compound `.zshader` packages with redirected and unredirected imports plus empty optional texture slots read back from `.zasset`. Full `vampire_project_session` real-backend validation no longer fails at shader artifact parse, but the broader session gate still times out in the runtime session path and remains pending. On 2026-06-21, Runtime 11 full-lib triage exposed `SceneMeshInstanceAsset` direct bincode failure `InvalidTagEncoding(36)` when skipped authoring fields shifted the following `AssetReference` bytes into an enum tag. The mesh asset serializer now writes every mesh instance and LOD field for non-human-readable serializers and keeps compact output only for human-readable formats; `artifact_store_bincode_roundtrips_scene_mesh_instance_asset` covers that direct binary guard. The same triage moved UI v1/v2 document artifacts behind the normalized TOML cache boundary so `project_manager_scans_ui_assets_and_assigns_ui_asset_kinds` and `project_manager_scans_zui_assets_and_restores_component_payloads` restore from `.zasset` without bincode attempting `deserialize_any` on interface document values. The direct core-min asset namespace rerun `target\codex-runtime11-coremin-tasks-0621\debug\deps\zircon_runtime-c339c28ec98a5de7.exe asset::tests:: --test-threads=1 --nocapture` passed 363/363 with 4334 filtered out, covering the mesh cache guard and UI cache restore path together.

The Runtime 04 structural audit split keeps artifact source/count ownership in `asset_pipeline_source_inventory.py`, artifact cache and artifact-store roundtrip anchors in `asset_pipeline_anchor_inventory.py`, audit reading/risk aggregation in the 328-line `asset_pipeline_boundary.py`, and Markdown rendering in the 117-line `asset_pipeline_markdown.py`. Current structural evidence reports `expected_source_file_count = 22`, `expected_guard_file_count = 11`, `worker_diagnostic_count = 7`, `expected_worker_diagnostic_count = 7`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 24`, `behavior_test_anchor_count = 20`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `retired_worker_request_sender_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this artifact doc aligned with Runtime 04, the runtime index, facade/worker/watcher/core-resource docs, M0 review, and runtime-interface convergence; this does not replace the broader pending `asset::` / `worker_pool` Cargo validation gate. Runtime 07 also records `artifact_cache_payload_owner_split_static_passed_cargo_deferred`: `cache_payload/{json_value,mesh,toml_value}.rs` keeps JSON, Mesh, and TOML wire owners folder-backed, `runtime_07_artifact_cache_payload_owner_split_keeps_wire_types_folder_backed` prevents those types from drifting back into the dispatcher, and the current owner-budget mirror after the render product diagnostics split is `large_file_hotspot_count = 40` / `runtime-other = 15`.
