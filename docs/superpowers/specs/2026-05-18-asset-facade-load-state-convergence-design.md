---
related_code:
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/facade/asset.rs
  - zircon_runtime/src/asset/facade/handle.rs
  - zircon_runtime/src/asset/facade/assets.rs
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/loading/ensure_resident.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime/src/core/resource/manager/lease_ops.rs
  - zircon_runtime/src/core/resource/runtime.rs
  - zircon_runtime/src/asset/tests/facade.rs
plan_sources:
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - user: 2026-05-18 chose typed facade/load state as the next asset-management optimization target
reference_sources:
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
tests:
  - cargo test -p zircon_runtime --lib facade --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-load-state-convergence
  - cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-load-state-convergence
doc_type: design-spec
---

# Asset Facade Load-State Convergence Design

## Goal

Converge Zircon's typed asset facade around a Bevy-style load-state query surface while keeping all identity, payload residency, dependency IDs, diagnostics, revisions, and events in `zircon_runtime::core::resource::ResourceManager`.

This is the next asset-management optimization after the image/texture import milestone. It is a lower-layer facade cleanup, not a new loader, watcher, artifact, or render-preparation milestone.

## Current Shape

`ProjectAssetManager` already exposes typed `load<TAsset>`, `handle<TAsset>`, `assets<TAsset>`, `load_state`, `recursive_dependency_load_state`, `asset_load_state_by_id`, and typed event subscription helpers. `Assets<TAsset>` already provides typed payload access and root load-state projection over `ResourceManager`.

The gap is that callers cannot query root, direct dependency, and recursive dependency state as one coherent typed result. Direct dependency state also has no first-class facade enum, so the existing recursive query has to carry more conceptual weight than it should.

## Architecture

The owner is `zircon_runtime::asset::facade`. The authoritative storage remains `zircon_runtime::core::resource`; the facade only reads `ResourceRecord`, `RuntimeResourceState`, and typed payload presence.

The design adds a small query model:

- `DependencyLoadState`: direct dependency aggregate state.
- `AssetLoadStates`: combined root, direct dependency, and recursive dependency state DTO.
- `ProjectAssetManager::load_states<TAsset>(handle)`: typed combined query.
- `ProjectAssetManager::dependency_load_state<TAsset>(handle)`: direct dependency query.
- Thin convenience predicates over `AssetLoadStates`: `is_loaded`, `is_loaded_with_direct_dependencies`, and `is_loaded_with_dependencies`.

No new asset store, cache, compatibility layer, or cross-crate facade is introduced.

## Data Flow

`handle<TAsset>(locator)` remains a locator-to-typed-handle lookup and does not force residency. `load<TAsset>(locator)` remains the explicit typed load path: it validates the record kind, calls `ensure_resident`, validates the typed payload, and returns the typed handle.

Query data flows from existing resource state:

- Root state uses the existing `AssetLoadState::from_resource(record, runtime_state, has_payload)` projection.
- Direct dependency state walks only `ResourceRecord.dependency_ids` for the root record.
- Recursive dependency state walks the full dependency tree with the existing visited-ID cycle guard.
- `AssetLoadStates` combines those three projections without changing residency.

Aggregation precedence remains deterministic:

1. `Failed`
2. `Reloading`
3. `Loading`
4. `NotLoaded`
5. `Loaded`

This keeps project/editor diagnostics stable. Missing dependency records are `Failed`, not an indefinite loading state.

## API Semantics

Query APIs do not return `Result`. Missing records, wrong typed handles, and missing dependencies map into load states:

- Missing root record: root `NotLoaded`, direct dependency `NotLoaded`, recursive dependency `NotLoaded`.
- Wrong root kind for `TAsset`: same as missing root for typed query APIs.
- Ready record without a resident typed payload: root `NotLoaded`.
- Correct record kind with a wrong concrete payload type: root `NotLoaded` for query APIs.
- Missing dependency record: dependency aggregate `Failed`.
- Root `Reloading` or `Failed`: combined loaded-with-dependencies predicates return false even if previous payload remains resident.

Acquisition APIs keep errors:

- `handle<TAsset>` returns `CoreError` for missing locator or kind mismatch.
- `load<TAsset>` returns `CoreError` for missing locator, kind mismatch, non-ready record, no project for restorable assets, missing artifacts, and wrong concrete typed payload.

## Bevy Alignment And Zircon Divergence

Bevy exposes `AssetServer::get_load_states`, `get_load_state`, `get_dependency_load_state`, `get_recursive_dependency_load_state`, and convenience loaded predicates. Zircon should adopt the same conceptual split because it is a proven asset-facing vocabulary and avoids forcing recursive dependency state to answer direct-dependency questions.

Zircon deliberately diverges in two places:

- `Reloading` remains explicit because Zircon's hot-reload UI and diagnostics already distinguish reload work from first-load work.
- Missing dependency records resolve to `Failed` because the project scan/import pipeline records unresolved dependency diagnostics and editor rows need deterministic failure states.

## Error Handling

The facade separates observation from acquisition:

- Observation methods return state values and never mutate residency.
- Acquisition methods return `Result` and may call `ensure_resident`.

This prevents state polling from reloading assets, avoids hidden artifact IO in UI/status code, and preserves the current lease-based residency model.

## Module Boundaries

Keep root wiring thin. If the implementation makes `facade/manager.rs` materially larger or mixes aggregation helpers with public API definitions, move aggregation helpers into a focused `facade/load_states.rs` module and re-export only the public DTOs from `facade/mod.rs`.

Do not modify:

- Scan/import behavior.
- Hot-reload watcher behavior.
- Artifact persistence.
- Render asset preparation.
- Root `Cargo.lock` or dependency versions.

## Validation Plan

Facade tests should cover:

- Combined states for a fully loaded root with loaded direct and recursive dependencies.
- Direct dependency `Reloading`, `Loading`, `NotLoaded`, and `Failed` precedence.
- Recursive dependency state remains separate from direct dependency state.
- Missing dependency records map to dependency `Failed`.
- Missing or wrong-kind root typed handle maps to `NotLoaded` states for queries.
- Ready record with no resident payload maps root to `NotLoaded` and does not call `ensure_resident`.
- `is_loaded`, `is_loaded_with_direct_dependencies`, and `is_loaded_with_dependencies` return true only for the matching root/direct/recursive loaded combinations.

Scoped validation should use package-level runtime commands first:

```powershell
cargo test -p zircon_runtime --lib facade --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-load-state-convergence
cargo check -p zircon_runtime --lib --tests --locked --offline --jobs 1 --target-dir D:/cargo-targets/zircon-asset-facade-load-state-convergence
```

If Windows workspace validation remains blocked by the known root lockfile `wgpu-hal` mixed `windows` crate issue, do not claim workspace-wide success. Record the blocker and use WSL/Linux evidence only if the implementation needs CI-parity confirmation.

## Out Of Scope

- Async loading semantics.
- Strong-handle residency policy changes.
- Dependency graph persistence changes.
- Importer diagnostics redesign.
- Watcher invalidation and hot-reload scheduling changes.
- Editor UI changes.
- Plugin workspace changes.
