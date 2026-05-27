---
related_code:
  - zircon_runtime/src/asset/facade/load_state.rs
  - zircon_runtime/src/asset/facade/manager.rs
  - zircon_runtime/src/asset/facade/mod.rs
  - zircon_runtime/src/asset/project/manager/scan_and_import.rs
  - zircon_runtime/src/asset/pipeline/manager/project_asset_manager/runtime.rs
  - zircon_runtime/src/core/resource/manager/payload_ops.rs
  - zircon_runtime_interface/src/resource/diagnostic.rs
  - zircon_runtime_interface/src/resource/resource_record.rs
  - zircon_runtime/src/core/framework/render/material/readiness_report.rs
  - zircon_runtime/src/core/framework/render/material/validation_error.rs
plan_sources:
  - .codex/plans/Bevy-Style Asset Stack Completion Plan.md
  - .codex/plans/Runtime 吸收层与 Editor_Scene 边界收束计划.md
  - .codex/plans/全系统重构方案.md
  - user: 2026-05-23 approved dependency/readiness diagnostics as the next asset-management optimization slice
reference_sources:
  - dev/bevy/crates/bevy_asset/src/server/mod.rs
  - dev/bevy/crates/bevy_asset/src/server/info.rs
  - dev/Fyrox/fyrox-resource/src/state.rs
tests:
  - planned: cargo test -p zircon_runtime --lib facade --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics
  - planned: cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics
doc_type: design-spec
---

# Asset Readiness Diagnostics Design

## Goal

Add a read-only asset readiness diagnostics surface that explains why a typed asset is or is not ready, without creating a second asset store or changing import identity, artifact layout, hot reload, mesh metadata, or render preparation.

The immediate consumer is editor/tooling status logic that needs more than aggregate `AssetLoadStates`: it needs the root record diagnostics, direct dependency causes, recursive dependency causes, and deterministic missing-dependency failures.

## Current Shape

`ProjectAssetManager` already exposes typed root/direct/recursive load-state queries through `AssetLoadState`, `DependencyLoadState`, `RecursiveDependencyLoadState`, and `AssetLoadStates`. These queries are intentionally read-only and do not call `ensure_resident`.

Importer diagnostics already flow into `ResourceRecord.diagnostics` through `ImportedAssetEntry.diagnostics` and unresolved dependency diagnostics from `scan_and_import.rs`. Failed imports also produce `ResourceState::Error` records with `ResourceDiagnostic::error(...)`.

The current gap is diagnostic visibility after the aggregate state is known:

- Dependency state says `Failed`, `Reloading`, `Loading`, or `NotLoaded`, but not which dependency caused it.
- Missing dependency records are treated as failed by the facade, but no structured missing dependency row is exposed.
- Successful import diagnostics can be lost when a ready payload is registered because `ResourceManager::register_ready` clears diagnostics on the caller-supplied record.
- Render material readiness already has detailed `RenderMaterialReadinessReport`, but it is a render-preparation report, not the general asset dependency graph report.

## Architecture

The owner is `zircon_runtime::asset::facade`. Authoritative identity, state, dependency IDs, diagnostics, revisions, and payload residency remain in `zircon_runtime::core::resource` and `ResourceRecord`.

Add a focused readiness diagnostics module, tentatively `zircon_runtime/src/asset/facade/readiness.rs`, and keep `facade/manager.rs` as a thin public entry surface. If the implementation can stay smaller without mixing public DTOs and traversal helpers into `manager.rs`, use the new module directly.

The public typed entry point should be one read-only report query:

```rust
ProjectAssetManager::readiness_report<TAsset>(handle: Handle<TAsset>) -> AssetReadinessReport
```

This API returns a value, not a `Result`, matching the existing observation APIs. Missing roots, wrong-kind roots, non-resident ready records, missing dependency records, and failed dependencies are represented in the report state and diagnostics.

## Data Model

The report should be small, serializable-friendly, and derived from existing records:

```rust
pub struct AssetReadinessReport {
    pub root: AssetReadinessNode,
    pub load_states: AssetLoadStates,
    pub dependencies: Vec<AssetDependencyReadiness>,
}

pub struct AssetReadinessNode {
    pub id: AssetId,
    pub locator: Option<AssetUri>,
    pub kind: Option<AssetKind>,
    pub revision: Option<u64>,
    pub load_state: AssetLoadState,
    pub diagnostics: Vec<ResourceDiagnostic>,
}

pub struct AssetDependencyReadiness {
    pub id: AssetId,
    pub locator: Option<AssetUri>,
    pub kind: Option<AssetKind>,
    pub revision: Option<u64>,
    pub depth: u32,
    pub direct: bool,
    pub load_state: AssetLoadState,
    pub diagnostics: Vec<ResourceDiagnostic>,
}
```

The exact field names can be adjusted during planning, but the design intent is fixed:

- `root` describes the requested handle and includes synthetic diagnostics for missing or wrong-kind roots.
- `load_states` reuses the already approved aggregate semantics and loaded predicates.
- `dependencies` lists direct and recursive dependency rows with enough record metadata for UI/tooling.
- Missing dependency records produce rows with `locator/kind/revision = None`, `load_state = Failed`, and a synthetic `ResourceDiagnostic::error("missing asset dependency record ...")`.

No new diagnostic severity enum is needed. Reuse `ResourceDiagnostic` so importer, unresolved dependency, missing record, and report synthesis all share one asset diagnostic vocabulary.

## Traversal Semantics

The report derives from the current root record observation plus registry lookups. It must not restore payloads, load artifacts, invoke importers, run render preparation, or mutate lease state.

Root handling:

- Missing root record: root `NotLoaded`, direct dependency `NotLoaded`, recursive dependency `NotLoaded`, synthetic error diagnostic.
- Wrong root kind for `TAsset`: same aggregate state as missing root, but include the actual kind in the synthetic diagnostic.
- Correct record kind: use `AssetLoadState::from_resource(...)` with the same payload-residency check as `load_states`.
- Ready record without a resident payload remains `NotLoaded`; this is not automatically an error because lease-driven residency can unload payloads.

Dependency handling:

- Direct dependencies are `root.dependency_ids` with `depth = 1` and `direct = true`.
- Recursive dependencies are discovered by walking each dependency's `dependency_ids` with a visited-ID guard.
- If the same dependency appears through multiple paths, keep the shallowest row and mark it direct if any direct edge exists.
- Cycles must not loop forever. The initial implementation may skip already visited nodes without emitting a cycle diagnostic; cycle reporting can be a later enhancement.
- Missing dependency records are deterministic failed rows, preserving the facade's current divergence from Bevy's indefinite loading behavior.

Aggregate state precedence remains unchanged:

1. `Failed`
2. `Reloading`
3. `Loading`
4. `NotLoaded`
5. `Loaded`

## Diagnostics Preservation

`ResourceManager::register_ready` should preserve diagnostics supplied on the current `ResourceRecord` instead of clearing them unconditionally. This does not keep stale errors alive if producers pass a fresh ready record with no diagnostics; it only stops dropping current successful-import diagnostics and unresolved dependency diagnostics during runtime resource registration.

The invariant becomes:

- The caller-supplied `ResourceRecord.diagnostics` is the current diagnostic truth for the record.
- A successful reimport with no diagnostics replaces previous diagnostics with an empty list.
- A successful reimport with warnings or unresolved dependency diagnostics keeps those diagnostics visible after runtime sync.
- Failed import records continue to store error diagnostics without payload registration.

## Reference Alignment

Bevy provides the dominant asset-state shape. `AssetServer::get_load_states`, `get_load_state`, `get_dependency_load_state`, `get_recursive_dependency_load_state`, and loaded predicates split root, direct dependency, and recursive dependency readiness (`dev/bevy/crates/bevy_asset/src/server/mod.rs:1210`). Bevy also tracks loading and failed dependency sets and propagates failure up dependents (`dev/bevy/crates/bevy_asset/src/server/info.rs:430`).

Fyrox provides the stabilizing resource diagnostics precedent. Its resource state carries `LoadError { path, error }` and exposes a displayable failure state (`dev/Fyrox/fyrox-resource/src/state.rs:275`). This supports Zircon's choice to attach path/record diagnostics to readiness rather than hiding causes behind a boolean or aggregate enum.

Zircon deliberately diverges from Bevy in two places:

- `Reloading` remains explicit because hot-reload diagnostics and UI need to distinguish reload work from first-load work.
- Missing dependency records are `Failed`, not indefinite `Loading`, because project/editor tools need deterministic repair rows.

## Render Readiness Boundary

Do not merge render material readiness into the asset facade in this slice. `RenderMaterialReadinessReport` remains the render-preparation report for shader contracts, texture upload support, fallback policies, and blocking material validation. The asset readiness report may document that material/render readiness is a downstream consumer, but it should not call `ResourceStreamer`, inspect GPU capabilities, or require graphics state.

This keeps ownership clear:

- `zircon_runtime::asset` explains source/import/resource/dependency readiness.
- `zircon_runtime::graphics` explains render prepare/cache readiness.

## Validation Plan

Focused runtime facade tests should cover:

- Report for a fully loaded root with loaded direct and recursive dependencies.
- Report rows include direct versus recursive depth and preserve shallowest/direct classification.
- Root diagnostics from `ResourceRecord.diagnostics` appear in the report.
- Dependency diagnostics from `ResourceRecord.diagnostics` appear on the dependency row.
- Missing dependency records appear as failed dependency rows with synthetic diagnostics.
- Missing root and wrong-kind root produce value reports with `NotLoaded` aggregate states and synthetic diagnostics.
- Ready but non-resident roots do not call `ensure_resident` and remain non-mutating.
- Cyclic dependency graphs terminate and do not duplicate rows endlessly.
- `register_ready` preserves current ready-record diagnostics and clears stale diagnostics when given a fresh record with an empty diagnostics list.

Scoped validation should use:

```powershell
cargo test -p zircon_runtime --lib facade --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics
cargo check -p zircon_runtime --lib --tests --locked --jobs 1 --target-dir D:/cargo-targets/zircon-asset-readiness-diagnostics
```

Workspace-level success should not be claimed unless fresh workspace validation is run and passes. Existing unrelated formatting, lockfile, and active Cargo queue blockers must be reported instead of hidden.

## Documentation Plan

Update `docs/zircon_runtime/asset/facade.md` after implementation. The doc should cover:

- `AssetReadinessReport` and dependency row semantics.
- Read-only/no-residency behavior.
- Diagnostics preservation through ready resource registration.
- Bevy alignment and Zircon divergences.
- The render readiness boundary with `RenderMaterialReadinessReport`.
- Focused tests and validation evidence.

Do not update mesh metadata or glTF docs in this slice while the active mesh metadata session owns those files.

## Out Of Scope

- Asset identity or `.zmeta` schema changes.
- Importer output contract changes.
- Hot-reload scheduling or watcher behavior changes.
- Mesh metadata, glTF morph/skin propagation, or plugin glTF edits.
- Render preparation, GPU texture upload, shader cache, or material runtime changes.
- Editor UI rendering of the report.
- Root `Cargo.lock`, dependency version changes, or workspace-wide validation claims.
