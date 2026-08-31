---
related_code:
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/template/asset/binding_reload_transaction.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_program.rs
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
  - zircon_runtime_interface/src/ui/template/document.rs
  - zircon_runtime_interface/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/surface/component_state.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/template/asset/binding_reload_transaction.rs
  - zircon_runtime/src/ui/template/asset/compiler/binding_program.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/optimize/zircon_runtime/74-runtime-ui-template-component-binding-expression-model-event-command-hot-reload-product-integration-review.md
tests:
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\hot_reload_executor.rs zircon_runtime\src\ui\template\asset\hot_reload_plan.rs zircon_runtime\src\ui\template\asset\surface_index.rs zircon_runtime\src\ui\template\asset\compiler\cache\compile_cache.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_hot_reload_executor.rs zircon_runtime\src\ui\tests\asset_surface_index.rs zircon_runtime\src\ui\tests\asset_hot_reload_plan.rs zircon_runtime\src\ui\tests\mod.rs
  - cargo test -p zircon_runtime --lib asset_hot_reload_executor --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture (blocked before tests by unrelated scene test exhaustiveness errors after executor test correction)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never (blocked by unrelated core runtime extension registry errors)
doc_type: module-detail
---

# UI Asset Hot Reload Executor

`UiAssetHotReloadExecutor` is the runtime-state combiner for the retained UI hot-reload path. It does not replace `UiAssetHotReloadPlan`; instead, it applies one already-built plan to the runtime state fragments that currently exist: compile cache, resource resolver, theme registry, registered surfaces, and the surface targeting index.

Template compilation remains host-owned. A host can install `UiAssetSurfaceRebuilder` to prepare replacement surfaces from changed assets. The executor owns the transaction boundary: every active replacement must prepare successfully before cache, resource, theme, or published surface state changes.

## Inputs

`UiAssetHotReloadPlan::execute_runtime_reload(...)` accepts:

- a mutable `UiAssetCompileCache`,
  - a mutable `UiAssetSurfaceIndex`,
- a mutable map of currently retained `UiSurface` values by `UiTreeId`,
- an optional mutable `UiResourceResolver`,
- an optional mutable `UiThemeRegistry`,
- an optional mutable `UiAssetSurfaceRebuilder`,
- an optional `UiThemeDocument`.

The optional theme document is only applied when the plan contains theme restyle assets. Non-theme plans do not mutate the active theme registry even if a document is accidentally supplied.

## Behavior

Execution performs two phases:

1. Prepare: clone only targeted retained surfaces; require a rebuilder for every active template target; prepare all replacement trees; reject tree-id mismatches; validate the replacement binding IR and root asset identity; reject generation reuse for different IR; migrate compatible component state by a unique `(component, control_id)` key or `(component, node_path)` fallback; clear transient input/focus/navigation state; and apply dirty marking to the staged map.
2. Publish: after all fallible preparation succeeds, evict compile cache entries, invalidate requested resource handles, apply an optional theme document, replace the retained surface map entries, drop each retired surface, and atomically replace the surface index's compiled node/binding ownership generation.

Expanded template nodes retain their source asset and one source asset per binding. The compiler interns those owners into `UiCompiledAssetId` values, and `UiAssetSurfaceIndex` publishes typed `UiCompiledNodeId` and generation-qualified `UiCompiledBindingHandle` reverse edges. Imported widget changes therefore target their owning surfaces without scanning every compiled binding.

`UiAssetTemplateRebuildReceipt` reports the affected assets and migrated/reset state counts per surface. Its `UiBindingQuiescenceReceipt` records old/published generations, retired/published binding counts, state migration counts, whether the old generation was actually retired, stale-handle rejection, and whether that retired generation is quiescent after publication. A same-program publication does not retire its generation, so `old_generation_retired`, `stale_handles_rejected`, and `old_generation_quiescent` remain false while its current handles remain valid. Duplicate stable keys, missing nodes, and changed component identities fail closed to reset instead of copying ambiguous state. Persistent values and durable component flags are retained for compatible nodes; focused, focus-visible, hovered, pressed, dragging, drop-hovered, active-drag-target, popup, input, focus, and navigation state are reset. The previous window state is retained.

`RebuilderRequired`, `PrepareFailed`, `TreeIdMismatch`, malformed replacement IR, valid-to-invalid generation replacement, missing or changed root asset identity, and generation collision return before shared runtime state is mutated. This preserves the last-known-good surfaces, compiled ownership index, and compile cache when any member of a multi-surface batch fails.

## Boundary

The executor does not:

- load changed source files,
- choose or own the template compiler used by the host rebuilder,
- update the asset dependency graph after recompilation,
- perform GPU uploads,
- compute node-level damage,
- retire external callback/model subscription leases,
- install itself into Editor or gameplay asset-watch ownership,
- own editor authoring diagnostics.

Those remain host/runtime integration work under Runtime11A/64 and the parent Runtime74 hot-reload item. This module supplies the reusable two-phase publication core and does not by itself close the product-level hot-reload gate.

## Current Coverage

`zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs` covers:

- combined template/theme/font reload execution: cache eviction, theme document application, resource refresh report, and dirty surface roots.
- resource-only reload execution: no template cache eviction, no theme reload, resource refresh report, and render-only dirty flags.
- multi-surface prepare failure preserving all last-known-good surfaces and the compile cache.
- atomic replacement publication with compatible state migration, incompatible state reset, and transient state cleanup.
- 1,000 stable component states migrated through one staged surface publication.
- source-asset ownership for compiled nodes and caller/component bindings.
- generation replacement, old-handle rejection, ownership-index publication, and quiescence receipt integrity.
- valid-to-empty-program rejection and same-program publication without false retirement or quiescence claims.
- a release-only 21-pair ownership lookup P95 gate against full-program scans.

The Runtime74 grouped coordinator validation is pending. No current Cargo pass is claimed here.
