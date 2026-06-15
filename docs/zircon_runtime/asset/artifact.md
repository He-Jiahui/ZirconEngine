---
related_code:
  - zircon_runtime/src/asset/artifact/mod.rs
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
implementation_files:
  - zircon_runtime/src/asset/artifact/store.rs
  - zircon_runtime/src/asset/artifact/cache_payload.rs
  - zircon_runtime/src/asset/artifact/cache_payload/scene.rs
  - zircon_runtime/src/tests/runtime_absorption/asset_pipeline.rs
  - .codex/skills/zircon-project-skills/zr-runtime-interface-convergence/scripts/runtime_structure_audits/asset_pipeline_boundary.py
plan_sources:
  - docs/plans/zircon_runtime/runtime/04-asset-pipeline-alignment.md
tests:
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_scene_assets_with_mesh_references
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_scene_assets_with_camera_targets
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_scene_assets_with_physics_components
  - zircon_runtime/src/asset/tests/assets/artifact_store.rs::artifact_store_roundtrips_scene_assets_with_script_binding_json_values
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

`ArtifactCacheAsset` is the bincode-safe wire enum. Variants that are already bincode-compatible can clone the imported asset directly. Variants with authoring-oriented serde shapes use dedicated cache structs and explicit conversion methods.

The dedicated cache layer currently protects these known problematic shapes:

- JSON-like data canonicalization, which cannot rely on `serde_json::Value` with bincode.
- Texture, shader, material, prefab, mesh, and physics material fields that use flattened data or authoring helper enums.
- Scene mesh instances and LODs that use `skip_serializing_if` to keep authoring files compact.
- Scene camera targets and collider shapes that use internally tagged serde enums.
- Scene joint metadata that uses physics constraint metadata with custom deserialization behavior.

The scene cache structs in `cache_payload/scene.rs` keep the runtime cache independent from authoring convenience syntax. `SceneMeshInstanceAsset`, `SceneCameraTargetAsset`, `SceneColliderShapeAsset`, and `PhysicsJointConstraintMetadata` are converted into cache-local structs or enums before bincode serialization and converted back after cache reads.

## Runtime 04 Fix Scope

Runtime 04 watcher validation exposed a lower-layer cache failure while opening projects that contained default scene artifacts:

`deserialize artifact cache: Bincode does not support the serde::Deserializer::deserialize_any method`

The failure happened before watcher assertions because scene `.zasset` files reused authoring serde shapes inside the bincode cache. The remediation keeps `ArtifactStore` unchanged at the public boundary and moves the fix into `cache_payload.rs` plus `cache_payload/scene.rs`, where scene mesh, camera, collider, and joint component payloads now have explicit bincode-safe cache representations.

## Validation

Focused artifact store tests cover scene artifact round-trips with mesh references, camera targets, physics components, and script binding JSON values. The broader watcher rerun is the acceptance path for the original failure because it opens a project, loads scene artifacts from `lib://scenes/*.zasset`, and then exercises hot-reload watcher behavior.

Static validation for this slice includes rustfmt over `cache_payload.rs`, `cache_payload/scene.rs`, and `artifact_store.rs`, plus conflict-marker checks over the changed runtime, docs, and session files. The first focused Cargo retry passed the camera-target and physics-component scene cache cases, then exposed the existing mesh-reference case as the same cache-wire boundary problem. Scene mesh payloads now use cache-local structs as well. The clean retry `cargo test -p zircon_runtime --lib artifact_store_roundtrips_scene_assets_with --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-04-asset-0612 --message-format short --color never -- --nocapture` passed 4/4; the watcher acceptance rerun `cargo test -p zircon_runtime --lib watcher --locked --jobs 1 --target-dir D:\cargo-targets\zircon-runtime-04-asset-0612 --message-format short --color never -- --test-threads=1 --nocapture` passed 7/7.

The Runtime 04 `asset_pipeline_boundary` audit also counts the four scene artifact roundtrip guards and checks that the artifact cache wire anchors stay present. Current structural evidence reports `expected_source_file_count = 19`, `expected_guard_file_count = 11`, `worker_diagnostic_count = 5`, `expected_worker_diagnostic_count = 5`, `artifact_store_roundtrip_count = 4`, `expected_artifact_store_roundtrip_count = 4`, `watcher_acceptance_reference_count = 1`, `expected_watcher_acceptance_count = 7`, `artifact_acceptance_reference_count = 3`, `test_anchor_count = 22`, `behavior_test_anchor_count = 18`, `missing_behavior_test_anchors = []`, `missing_doc_anchors = []`, `missing_cargo_gate_anchors = []`, `retired_worker_new_references = []`, `old_watch_debounce_references = []`, `mirror_docs_guard_present = true`, and `risks = []`. `runtime_04_asset_pipeline_mirror_docs_match_structure_audit_counts` keeps this artifact doc aligned with Runtime 04, the runtime index, facade/worker/watcher/core-resource docs, M0 review, and runtime-interface convergence; this does not replace the broader pending `asset::` / `worker_pool` Cargo validation gate.
