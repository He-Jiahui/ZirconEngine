---
related_code:
  - zircon_runtime/src/asset/mod.rs
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/event.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/readiness.rs
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
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
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
  - zircon_runtime/src/asset/facade/readiness.rs
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
  - zircon_runtime/src/asset/pipeline/manager/asset_manager/asset_manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/construction.rs
  - zircon_runtime/src/asset/pipeline/manager/service_contracts/asset_manager_contract.rs
  - zircon_runtime/src/asset/project/manager/artifact_access.rs
  - zircon_runtime/src/asset/project/manager/asset_lookup.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime_interface/src/resource/resource_event.rs
plan_sources:
  - user: 2026-05-08 implement Bevy-Style Asset Stack Completion Plan M1
  - user: 2026-05-08 continue Bevy-Style Asset Stack Completion Plan M2
  - user: 2026-05-08 continue Bevy-Style Asset Stack Completion Plan M3
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/资产 .zmeta 与 Shader Material 资产化计划.md
  - user: 2026-05-20 implement ZirconEngine asset/texture/model/ZShader/ZMaterial/ZMesh completion plan
  - docs/superpowers/specs/2026-05-18-asset-facade-load-state-convergence-design.md
  - docs/superpowers/plans/2026-05-18-asset-facade-load-state-convergence.md
  - docs/superpowers/specs/2026-05-23-asset-readiness-diagnostics-design.md
  - docs/superpowers/plans/2026-05-23-asset-readiness-diagnostics.md
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
  - zircon_runtime/src/asset/tests/facade.rs::load_states_separate_root_direct_and_recursive_dependency_state
  - zircon_runtime/src/asset/tests/facade.rs::dependency_load_state_applies_direct_precedence_and_missing_records
  - zircon_runtime/src/asset/tests/facade.rs::load_states_for_missing_wrong_kind_and_non_resident_roots_do_not_restore_payloads
  - zircon_runtime/src/core/resource/tests.rs::register_ready_preserves_current_diagnostics_and_replaces_stale_diagnostics
  - zircon_runtime/src/asset/facade/event.rs::tests::typed_asset_events_roundtrip_for_tooling_snapshots
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_exposes_loaded_dependency_rows_and_record_diagnostics
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_and_load_states_roundtrip_for_tooling_snapshots
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_marks_missing_and_wrong_kind_roots_without_restoring_payloads
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_marks_missing_dependency_records_as_failed_rows
  - zircon_runtime/src/asset/tests/facade.rs::readiness_report_keeps_shallowest_direct_dependency_row_and_terminates_cycles
  - zircon_runtime/src/asset/tests/pipeline/manager.rs::asset_manager_service_reports_importer_capabilities_before_and_after_project_open
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
- `AssetEvent<TAsset>` is also a serde DTO. It serializes the typed handle id, locator, previous locator, revision, and stable snake-case event variant names for editor/tool event snapshots without requiring the asset payload itself to serialize. `AssetEventKind` mirrors that payload-free category, and `AssetEvent<TAsset>` exposes `event_kind()`, `locator()`, `previous_locator()`, and `revision()` for metadata-only event consumers.
- `AssetLoadState` maps missing records, import/runtime loading, loaded resident payloads, failed resources, and Zircon's explicit `Reloading` state.
- `DependencyLoadState` maps direct dependency aggregate state using the same Zircon state vocabulary as root loads, including explicit `Reloading`.
- `AssetLoadState`, `DependencyLoadState`, and `RecursiveDependencyLoadState` serialize their public states as stable snake-case strings such as `loaded`, `failed`, and `reloading` for editor/tool snapshots.
- `AssetLoadStates` groups root, direct dependency, and recursive dependency state so status panels and runtime callers can query one coherent typed state tuple derived from one root record observation without forcing payload residency. The tuple is a serde DTO, so tooling can persist and compare the same state surface that runtime code reads. It also exposes classification helpers for loaded, not-loaded, loading-class, and failed status rows.
- `AssetReadinessReport` is the typed diagnostic snapshot for a requested asset handle. It contains the root readiness node, aggregate `AssetLoadStates`, and per-dependency readiness rows.
- `AssetReadinessNode` carries the requested asset id, optional locator/kind/revision metadata, root load state, and `ResourceDiagnostic` entries from the current record plus synthetic missing/wrong-kind diagnostics.
- `AssetDependencyReadiness` carries dependency id, optional locator/kind/revision metadata, direct/recursive depth, direct-edge classification, load state, and diagnostics.
- `AssetReadinessReport`, `AssetReadinessNode`, and `AssetDependencyReadiness` are serde DTOs. They preserve IDs, locators, kind, revision, state, depth/direct classification, and diagnostics across JSON roundtrips without invoking import or residency logic.
- `ProjectAssetManager::readiness_report<TAsset>(handle)` is read-only. It does not restore payloads, run importers, touch artifacts, invoke graphics, or mutate lease-driven residency.
- `ProjectAssetManager` now exposes `load<TAsset>(locator)`, `handle<TAsset>(locator)`, `assets<TAsset>()`, `load_state(handle)`, `dependency_load_state(handle)`, `load_states(handle)`, `readiness_report<TAsset>(handle)`, `is_loaded(handle)`, `is_loaded_with_direct_dependencies(handle)`, `is_loaded_with_dependencies(handle)`, `recursive_dependency_load_state(handle)`, `asset_load_state_by_id<TAsset>(id)`, `subscribe_asset_events<TAsset>()`, and importer capability report helpers.
- The public `AssetManager` service trait forwards the importer capability report helpers so tools that resolve the manager through the core service boundary can query importer availability without downcasting to `ProjectAssetManager`.
- `MeshAsset` is now part of this typed facade through `AssetKind::Mesh`, `MeshMarker`, `Handle<MeshAsset>`, and `Assets<MeshAsset>`.
- The top-level `zircon_runtime::asset` facade also re-exports shared asset validation helpers such as `validate_wgsl_captures(...)`, so documented fixtures, shader import, and public callers use the same shader/material capture contract.

## Bevy Alignment And Zircon Divergence

The facade follows Bevy's typed `Handle<T>`, `Assets<T>`, typed asset events, and load-state vocabulary, but it preserves Zircon's existing storage and residency model. A `Handle<TAsset>` is a cheap identity value; it does not keep a payload resident by itself. Residency is still controlled by `ResourceLease<T>` from `Assets<TAsset>::acquire`, and the last lease drop may unload the payload until `ProjectAssetManager` rehydrates it from project artifacts or builtins.

`AssetLoadState::Reloading` is a Zircon extension. It is treated as a loading-class state through `is_loading_class()` while keeping the explicit state visible for diagnostics and hot-reload UI.

M2 adds dependency graph behavior without moving authority out of `ResourceManager`. `ImportedAssetEntry.dependencies` is persisted into the project meta document as locator data for each root or labeled subasset entry, then the completed `ProjectManager` registry resolves those locators to `ResourceRecord.dependency_ids`. This two-phase resolution lets forward references within a project scan resolve to the target asset's UUID-derived `ResourceId` instead of a locator-derived fallback ID. Unresolved dependency locators become `ResourceDiagnostic::error("unresolved asset dependency ...")` on the owning record.

M3 adds labeled subasset identity to the same facade path. Importers now return `AssetImportOutcome.entries`; the root entry keeps the source locator, and each subasset uses the existing `ResourceLocator` label syntax. The project scanner stores a separate `ResourceRecord` and artifact for each entry, so a typed `Handle<TAsset>` for `res://bundle.multi#Texture0` points at `AssetId::from_asset_uuid(entry.uuid)`. Scene/project references resolve by `AssetReference.uuid` first and use `AssetReference.url` only as a repair locator, while unknown label loads use `AssetImportError::MissingAssetLabel` so editor diagnostics can distinguish “source missing” from “subasset label missing”.

`ProjectAssetManager::recursive_dependency_load_state(handle)` now walks `ResourceRecord.dependency_ids` from the root handle. The root asset must first be directly `Loaded`; otherwise the root state is returned. Recursive dependency aggregation uses precedence `Failed > Reloading > Loading > NotLoaded > Loaded`, treats missing dependency records as `Failed`, and protects against cycles with a visited ID set. This deliberately diverges from Bevy's unknown-dependency behavior, which can remain indefinitely loading; Zircon reports missing graph edges deterministically because project/editor diagnostics need stable failure rows.

Readiness reports walk `ResourceRecord.dependency_ids` breadth-first. Direct dependencies are depth `1`; nested dependencies use increasing depth. Duplicate dependency rows keep the shallowest depth and set `direct = true` when any direct edge exists. Missing dependency records produce failed rows with synthetic `ResourceDiagnostic::error(...)` entries so editor repair views do not collapse into indefinite loading.

`ProjectAssetManager::dependency_load_state(handle)` aggregates only first-level dependencies from the root record's `dependency_ids` with the same precedence, while `recursive_dependency_load_state(handle)` walks the full dependency tree. Both direct and recursive aggregation treat missing dependency records as `Failed`. The direct query validates that the root record exists and has the requested asset kind, but it does not require the root payload to be resident. A ready root with no direct dependencies can therefore report root `NotLoaded`, direct dependency `Loaded`, and recursive dependency `NotLoaded` after the last lease drops. `load_states(handle)` returns these three views together from one root record observation, and the loaded predicates require the root to be `Loaded` before direct or recursive dependency success can make the combined predicate true. This is a coherent typed state tuple, not a global atomic snapshot of the full resource graph under concurrent mutation.

All facade state queries are read-only. They use registry records, runtime state, dependency IDs, and current payload residency, but they never call `ensure_resident`; polling `dependency_load_state(handle)`, `load_states(handle)`, `readiness_report<TAsset>(handle)`, or the `is_loaded*` predicates cannot reload artifacts or mutate lease-driven residency.

Ready resource registration preserves the diagnostics supplied on the current `ResourceRecord`. A clean successful reimport supplies an empty diagnostics list and clears stale diagnostics; a successful import with warnings or unresolved dependency diagnostics keeps those diagnostics visible after runtime sync.

Importer capability reporting now lives beside the same facade entry point. `AssetImporterCapabilityReport` pairs the importer descriptor with `AssetImporterCapabilityStatus::Available` or `DiagnosticOnly`, and `ProjectAssetManager::asset_importer_capability_report_for_source(...)` can answer the expected importer status before a source file is scanned. The same query is exposed through the object-safe `AssetManager` service trait, so editor and tooling callers do not need a concrete manager reference just to surface importer availability. This keeps diagnostic-only backends such as FBX/USD/DAE/3DS visible to tools without pretending that they can produce runtime assets.

## Render Readiness Boundary

Asset readiness reports stop at source/import/resource/dependency readiness. Render-specific material readiness remains owned by `RenderMaterialReadinessReport` and graphics resource preparation, where shader contracts, texture upload support, fallback policy, and GPU/device constraints are known.

## Event Invariant

Typed event filtering must work even after removal, when the registry entry no longer exists. For that reason `ResourceEvent` now carries `resource_kind` at emission time. Resource producers in `ResourceManager` fill this from the affected `ResourceRecord`, and typed asset receivers can filter without querying mutable registry state. `Assets<TAsset>::subscribe_events()` returns `AssetEventReceiver<TAsset>` instead of a bare channel so dropping the typed receiver also disconnects the filter thread from its shutdown channel.

Serialized typed events are snapshot DTOs, not a replay log contract. They preserve identity, locator transition data, snake-case event variant, and revision for tooling, but consumers should still treat `ResourceManager` and the project meta/artifact stores as the authority for current asset state. `AssetEventKind` uses the same snake-case vocabulary as the serialized event variants for tools that need a detached event category.

## Test Coverage

`zircon_runtime/src/asset/tests/facade.rs` covers typed handle kind mismatch, typed payload access, acquire/release residency behavior, event filtering across kinds including removed events, rename/reload/remove event ordering, load-state mapping, direct and recursive dependency graph state, missing dependency failure, wrong concrete payload residency, failed reload preserving last-good payload, `Assets<TAsset>::insert/remove_by_locator`, and `ProjectAssetManager` generic load/handle/state/event entry points. `zircon_runtime/src/asset/facade/event.rs::tests::typed_asset_events_roundtrip_for_tooling_snapshots` verifies JSON roundtrips for typed added and renamed asset events, including handle id, locator, previous locator, snake-case variant, revision, `AssetEventKind`, and the read-only event metadata accessors. The load-state convergence coverage includes `load_states_separate_root_direct_and_recursive_dependency_state`, `dependency_load_state_applies_direct_precedence_and_missing_records`, and `load_states_for_missing_wrong_kind_and_non_resident_roots_do_not_restore_payloads`, which verify `AssetLoadStates`, `DependencyLoadState`, missing and wrong-kind root mapping, non-resident root read-only behavior, direct versus recursive aggregation, and the `is_loaded*` predicates. `zircon_runtime/src/asset/facade/load_state.rs::tests::asset_load_states_classification_helpers_cover_tooling_status_rows` covers the metadata-only classification helpers used by tool status rows.

The readiness diagnostics coverage includes `readiness_report_exposes_loaded_dependency_rows_and_record_diagnostics`, `readiness_report_and_load_states_roundtrip_for_tooling_snapshots`, `readiness_report_marks_missing_and_wrong_kind_roots_without_restoring_payloads`, `readiness_report_marks_missing_dependency_records_as_failed_rows`, and `readiness_report_keeps_shallowest_direct_dependency_row_and_terminates_cycles`. These tests verify root and dependency diagnostics, synthetic missing and wrong-kind diagnostics, read-only non-resident root behavior, missing dependency rows, shallowest/direct row merging, cycle termination, JSON roundtrips for readiness reports, and snake-case state field output for tooling snapshots. `zircon_runtime/src/core/resource/tests.rs::register_ready_preserves_current_diagnostics_and_replaces_stale_diagnostics` verifies that runtime ready registration preserves current diagnostics while allowing clean reimports to clear stale diagnostics.

`zircon_runtime/src/asset/tests/assets/importer.rs` covers importer capability reports, including diagnostic-only model formats that remain intentionally non-producing. `zircon_runtime/src/asset/tests/pipeline/manager.rs::asset_manager_service_reports_importer_capabilities_before_and_after_project_open` covers the service-trait path before a project is open and after the pending importer registry is applied to the active project. `zircon_runtime/src/asset/tests/project/manager.rs` covers entry dependency resolution into `ResourceRecord.dependency_ids`, unresolved dependency diagnostics, restart restore through the meta-persisted dependency locator list, root plus labeled subasset artifact persistence, duplicate label failure records, and structured unknown-label load errors. `zircon_runtime/src/asset/tests/project/package_assets.rs` covers direct and manifest-based package asset roots, package subasset UUID/artifact restore, unknown package lookup, malformed package roots, and structured missing-label errors for `package://` sources. `zircon_runtime/src/asset/tests/project/zmeta.rs` covers `.zmeta` schema generation, ignored old sidecars, UUID-first stale-URL lookup, restored entry URL remapping after source rename, and subasset UUID preservation across transient failed reimport. `zircon_runtime/src/core/resource/tests.rs` covers dependency ID changes as revision-bearing record changes.

The 2026-05-20 runtime `--tests` check also covers the facade-level `validate_wgsl_captures(...)` re-export used by the documented `.zshader` fixture; the first run exposed the missing top-level export and the rerun passed after restoring that public surface.

## Validation Evidence

2026-05-26 asset event metadata accessor closeout:

- `rustfmt --edition 2021 --check zircon_runtime/src/asset/facade/event.rs zircon_runtime/src/asset/facade/mod.rs zircon_runtime/src/asset/mod.rs` passed.
- `git diff --check -- zircon_runtime/src/asset/facade/event.rs zircon_runtime/src/asset/facade/mod.rs zircon_runtime/src/asset/mod.rs docs/zircon_runtime/asset/facade.md .codex/sessions/20260526-0941-asset-event-metadata.md` passed with line-ending warnings only.
- `cargo check -p zircon_runtime --lib --tests --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --message-format short --color never` passed with existing warnings only.
- `cargo test -p zircon_runtime --lib typed_asset_events_roundtrip_for_tooling_snapshots --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --no-run --message-format short --color never` passed and produced `D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin/debug/deps/zircon_runtime-a42efe1f3ce69c19.exe`.
- `D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin/debug/deps/zircon_runtime-a42efe1f3ce69c19.exe asset::facade::event::tests::typed_asset_events_roundtrip_for_tooling_snapshots --exact --test-threads=1` passed: 1 passed, 0 failed, 2067 filtered out.

2026-05-26 asset facade event snapshot DTO closeout:

- `rustfmt --edition 2021 --check zircon_runtime/src/asset/facade/event.rs` passed.
- `AssetEvent<TAsset>` JSON event variants are emitted as stable snake-case names (`added`, `renamed`, `modified`, `removed`, `reload_failed`) to match the load-state/readiness snapshot naming style.
- `cargo test -p zircon_runtime --lib typed_asset_events_roundtrip_for_tooling_snapshots --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --message-format short --color never -- --test-threads=1` passed: 1 passed, 0 failed, 2065 filtered out.
- `cargo test -p zircon_runtime --lib typed_asset_events --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --message-format short --color never -- --test-threads=1` passed: 3 passed, 0 failed, 2063 filtered out.
- `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --message-format short --color never` passed for default runtime lib and tests with existing warnings only.
- `git diff --check -- zircon_runtime/src/asset/facade/event.rs docs/zircon_runtime/asset/facade.md .codex/sessions/20260526-0103-asset-event-snapshot-continuation.md` passed with line-ending warnings only.

2026-05-26 asset facade serializable DTO closeout:

- `rustfmt --edition 2021 --check zircon_runtime/src/asset/facade/load_state.rs zircon_runtime/src/asset/facade/readiness.rs zircon_runtime/src/asset/tests/facade.rs` passed after formatting the touched Rust files.
- `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --message-format short --color never` passed for the focused runtime library surface with existing warnings only.
- `cargo test -p zircon_runtime --lib readiness_report_and_load_states_roundtrip_for_tooling_snapshots --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --message-format short --color never -- --test-threads=1` passed: 1 passed, 0 failed, 2062 filtered out.
- `cargo test -p zircon_runtime --lib readiness_report --no-default-features --features core-min --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --message-format short --color never -- --test-threads=1` passed: 12 passed, 0 failed, 2051 filtered out.
- `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-serializable-dtos-coremin --message-format short --color never` passed for default runtime lib and tests with existing warnings only.
- `git diff --check -- zircon_runtime/src/asset/facade/load_state.rs zircon_runtime/src/asset/facade/readiness.rs zircon_runtime/src/asset/tests/facade.rs docs/zircon_runtime/asset/facade.md .codex/sessions/20260526-0000-asset-facade-continuation.md` passed with line-ending warnings only.

2026-05-24 asset readiness diagnostics closeout used focused validation instead of claiming workspace-wide runtime success:

- `rustfmt --edition 2021 --check zircon_runtime/src/asset/facade/readiness.rs zircon_runtime/src/asset/facade/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/core/resource/manager/payload_ops.rs zircon_runtime/src/core/resource/tests.rs zircon_runtime/src/asset/tests/facade.rs` initially reported formatting drift in the Milestone 1/2 Rust files. The same file list was formatted with `rustfmt --edition 2021 ...`, the diff was inspected, and the focused `--check` rerun passed.
- `cargo fmt --all --check` passed.
- `cargo test -p zircon_runtime --lib register_ready_preserves_current_diagnostics_and_replaces_stale_diagnostics --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1` passed: 1 passed, 0 failed, 1960 filtered out.
- `cargo test -p zircon_runtime --lib readiness_report --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1` passed: 7 passed, 0 failed, 1954 filtered out.
- `cargo test -p zircon_runtime --lib facade --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics -- --test-threads=1` passed: 29 passed, 0 failed, 1932 filtered out.
- `cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics` passed for `zircon_runtime` lib and tests with existing warnings.
- `git diff --check -- zircon_runtime/src/asset/facade/readiness.rs zircon_runtime/src/asset/facade/mod.rs zircon_runtime/src/asset/mod.rs zircon_runtime/src/core/resource/manager/payload_ops.rs zircon_runtime/src/core/resource/tests.rs zircon_runtime/src/asset/tests/facade.rs docs/zircon_runtime/asset/facade.md docs/superpowers/specs/2026-05-23-asset-readiness-diagnostics-design.md docs/superpowers/plans/2026-05-23-asset-readiness-diagnostics.md .codex/sessions/20260523-0744-asset-stack-next-slice-design.md` exited successfully with line-ending warnings only.
