---
related_code:
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/template/asset/surface_index.rs
  - zircon_runtime/src/ui/template/asset/hot_reload_plan.rs
  - zircon_runtime_interface/src/ui/tree/node/template_node_metadata.rs
  - zircon_runtime/src/ui/tests/runtime_ui_asset_surface_index.rs
  - zircon_runtime/src/ui/tests/runtime_ui_layout_routes.rs
  - zircon_runtime/src/ui/tests/mod.rs
implementation_files:
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_manager.rs
  - zircon_runtime/src/ui/runtime_ui/runtime_ui_fixture.rs
  - zircon_runtime/src/ui/tests/runtime_ui_asset_surface_index.rs
  - zircon_runtime/src/ui/tests/mod.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture plan code
  - docs/plans/zircon_editor/editor_ui/05-ui-asset-management.md
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
tests:
  - rustfmt --edition 2021 zircon_runtime\src\ui\runtime_ui\runtime_ui_manager.rs zircon_runtime\src\ui\runtime_ui\runtime_ui_fixture.rs zircon_runtime\src\ui\tests\runtime_ui_asset_surface_index.rs zircon_runtime\src\ui\tests\mod.rs
  - cargo check -p zircon_runtime --lib --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-surface-node-index-0612-coremin --message-format short --color never (2026-06-12: passed before unrelated render-chain errors appeared; later rerun blocked by unrelated render pass command-list errors)
  - cargo test -p zircon_runtime --lib runtime_ui_asset_surface_index --no-default-features --features core-min --locked --jobs 1 --target-dir E:\cargo-targets\zircon-editor-ui-runtime-index-0612-coremin-check --message-format short --color never -- --test-threads=1 --nocapture
doc_type: module-detail
---

# Runtime UI Manager

`RuntimeUiManager` is the runtime-owned retained UI host used by the editor UI architecture before the editor shell fully cuts over. It owns the active `UiSurface`, the V2 prototype file cache, the theme registry, and the input router. Host/editor code feeds normalized runtime events into this manager instead of directly juggling a surface plus separate dispatchers.

## Fixture Asset Identity

`RuntimeUiFixture` now exposes two identities:

- `asset_id()`: the stable logical tree id used for runtime diagnostics and `UiTreeId`.
- `asset_uri()`: the `res://...v2.ui.toml` source URI that matches asset watch events and hot-reload classification.

Both are registered for a loaded fixture. The logical id keeps existing runtime frame/debug behavior stable, while the URI lets `UiAssetHotReloadPlan` classify template changes by suffix and target the loaded surface through `UiAssetSurfaceIndex`.

## Surface Index Registration

After `load_builtin_fixture(...)` successfully loads and compiles a V2 document, the manager records the active surface in `UiAssetSurfaceIndex`:

- the fixture logical id,
- the fixture source URI,
- every import/resource reference returned by `ui_v2_asset_references(root_document)`.

After the surface computes layout, the manager also calls `UiAssetSurfaceIndex::record_tree_node_resources(&surface.tree)`. That production hook scans retained template node metadata for explicit resource URIs and `{ kind, uri, fallback }` tables, registering node-level resource edges when authored assets provide them. Existing runtime fixtures mostly use literal chrome values rather than real `res://` resource nodes, so the hook is primarily a forward-compatible data path for editor shell/runtime fixture assets that start using icon/font/texture resources.

The index is updated only after the surface computes layout successfully, so failed fixture loads do not replace the active runtime surface registration. Current fixture files do not yet import external theme/icon/font resources, but the registration path is already wired for those references.

## Hot Reload Bridge

The runtime manager exposes `asset_surface_index()` for runtime tests and future hot-reload executors. A watch event for `res://ui/runtime/fixtures/quest_log_dialog.v2.ui.toml` can now flow through:

1. `UiAssetDependencyIndex::apply_watch_changes(...)`
2. `UiAssetHotReloadPlan::from_watch_report(...)`
3. `UiAssetSurfaceIndex::target_surfaces_for_plan(...)`

and resolve back to the loaded `runtime.ui.quest_log_dialog` surface.

The bridge now registers both whole-surface asset ownership and optional node-level resource ownership. Template rebuilds still target whole surfaces. Non-structural theme/resource plans can use node-level dirtying when coverage is complete; otherwise they fall back to root-level dirtying in `UiAssetSurfaceIndex`.

## Current Coverage

`runtime_ui_asset_surface_index` covers:

- loaded fixture surfaces registering their logical id and `res://` source URI;
- reverse lookup from fixture source URI to `UiTreeId`;
- template file modification plans targeting the active runtime fixture surface for rebuild.
- production fixture load path invoking node-resource registration after surface construction.

Focused validation is currently blocked by unrelated scene/runtime and render-chain compile failures recorded in the session note, so this module documents the intended command and relies on rustfmt plus the last successful `zircon_runtime --lib core-min` check until those blockers clear.
