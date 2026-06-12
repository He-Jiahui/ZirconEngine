---
related_code:
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/compiler/cache/compile_cache.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/theme/mod.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/mod.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\hot_reload_executor.rs zircon_runtime\src\ui\template\asset\hot_reload_plan.rs zircon_runtime\src\ui\template\asset\surface_index.rs zircon_runtime\src\ui\template\asset\compiler\cache\compile_cache.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_hot_reload_executor.rs zircon_runtime\src\ui\tests\asset_surface_index.rs zircon_runtime\src\ui\tests\asset_hot_reload_plan.rs zircon_runtime\src\ui\tests\mod.rs
  - cargo test -p zircon_runtime --lib asset_hot_reload_executor --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (blocked before tests by unrelated scene test exhaustiveness errors after executor test correction)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never (blocked by unrelated core runtime extension registry errors)
doc_type: module-detail
---

# UI Asset Hot Reload Executor

`UiAssetHotReloadExecutor` is the first runtime-state combiner for the M2 hot-reload path. It does not replace `UiAssetHotReloadPlan`; instead, it applies one already-built plan to the runtime state fragments that currently exist: compile cache, theme registry, registered surfaces, and the surface targeting index.

The executor keeps IO and GPU work outside this layer. It reports `resource_refresh_assets` and `unclassified_assets` for a later consumer-level resource resolver, rather than pretending that icon, texture, or font handles can be refreshed before that resolver exists.

## Inputs

`UiAssetHotReloadPlan::execute_runtime_reload(...)` accepts:

- a mutable `UiAssetCompileCache`,
- a `UiAssetSurfaceIndex`,
- a mutable map of currently retained `UiSurface` values by `UiTreeId`,
- an optional mutable `UiThemeRegistry`,
- an optional `UiThemeDocument`.

The optional theme document is only applied when the plan contains theme restyle assets. Non-theme plans do not mutate the active theme registry even if a document is accidentally supplied.

## Behavior

Execution performs four ordered steps:

1. Evict compiled cache entries and invalidation snapshots for template rebuild targets and removed compiled assets.
2. Apply the supplied theme document to `UiThemeRegistry` when the plan represents a theme reload.
3. Use `UiAssetSurfaceIndex` to find retained surfaces affected by template/theme/resource queues, then mark target surface roots with the plan's aggregate dirty domains.
4. Return `UiAssetHotReloadExecutionReport` with cache eviction counts, theme reload outcome, surface dirty/missing-surface report, template rebuild targets, removed compiled assets, resource refresh assets, and unclassified assets.

This ordering ensures template cache state is invalidated before any later recompile step, while surface damage remains deterministic and reportable even if some registered surfaces have already been dropped.

## Boundary

The executor does not:

- load changed source files,
- recompile template targets,
- update the dependency index after recompilation,
- resolve icon/font/texture handles,
- perform GPU uploads,
- compute node-level damage,
- own editor authoring diagnostics.

Those are the next runtime state executor layers. This module deliberately keeps the current step small enough to validate: plan queues can now mutate cache/theme/surface state without coupling to asset IO or render backends.

## Current Coverage

`zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs` covers:

- combined template/theme/font reload execution: cache eviction, theme document application, resource refresh report, and dirty surface roots.
- resource-only reload execution: no template cache eviction, no theme reload, resource refresh report, and render-only dirty flags.

Focused executor validation is currently blocked before test execution by unrelated active workspace changes. The first executor test run reached execution and showed a semantic issue in the test: theme/font changes should not evict template cache without a template change. The test was corrected to include a template change for cache eviction. The next run stopped during lib-test compilation in unrelated scene tests, and `cargo check --lib` stopped in unrelated core runtime extension registry code. `rustfmt --check` for the touched executor files passed.
