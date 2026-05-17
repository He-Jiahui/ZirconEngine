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
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/error.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/asset/project/manager/asset_lookup.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/registry_ops.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/resource_event.rs
implementation_files:
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/impls.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/project/meta.rs
  - zircon_runtime/src/asset/importer/contract.rs
  - zircon_runtime/src/asset/importer/native.rs
  - zircon_runtime/src/asset/importer/error.rs
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
tests:
  - zircon_runtime/src/asset/tests/facade.rs
  - zircon_runtime/src/asset/tests/project/manager.rs
  - zircon_runtime/src/asset/tests/project/zmeta.rs
  - zircon_runtime/src/core/resource/tests.rs
  - zircon_runtime_interface/src/tests/resource_contracts.rs
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
- `ProjectAssetManager` now exposes `load<TAsset>(locator)`, `handle<TAsset>(locator)`, `assets<TAsset>()`, `load_state(handle)`, `recursive_dependency_load_state(handle)`, `asset_load_state_by_id<TAsset>(id)`, and `subscribe_asset_events<TAsset>()`.

## Bevy Alignment And Zircon Divergence

The facade follows Bevy's typed `Handle<T>`, `Assets<T>`, typed asset events, and load-state vocabulary, but it preserves Zircon's existing storage and residency model. A `Handle<TAsset>` is a cheap identity value; it does not keep a payload resident by itself. Residency is still controlled by `ResourceLease<T>` from `Assets<TAsset>::acquire`, and the last lease drop may unload the payload until `ProjectAssetManager` rehydrates it from project artifacts or builtins.

`AssetLoadState::Reloading` is a Zircon extension. It is treated as a loading-class state through `is_loading_class()` while keeping the explicit state visible for diagnostics and hot-reload UI.

M2 adds dependency graph behavior without moving authority out of `ResourceManager`. `ImportedAssetEntry.dependencies` is persisted into the project meta document as locator data for each root or labeled subasset entry, then the completed `ProjectManager` registry resolves those locators to `ResourceRecord.dependency_ids`. This two-phase resolution lets forward references within a project scan resolve to the target asset's UUID-derived `ResourceId` instead of a locator-derived fallback ID. Unresolved dependency locators become `ResourceDiagnostic::error("unresolved asset dependency ...")` on the owning record.

M3 adds labeled subasset identity to the same facade path. Importers now return `AssetImportOutcome.entries`; the root entry keeps the source locator, and each subasset uses the existing `ResourceLocator` label syntax. The project scanner stores a separate `ResourceRecord` and artifact for each entry, so a typed `Handle<TAsset>` for `res://bundle.multi#Texture0` points at `AssetId::from_asset_uuid(entry.uuid)`. Scene/project references resolve by `AssetReference.uuid` first and use `AssetReference.url` only as a repair locator, while unknown label loads use `AssetImportError::MissingAssetLabel` so editor diagnostics can distinguish “source missing” from “subasset label missing”.

`ProjectAssetManager::recursive_dependency_load_state(handle)` now walks `ResourceRecord.dependency_ids` from the root handle. The root asset must first be directly `Loaded`; otherwise the direct state is returned. Dependency aggregation uses precedence `Failed > Reloading > Loading > NotLoaded > Loaded`, treats missing dependency records as `Failed`, and protects against cycles with a visited ID set. This deliberately diverges from Bevy's unknown-dependency behavior, which can remain indefinitely loading; Zircon reports missing graph edges deterministically because project/editor diagnostics need stable failure rows.

## Event Invariant

Typed event filtering must work even after removal, when the registry entry no longer exists. For that reason `ResourceEvent` now carries `resource_kind` at emission time. Resource producers in `ResourceManager` fill this from the affected `ResourceRecord`, and typed asset receivers can filter without querying mutable registry state. `Assets<TAsset>::subscribe_events()` returns `AssetEventReceiver<TAsset>` instead of a bare channel so dropping the typed receiver also disconnects the filter thread from its shutdown channel.

## Test Coverage

`zircon_runtime/src/asset/tests/facade.rs` covers typed handle kind mismatch, typed payload access, acquire/release residency behavior, event filtering across kinds including removed events, rename/reload/remove event ordering, load-state mapping, wrong concrete payload residency, failed reload preserving last-good payload, `Assets<TAsset>::insert/remove_by_locator`, recursive dependency graph state, missing dependency failure, and `ProjectAssetManager` generic load/handle/state/event entry points.

`zircon_runtime/src/asset/tests/project/manager.rs` covers entry dependency resolution into `ResourceRecord.dependency_ids`, unresolved dependency diagnostics, restart restore through the meta-persisted dependency locator list, root plus labeled subasset artifact persistence, duplicate label failure records, and structured unknown-label load errors. `zircon_runtime/src/asset/tests/project/zmeta.rs` covers `.zmeta` schema generation, ignored old sidecars, UUID-first stale-URL lookup, restored entry URL remapping after source rename, and subasset UUID preservation across transient failed reimport. `zircon_runtime/src/core/resource/tests.rs` covers dependency ID changes as revision-bearing record changes.
