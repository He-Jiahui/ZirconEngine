---
related_code:
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/watch_invalidation.rs
  - zircon_runtime/src/ui/template/asset/dependency_index.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_plan.rs
  - zircon_runtime/src/ui/tests/asset_surface_index.rs
  - zircon_runtime/src/ui/tests/mod.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/mod.rs
  - zircon_runtime/src/ui/template/asset/compiler/mod.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_plan.rs
  - zircon_runtime/src/ui/tests/asset_surface_index.rs
  - zircon_runtime/src/ui/tests/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\compiler\cache\compile_cache.rs zircon_runtime\src\ui\template\asset\compiler\cache\mod.rs zircon_runtime\src\ui\template\asset\compiler\mod.rs zircon_runtime\src\ui\template\asset\hot_reload_plan.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_hot_reload_plan.rs zircon_runtime\src\ui\tests\mod.rs
  - git diff --check -- zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs zircon_runtime/src/ui/template/asset/compiler/cache/mod.rs zircon_runtime/src/ui/template/asset/compiler/mod.rs zircon_runtime/src/ui/template/asset/hot_reload_plan.rs zircon_runtime/src/ui/template/asset/mod.rs zircon_runtime/src/ui/template/mod.rs zircon_runtime/src/ui/tests/asset_hot_reload_plan.rs zircon_runtime/src/ui/tests/mod.rs docs/zircon_runtime/ui/template/asset/hot_reload_plan.md
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-dependency-index-0612-coremin-check --message-format short --color never
  - cargo test -p zircon_runtime --lib asset_hot_reload_plan --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-hot-reload-plan-0612-coremin-check-meta --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib asset_surface_index --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture
  - cargo test -p zircon_runtime --lib asset_hot_reload_executor --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (blocked before final rerun by unrelated scene test exhaustiveness errors)
doc_type: module-detail
---

# UI Asset Hot Reload Plan

`UiAssetHotReloadPlan` is the M2 consumer-side bridge for `UiAssetWatchInvalidationReport`. The dependency index answers which UI assets are affected by a changed locator; this module translates that answer into runtime actions that later executors can apply.

The plan constructor is deliberately pure. It does not read assets, apply a theme document, recompile templates, mutate a surface, or touch GPU resources. It classifies changed URI strings and returns ordered action queues plus the aggregate `UiDirtyFlags` required by the current batch.

The module also exposes two explicit execution helpers for the bottom of the hot-reload path: compile-cache eviction and coarse surface dirty marking. Those helpers are opt-in side effects so callers can keep import, theme reload, resource resolver refresh, and editor diagnostics ordered around the same plan.

`UiAssetSurfaceIndex` is the companion data surface for multi-surface runtime targeting. It records which retained `UiSurface` instances own or reference compiled UI assets and resource URIs, then maps a plan into template rebuild, theme restyle, and resource damage surface lists.

`UiAssetHotReloadExecutor` is the first runtime-state combiner over those pieces. It applies one plan to compile-cache eviction, optional theme registry reload, and registered surface dirty marking, then reports template rebuild and resource refresh queues for the later asset IO/resource resolver stages.

## Classification

`classify_ui_hot_reload_asset(asset_id)` strips `#label` fragments, lowercases the URI string, and matches the same authoring suffixes used by the asset importer path:

- `Template`: `.zui`, `.v2.ui.toml`, and legacy `.ui.toml`.
- `Theme`: `.theme.toml`.
- `Icon`: `.icon.toml` and direct `.svg` references.
- `Font`: `.font.toml`, `.ttf`, `.otf`, `.ttc`, `.woff`, and `.woff2`.
- `Texture`: common image/container suffixes such as `.png`, `.jpg`, `.dds`, `.ktx2`, `.cube`, and `.psd`.
- `Other`: unknown resources. They remain visible in `unclassified_assets` and are treated as render-resource refreshes so file changes are not silently dropped.

## Plan Fields

`changed_assets` and `removed_assets` mirror the watch report for diagnostics.

`template_rebuild_targets` lists changed templates plus transitive dependents from the dependency index. Removed templates are not re-added to this list; they go to `removed_compiled_assets` so the later executor can evict stale compiled entries and rebuild only remaining dependents.

`theme_restyle_assets` and `theme_restyle_targets` implement the plan-04 rule that theme hot reload should restyle affected surfaces without rebuilding the template tree. The aggregate dirty domains set style, layout, hit-test, render, and text, but do not set input or visible-range dirty.

`resource_refresh_assets` and `resource_damage_targets` cover icon, texture, font, and unknown resource changes. Icon and texture changes only set render dirty. Font changes set text, layout, hit-test, and render dirty because changed font data can alter shaping and measurements.

`rebuild_required` is only set for template changes/removals. Theme and resource changes are intentionally routed through restyle or resource refresh paths.

## Execution Helpers

`UiAssetCompileCache::evict_asset(...)` and `UiAssetCompileCache::evict_assets(...)` remove cached compiled documents whose `compiled.asset.id` matches a target asset id. They also clear the matching invalidation snapshot slot, because cache entries are keyed by compile options while invalidation snapshots are keyed by the `kind:asset_id` slot used by `UiInvalidationGraph`.

`UiAssetHotReloadPlan::evict_compile_cache(cache)` evicts the union of `template_rebuild_targets` and `removed_compiled_assets`. This gives the later reload executor one call for both "source changed, rebuild it" and "source disappeared, remove stale compiled state" cases.

`UiAssetHotReloadPlan::mark_surface_roots_dirty(surface)` applies the aggregate dirty domains to every root in the target `UiSurface` and returns `UiAssetHotReloadSurfaceDirtyReport`. `rebuild_dirty_surface(surface, root_size)` then calls `UiSurface::rebuild_dirty(...)` using the same dirty domains.

Surface dirty application is intentionally coarse in this slice. Runtime surfaces do not yet retain a durable asset-id to node/subtree ownership map, so the helper marks roots instead of guessing precise subtrees. A later executor can narrow this once compiled templates, resource dependencies, and surface metadata expose stable owner mappings.

`UiAssetSurfaceIndex::mark_target_surfaces_dirty(plan, surfaces)` applies that same coarse dirty operation to registered target surfaces and reports stale registrations through `missing_surfaces`. This gives the later runtime state executor a deterministic bridge from plan queues to currently retained surfaces without inventing node-level ownership yet.

`UiAssetHotReloadPlan::execute_runtime_reload(...)` composes the current executable subset. It evicts planned compiled cache entries, applies a supplied `UiThemeDocument` only for theme reload plans, marks indexed surfaces dirty, and returns `UiAssetHotReloadExecutionReport`. It intentionally does not load files, recompile template targets, or refresh GPU resource handles.

## Current Coverage

`zircon_runtime/src/ui/tests/asset_hot_reload_plan.rs` covers:

- theme changes route to restyle and avoid template rebuild.
- icon and texture changes route to resource refresh and render damage.
- font changes mark text/layout/hit-test/render dirty.
- template changes rebuild the changed template and transitive dependents.
- removed templates evict compiled assets without re-queueing the missing source.
- planned template rebuilds evict compiled cache entries and invalidation snapshots.
- planned dirty domains are applied to surface roots and consumed by `UiSurface::rebuild_dirty(...)`.
- plan queues map through `UiAssetSurfaceIndex` to registered runtime surfaces and report stale surface registrations.
- runtime executor entry composes cache eviction, optional theme reload, surface dirty application, and resource refresh reporting.
- suffix classification for theme, icon, SVG, font, template, texture, and unknown resources.

Focused validation in the `core-min` profile passed: the runtime library check completed with existing warnings, the filtered `asset_hot_reload_plan` lib test passed 10 tests, and the filtered `asset_surface_index` lib test passed 5 tests. A broader `cargo check --tests` run was blocked by pre-existing `virtual_geometry_debug_snapshot_contract` `RenderMeshSnapshot` initializer errors outside this UI asset slice.
