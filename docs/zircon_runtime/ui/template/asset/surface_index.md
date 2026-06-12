---
related_code:
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime/src/ui/template/asset/dependency_index.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/surface/surface/rebuild.rs
  - zircon_runtime_interface/src/ui/event_ui/reflection.rs
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime/src/ui/tests/asset_surface_index.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/runtime_ui_asset_surface_index.rs
  - zircon_runtime/src/ui/tests/mod.rs
implementation_files:
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_executor.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/template/asset/mod.rs
  - zircon_runtime/src/ui/template/mod.rs
  - zircon_runtime/src/ui/tests/asset_surface_index.rs
  - zircon_runtime/src/ui/tests/asset_hot_reload_executor.rs
  - zircon_runtime/src/ui/tests/runtime_ui_asset_surface_index.rs
  - zircon_runtime/src/ui/tests/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
tests:
  - rustfmt --edition 2021 zircon_runtime\src\ui\template\asset\surface_index.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_surface_index.rs zircon_runtime\src\ui\tests\mod.rs
  - cargo test -p zircon_runtime --lib asset_surface_index --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-node-index-0612-coremin --message-format short --color never (2026-06-12: passed, existing warnings only)
  - rustfmt --edition 2021 --check zircon_runtime\src\ui\template\asset\surface_index.rs zircon_runtime\src\ui\template\asset\mod.rs zircon_runtime\src\ui\template\mod.rs zircon_runtime\src\ui\tests\asset_surface_index.rs (2026-06-12: passed after node resource auto-registration)
  - cargo test -p zircon_runtime --lib asset_surface_index --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-node-index-0612-coremin --message-format short --color never -- --test-threads=1 --nocapture (2026-06-12: blocked during unrelated lib-test compilation by plugin extension dyn-compat errors and mesh indirect-args Eq errors before the filtered tests ran)
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-node-index-0612-coremin --message-format short --color never (2026-06-12 later rerun: blocked by unrelated render pass command-list errors in scene_renderer_core_render_compiled_scene/render.rs, render_pass_execution_context/gpu.rs, and viewport_overlay_renderer/record_meshes.rs)
doc_type: module-detail
---

# UI Asset Surface Index

`UiAssetSurfaceIndex` is the M2 runtime data surface between hot-reload planning and retained UI surfaces. `UiAssetDependencyIndex` knows which compiled UI assets are affected by a changed asset, and `UiAssetHotReloadPlan` classifies the resulting work. The surface index answers the next question: which current `UiSurface` instances should receive rebuild, restyle, or resource-damage dirty domains.

The index is registration-based. Runtime surfaces do not yet carry durable asset-to-node ownership metadata, so this module records surface-level ownership and references explicitly instead of trying to infer subtrees from current node attributes.

## Stored State

The module keeps two synchronized maps:

- `assets_by_surface`: each `UiTreeId` to the ordered, deduplicated UI asset/resource URI list associated with that surface.
- `surfaces_by_asset`: each UI asset/resource URI to the sorted set of `UiTreeId` values that currently own or reference it.
- `node_assets_by_surface`: optional per-surface node registrations from `UiNodeId` to the asset/resource URIs used by that node.
- `nodes_by_asset`: each UI asset/resource URI to sorted `(UiTreeId, UiNodeId)` targets.

`record_surface_assets(tree_id, asset_ids)` replaces a surface registration. It first removes stale reverse edges, deduplicates non-empty asset strings while preserving caller order, then writes both maps. This lets a surface be rebuilt from a different template or theme without leaving old references live.

`record_node_assets(tree_id, node_id, asset_ids)` is the manual precision path. It replaces one node's asset edges without rewriting the surface-level registration. This lets runtime tree builders, compiled-template consumers, or editor diagnostics progressively attach exact asset ownership as that metadata becomes available.

`record_tree_node_resources(tree)` is the automatic precision path for already-built retained trees. It first removes any previous node-resource edges for the tree id, then scans each current node's `UiTemplateNodeMetadata` maps (`attributes`, `slot_attributes`, and `style_overrides`) for resource URI strings and explicit `{ kind, uri, fallback }` resource tables. It registers primary URIs and valid fallback URIs, returns `UiAssetSurfaceNodeResourceRegistrationReport`, and reports current nodes without resource-bearing metadata. Clearing the old tree's node edges before rebuilding prevents hot reload from keeping resource references for nodes that no longer exist.

`record_compiled_surface(tree_id, compiled)` is the compiler-facing helper. It registers the compiled document asset id, every compiled `UiResourceDependency` URI, and fallback resource URIs. It does not register imported widget/style documents unless their dependencies were present in the compiled dependency list; the dependency index remains the source for template-to-template cascade relationships.

## Hot Reload Targeting

`target_surfaces_for_plan(plan)` maps each plan queue onto surface ids:

- `template_rebuild_targets` become `template_rebuild_surfaces`.
- `removed_compiled_assets` become `removed_compiled_surfaces`.
- `theme_restyle_assets` plus `theme_restyle_targets` become `theme_restyle_surfaces`.
- `resource_refresh_assets` plus `resource_damage_targets` become `resource_damage_surfaces`.

The returned `UiAssetSurfaceHotReloadTargets` also mirrors the plan's aggregate `UiDirtyFlags` and `rebuild_required` flag. Surface lists are ordered by the underlying `UiTreeId` set for deterministic tests and diagnostics.

`target_nodes_for_plan(plan)` mirrors the same queues onto optional `(tree_id, node_id)` targets. It is a diagnostics and precision-damage data surface; structural template rebuilds remain surface-level because a changed component can alter child structure, slots, input routes, and visible ranges beyond the currently registered node.

`mark_target_surfaces_dirty(plan, surfaces)` applies the most precise safe dirtying available:

- for non-structural theme/resource plans where every target for a surface has node coverage, it marks only those nodes and writes `node_dirty_reports`;
- if a target surface has no node coverage or mixed surface-level and node-level coverage, it falls back to `plan.mark_surface_roots_dirty(...)`;
- if the plan requires template rebuild, it always uses root-level dirtying.

Missing registered surfaces are reported in `UiAssetSurfaceHotReloadApplyReport::missing_surfaces`. Missing registered nodes are reported in `UiAssetHotReloadNodeDirtyReport::missing_nodes` instead of aborting the whole batch.

`UiAssetHotReloadExecutor` consumes this index when applying a plan to runtime state. The index remains pure and reusable; the executor owns the side effects of cache eviction, theme reload, and dirty marking.

## Runtime UI Manager Integration

`RuntimeUiManager::load_builtin_fixture(...)` now records the active fixture surface after the V2 document compiles and computes layout. The registration includes the fixture's stable logical id, the `res://...v2.ui.toml` source URI exposed by `RuntimeUiFixture::asset_uri()`, and all V2 import/resource references collected from the loaded root document.

The source URI entry is required for hot reload: asset watch events are expressed as `res://` URIs and `UiAssetHotReloadPlan` classifies template work by URI suffix. The logical id remains registered for diagnostics and runtime tree identity. This is the first production-side caller of the surface index; editor shell cutover can reuse the same registration shape when it moves from fixture surfaces to full workbench documents.

## Boundary

This module does not:

- load or recompile UI assets,
- mutate `UiThemeRegistry`,
- refresh icon, texture, or font resource handles,
- own editor authoring state.

Its precise dirty application is opt-in and node-level only. It does not infer subtree ownership, GPU atlas regions, or resource handles; callers must register exact node asset edges directly or call `record_tree_node_resources(...)` after building a retained tree. Coverage gaps deliberately fall back to root-level dirtying to avoid partial restyle or partial resource updates.

## Current Coverage

`zircon_runtime/src/ui/tests/asset_surface_index.rs` covers:

- surface asset registration, deduplication, and stale reverse-edge replacement.
- node asset registration, deduplication, and stale reverse-edge replacement.
- automatic node resource registration from `UiTemplateNodeMetadata`.
- stale node-resource edge cleanup when a rebuilt tree no longer contains an old node.
- compiled document resource-dependency registration, including fallback resources.
- theme and resource hot-reload plans mapping to the expected surfaces.
- resource hot-reload plans mapping to precise nodes when node edges exist.
- root-level dirty marking and missing-surface diagnostics.
- precise node dirty marking and missing-node diagnostics.
- mixed node/surface targets falling back to root dirtying.
- template rebuild plans targeting the surface that owns the compiled asset.
- runtime fixture loading registering the active surface by both logical id and `res://` template URI, then mapping a template watch plan back to that surface.

Focused validation in the `core-min` profile passed: the filtered `asset_surface_index` lib test ran 5 tests with 0 failures. Existing unrelated runtime warnings remain outside this UI asset slice.

After adding node-level precision indexing, `cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-node-index-0612-coremin --message-format short --color never` passed with existing warnings only.

The updated filtered `asset_surface_index` lib-test command is currently blocked before the filtered tests run by unrelated lib-test compilation errors: `zircon_runtime/src/tests/plugin_extensions/extension_registry_bridge.rs` has dyn-compat / `PluginInterface` errors for `WeatherQueryInterface`, and `zircon_runtime/src/graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_batcher.rs` derives equality over `IndexedIndirectArgs` without `PartialEq`/`Eq`.

The newer `runtime_ui_asset_surface_index` filtered validation command is currently blocked before test execution by unrelated scene/runtime compile failures; see the active session note for the exact blocker list.
