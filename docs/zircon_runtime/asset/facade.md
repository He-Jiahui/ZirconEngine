---
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/asset/project/manager/asset_lookup.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/resource_event.rs
implementation_files:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/assets/mesh/mod.rs
  - zircon_runtime/src/asset/assets/mesh/mesh_asset.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/registry.rs
  - zircon_runtime/src/asset/importer/ingest/asset_importer.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/asset/project/manager/asset_lookup.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/resource_event.rs
plan_sources:
  - user: 2026-05-08 implement Bevy-Style Asset Stack Completion Plan M1
  - user: 2026-05-08 continue Bevy-Style Asset Stack Completion Plan M2
  - user: 2026-05-08 continue Bevy-Style Asset Stack Completion Plan M3
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - user: 2026-05-20 implement ZirconEngine asset/texture/model/ZShader/ZMaterial/ZMesh completion plan
tests:
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_runtime/src/asset/tests/assets/mesh.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/project/package_assets.rs
  - CARGO_TARGET_DIR=D:\cargo-targets\zircon-asset-package-m2 cargo test -p zircon_runtime --lib --locked asset::tests::project::package_assets --jobs 1 --message-format short --color never -- --test-threads=1 (2026-05-20 package roots M2: passed, 3 passed)
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/core/resource/tests.rs
  - zircon_runtime_interface/src/tests/resource_contracts.rs
  - cargo check -p zircon_runtime --lib --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-asset-parity-runtime-lib-0520 --message-format short --color never (2026-05-20 asset parity implementation: passed; existing warnings only)
  - cargo test -p zircon_runtime --lib importer_capability_report_marks_diagnostic_only_backends --locked --offline --jobs 1 --target-dir E:\cargo-targets\zircon-asset-parity-runtime-lib-0520 --message-format short --color never -- --test-threads=1 (2026-05-20 asset parity implementation: timed out during Windows test build/link before Rust test diagnostics)
  - cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir E:\Git\ZirconEngine\zircon_plugins\target --message-format short --color never (2026-05-20 facade WGSL capture re-export: initially failed with E0425 for `crate::asset::validate_wgsl_captures`; passed after top-level re-export, existing warnings only)
doc_type: module-detail
---

# Asset Facade

## Purpose

`zircon_runtime::asset::facade` adds the Bevy-style typed asset surface for the asset stack plan without creating a second asset store. The facade is a typed view over `zircon_runtime::core::resource::ResourceManager`, so identity, records, payloads, runtime residency, revision, dependency IDs, diagnostics, and resource events remain owned by the existing resource foundation.

## Public Surface

- `Asset` maps a Rust asset payload type to its canonical `ResourceMarker` and user-facing label.
- `Handle<TAsset>` wraps `ResourceHandle<TAsset::Marker>` and can convert to or from `UntypedResourceHandle` only when the stored `ResourceKind` matches.
- `Assets<TAsset>` wraps a cloned `ResourceManager` and exposes typed `get`, `get_cloned`, `acquire`, `contains`, `insert`, `remove_by_locator`, `load_state`, and event subscription helpers.
- `AssetEvent<TAsset>` maps `ResourceEventKind::{Added, Updated, Removed, Renamed, ReloadFailed}` to typed events and filters by `ResourceEvent.resource_kind`; `AssetEventReceiver<TAsset>` owns the filter-thread shutdown signal for the subscription lifetime.
- `AssetLoadState` maps missing records, import/runtime loading, loaded resident payloads, failed resources, and Zircon's explicit `Reloading` state.
- `ProjectAssetManager` now exposes `load<TAsset>(locator)`, `handle<TAsset>(locator)`, `assets<TAsset>()`, `load_state(handle)`, `direct_dependency_load_state(handle)`, `recursive_dependency_load_state(handle)`, `asset_load_state_by_id<TAsset>(id)`, `subscribe_asset_events<TAsset>()`, and importer capability report helpers.
- `MeshAsset` is now part of this typed facade through `AssetKind::Mesh`, `MeshMarker`, `Handle<MeshAsset>`, and `Assets<MeshAsset>`.
- The top-level `zircon_runtime::asset` facade also re-exports shared asset validation helpers such as `validate_wgsl_captures(...)`, so documented fixtures, shader import, and public callers use the same shader/material capture contract.

## Bevy Alignment And Zircon Divergence

The facade follows Bevy's typed `Handle<T>`, `Assets<T>`, typed asset events, and load-state vocabulary, but it preserves Zircon's existing storage and residency model. A `Handle<TAsset>` is a cheap identity value; it does not keep a payload resident by itself. Residency is still controlled by `ResourceLease<T>` from `Assets<TAsset>::acquire`, and the last lease drop may unload the payload until `ProjectAssetManager` rehydrates it from project artifacts or builtins.

`AssetLoadState::Reloading` is a Zircon extension. It is treated as a loading-class state through `is_loading_class()` while keeping the explicit state visible for diagnostics and hot-reload UI.

M2 adds dependency graph behavior without moving authority out of `ResourceManager`. `ImportedAssetEntry.dependencies` is persisted into the project meta document as locator data for each root or labeled subasset entry, then the completed `ProjectManager` registry resolves those locators to `ResourceRecord.dependency_ids`. This two-phase resolution lets forward references within a project scan resolve to the target asset's UUID-derived `ResourceId` instead of a locator-derived fallback ID. Unresolved dependency locators become `ResourceDiagnostic::error("unresolved asset dependency ...")` on the owning record.

M3 adds labeled subasset identity to the same facade path. Importers now return `AssetImportOutcome.entries`; the root entry keeps the source locator, and each subasset uses the existing `ResourceLocator` label syntax. The project scanner stores a separate `ResourceRecord` and artifact for each entry, so a typed `Handle<TAsset>` for `res://bundle.multi#Texture0` points at `AssetId::from_asset_uuid(entry.uuid)`. Scene/project references resolve by `AssetReference.uuid` first and use `AssetReference.url` only as a repair locator, while unknown label loads use `AssetImportError::MissingAssetLabel` so editor diagnostics can distinguish “source missing” from “subasset label missing”.

`ProjectAssetManager::recursive_dependency_load_state(handle)` now walks `ResourceRecord.dependency_ids` from the root handle. The root asset must first be directly `Loaded`; otherwise the direct state is returned. Dependency aggregation uses precedence `Failed > Reloading > Loading > NotLoaded > Loaded`, treats missing dependency records as `Failed`, and protects against cycles with a visited ID set. This deliberately diverges from Bevy's unknown-dependency behavior, which can remain indefinitely loading; Zircon reports missing graph edges deterministically because project/editor diagnostics need stable failure rows.

`ProjectAssetManager::direct_dependency_load_state(handle)` aggregates only first-level dependencies with the same precedence. This lets editor rows and asset inspectors distinguish a root asset that is directly blocked by a material/texture/shader dependency from a deeper recursive failure, while still sharing the same `AssetLoadState` vocabulary used by Bevy-style `AssetServer::load_state` queries.

Importer capability reporting now lives beside the same facade entry point. `AssetImporterCapabilityReport` pairs the importer descriptor with `AssetImporterCapabilityStatus::Available` or `DiagnosticOnly`, and `ProjectAssetManager::asset_importer_capability_report_for_source(...)` can answer the expected importer status before a source file is scanned. This keeps diagnostic-only backends such as FBX/USD/DAE/3DS visible to tools without pretending that they can produce runtime assets.

## Event Invariant

Typed event filtering must work even after removal, when the registry entry no longer exists. For that reason `ResourceEvent` now carries `resource_kind` at emission time. Resource producers in `ResourceManager` fill this from the affected `ResourceRecord`, and typed asset receivers can filter without querying mutable registry state. `Assets<TAsset>::subscribe_events()` returns `AssetEventReceiver<TAsset>` instead of a bare channel so dropping the typed receiver also disconnects the filter thread from its shutdown channel.

## Test Coverage

`zircon_runtime/src/asset/tests/facade.rs` covers typed handle kind mismatch, typed payload access, acquire/release residency behavior, event filtering across kinds including removed events, rename/reload/remove event ordering, load-state mapping, direct and recursive dependency graph state, missing dependency failure, wrong concrete payload residency, failed reload preserving last-good payload, `Assets<TAsset>::insert/remove_by_locator`, and `ProjectAssetManager` generic load/handle/state/event entry points.

`zircon_runtime/src/asset/tests/assets/importer.rs` covers importer capability reports, including diagnostic-only model formats that remain intentionally non-producing. `zircon_runtime/src/asset/tests/project/manager.rs` covers entry dependency resolution into `ResourceRecord.dependency_ids`, unresolved dependency diagnostics, restart restore through the meta-persisted dependency locator list, root plus labeled subasset artifact persistence, duplicate label failure records, and structured unknown-label load errors. `zircon_runtime/src/asset/tests/project/package_assets.rs` covers direct and manifest-based package asset roots, package subasset UUID/artifact restore, unknown package lookup, malformed package roots, and structured missing-label errors for `package://` sources. `zircon_runtime/src/asset/tests/project/zmeta.rs` covers `.zmeta` schema generation, ignored old sidecars, UUID-first stale-URL lookup, restored entry URL remapping after source rename, and subasset UUID preservation across transient failed reimport. `zircon_runtime/src/core/resource/tests.rs` covers dependency ID changes as revision-bearing record changes.

The 2026-05-20 runtime `--tests` check also covers the facade-level `validate_wgsl_captures(...)` re-export used by the documented `.zshader` fixture; the first run exposed the missing top-level export and the rerun passed after restoring that public surface.
