---
related_code:
  - zircon_runtime/src/ui/template/asset/dependency_index.rs
  - zircon_runtime/src/ui/template/asset/watch_invalidation.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/asset/watch/asset_change.rs
  - zircon_runtime/src/asset/watch/asset_change_kind.rs
  - zircon_runtime/src/asset/assets/ui.rs
  - zircon_runtime/src/ui/tests/asset_dependency_index.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/asset_surface_index.rs
  - zircon_runtime/src/ui/tests/mod.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/dependency_index.rs
  - zircon_runtime/src/ui/template/asset/watch_invalidation.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - zircon_runtime/src/ui/tests/asset_dependency_index.rs
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\dependency_index.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_dependency_index.rs zircon_runtime\src\ui\tests\mod.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-dependency-index-0612-coremin-check --message-format short --color never
  - cargo test -p zircon_runtime --lib asset_dependency_index --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-dependency-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\watch_invalidation.rs zircon_runtime\src\ui\template\asset\dependency_index.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_dependency_index.rs zircon_runtime\src\ui\tests\mod.rs
  - cargo test -p zircon_runtime --lib asset_dependency_index --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-dependency-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12 watch invalidation slice: passed, 8 passed / 0 failed / 3563 filtered out)
  - cargo test -p zircon_runtime --lib asset_hot_reload_plan --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-hot-reload-plan-0612-coremin-check-meta --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib asset_surface_index --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib asset_hot_reload_executor --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (blocked before final rerun by unrelated scene test exhaustiveness errors)
doc_type: module-detail
---

# UI Asset Dependency Index

`UiAssetDependencyIndex` is the runtime-owned query surface for compiled UI asset dependencies. It belongs beside the template asset compiler and prototype-store code because it is populated from compiler-time reference collection, then consumed later by watch, invalidation, and editor resource-browser features.

The index does not read files, talk to the asset watcher, compile templates, or resolve GPU resources. It is a pure in-memory graph over already-normalized `AssetReference` values. That boundary keeps the first M1.S2 slice small while giving later hot-reload and reference-finder work a stable API.

## Stored State

The module keeps two synchronized tables:

- `references_by_asset`: compiled UI asset id to the list of assets it references.
- `dependents_by_asset`: referenced asset id to every compiled UI asset that depends on it.

The keys are locator strings such as `res://ui/theme/base.v2.ui.toml`. Existing collection helpers such as `ui_v2_asset_references` strip component labels before producing `AssetReference` values, so `res://ui/components/button.zui#Button` is indexed at the file asset level as `res://ui/components/button.zui`.

`record_compiled(asset_id, refs)` replaces the previous record for that asset. It first removes old reverse edges, deduplicates the new references by locator string, then writes both forward and reverse maps. That replacement behavior is important for editor hot reload: a template can stop importing an old style file and the old style file must immediately stop listing the template as a dependent.

## Queries

`references_of(asset_id)` returns the forward dependency slice for one compiled UI asset. Missing assets return an empty slice so callers do not need to branch before rendering a dependency panel.

`dependents_of(asset_id)` returns the directly dependent compiled assets for a changed dependency. The iterator is stable because the reverse table uses `BTreeSet`.

`cascade_invalidation_targets(changed)` performs breadth-first traversal over the reverse graph. The changed asset seeds traversal but is not returned as its own target, which prevents self-cycles from scheduling duplicate rebuilds. The returned list is stable and includes transitive dependents, for example `theme -> component -> view`.

`query_asset(asset_id)` builds `UiAssetDependencyQueryReport` for asset-browser and reference-finder consumers. The report includes the queried `asset_id`, the asset's direct outgoing `AssetReference` list, direct incoming dependents, and the full cascade-dependent list that hot reload would affect. This is intentionally locator-level and runtime-owned: editor code can convert the strings into `AssetReferenceSnapshot` rows when it has catalog metadata, while runtime hot reload can consume the same query without depending on editor types.

## Watch Invalidation

`UiAssetDependencyIndex::apply_watch_changes(changes)` is the M2.S1 bridge from the asset watcher to UI-level invalidation. The input is the already-folded `AssetChange` list produced by `AssetWatcher::fold_events(...)`, not raw `notify` events. The output is `UiAssetWatchInvalidationReport`:

- `changed_assets`: every changed locator string in folded order.
- `rebuild_targets`: every direct or transitive dependent that must be recompiled or restyled, deduplicated while preserving first discovery order.
- `removed_assets`: index keys removed because a watched asset disappeared or moved away from its old URI.

Modified and added assets only compute dependents from the changed URI; later stages are responsible for re-recording the asset after import/compile succeeds. Removed assets compute dependents first, then remove their forward and reverse index entries so stale dependencies stop participating in later cascades. Renamed assets invalidate dependents of the old URI, remove the old key, then also invalidate dependents of the new URI. That makes both "old reference now missing" and "new reference now present" consumers visible to the rebuild executor.

The bridge is still pure graph logic. It does not recompile templates, classify document fingerprints, touch `UiInvalidationGraph`, or mark surface damage. The first consumer-side slice is `UiAssetHotReloadPlan`, which routes the report into restyle, rebuild, and resource-refresh queues while leaving actual execution to the next runtime state mutator.

## Hot Reload Plan Consumer

`UiAssetHotReloadPlan::from_watch_report(report)` turns the watch report into ordered action queues:

- template changes/removals set `rebuild_required`, schedule changed templates plus transitive dependents, and evict removed compiled assets instead of re-queueing missing sources.
- theme changes schedule restyle assets and affected targets without setting `rebuild_required`.
- icon/texture changes schedule resource refresh plus render damage.
- font changes schedule resource refresh and mark text/layout/hit-test/render dirty because font data can affect measurement.
- unknown resource suffixes stay visible in `unclassified_assets` and are conservatively routed through resource refresh/render damage.

The first execution helpers now apply the compile-cache and surface-dirty parts of those queues: `UiAssetHotReloadPlan::evict_compile_cache(...)` clears cached compiled documents plus invalidation snapshots, while `mark_surface_roots_dirty(...)` and `rebuild_dirty_surface(...)` apply the aggregate dirty domains to current surface roots. `UiAssetSurfaceIndex` adds the missing registration-based bridge from plan queues to currently retained `UiSurface` instances. `UiAssetHotReloadExecutor` composes cache eviction, optional theme registry reload, surface dirty marking, and refresh-queue reporting. Resource resolver refresh, recompile IO, and precise asset-to-node damage are still owned by later runtime state executor layers.

## Current Coverage

`zircon_runtime/src/ui/tests/asset_dependency_index.rs` covers:

- v2 UI asset imports/resources collected through `ui_v2_asset_references`.
- forward and reverse query results.
- transitive cascade invalidation.
- locator-level dependency query reports for references, direct dependents, and cascade dependents.
- stale reverse-edge removal when an asset is re-recorded.
- asset removal and cycle handling.
- modified watch changes mapped to cascade rebuild targets.
- removed watch changes invalidating dependents before deleting index entries.
- renamed watch changes invalidating both old and new URI dependents.
- cross-change rebuild target deduplication.
- hot-reload action planning for theme, icon, font, texture, template, removed-template, and unknown suffix routing.
- hot-reload execution helpers for compile-cache eviction and root-level surface dirty/rebuild behavior.
- surface-level hot-reload targeting from registered assets/resources to current `UiTreeId` values.
- runtime executor composition for cache eviction, theme reload, dirty surface application, and resource refresh reporting.

Focused validation for the M1.S2 dependency-index slice passed in the `core-min` profile: the library type check completed with existing warnings, and the filtered `asset_dependency_index` test run passed all 4 tests. The M2.S1 watch-invalidation slice extended the same filtered run to 8 tests and passed after the rename/remove bridge was added.
The M2.S2 hot-reload planning/execution-helper slice added separate filtered `asset_hot_reload_plan` and `asset_surface_index` runs, which passed 10 and 5 tests respectively after cold test-target compilation. A broader `cargo check --tests` attempt remains blocked by unrelated `virtual_geometry_debug_snapshot_contract` `RenderMeshSnapshot` initializer errors.
