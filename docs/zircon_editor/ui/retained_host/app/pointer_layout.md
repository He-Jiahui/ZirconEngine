---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces/state.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces/sync.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces/ui_writeback.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/hierarchy.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/menu.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/pane_surface_context.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/shell_chrome.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/asset_surface_pointer_state.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces/state.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces/sync.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces/ui_writeback.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/hierarchy.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/menu.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/pane_surface_context.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/shell_chrome.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app pointer-layout asset-surface ownership scan
  - app pointer-layout asset-surface subowner ownership scan
  - app pointer-layout welcome/detail-scroll ownership scan
  - app pointer-layout menu/chrome/hierarchy ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Pointer Layouts

`app/pointer_layout.rs` is the structural entry for retained-host pointer layout synchronization for shell-level Workbench surfaces. Its child modules keep the top-level sync methods for menu state, activity rail, host pages, document tabs, drawer headers, hierarchy rows, detail scrolls, asset surfaces, and the shared pane-surface host accessor used by pointer layout children.

The module bridges computed Workbench model/layout frames into pointer bridges and writes hover/scroll state back into `UiHostContext` or `PaneSurfaceHostContext` globals. It should stay focused on layout/state synchronization and avoid accumulating pointer event dispatch behavior, which belongs in the dedicated pointer action modules.

## Menu And Shell Chrome

`app/pointer_layout/menu.rs` owns menu pointer layout synchronization and menu hover/open state writeback to `UiHostContext`. It builds the menu layout from the Workbench menu model, chrome snapshot, shell size, preset names, active preset, and template outer shell frames.

`app/pointer_layout/shell_chrome.rs` owns Workbench chrome pointer layout synchronization for the activity rail, host page tabs, document tabs, and drawer headers. It bridges Workbench model/layout frames into the corresponding pointer bridges without owning event dispatch.

## Hierarchy And Shared Pane Context

`app/pointer_layout/hierarchy.rs` owns hierarchy pane pointer layout synchronization and hover/scroll state writeback. It converts scene entries into hierarchy node ids only after a valid pane size is known.

`app/pointer_layout/pane_surface_context.rs` owns the app-internal `PaneSurfaceHostContext` accessor shared by pointer layout children. The helper is visible only inside the retained-host app subtree so sibling layout modules can write hover/scroll state without widening UI globals beyond the app boundary.

## Asset Surfaces

`app/pointer_layout/asset_surfaces.rs` is the structural entry for asset surface pointer layout synchronization for the activity Assets pane and the Asset Browser pane.

`app/pointer_layout/asset_surfaces/sync.rs` owns folder tree, content list, references list, and used-by list layout synchronization from `AssetWorkspaceSnapshot`.

`app/pointer_layout/asset_surfaces/ui_writeback.rs` owns each surface mode's hover/scroll state writeback to the pane-surface host globals.

`app/pointer_layout/asset_surfaces/state.rs` owns app-internal asset surface state accessors, snapshot selection for pointer actions, and asset reference list layout helpers used by `asset_tree_pointer.rs`, `asset_content_pointer.rs`, `asset_content_pointer/target.rs`, `asset_reference_pointer.rs`, and `asset_reference_pointer/target.rs`.

The helper methods use `pub(in crate::ui::retained_host::app)` because sibling pointer dispatch modules need to read or mutate the shared `AssetSurfacePointerState`, but the API should not widen beyond the retained-host app subtree.

## Welcome Recent

`app/pointer_layout/welcome_recent.rs` owns Welcome recent-project list pointer layout synchronization. It resolves the pane size from the retained template bridge when necessary, syncs recent project paths from the editor chrome snapshot, updates the bridge with current scroll/hover state, and writes hovered row/action state back to `PaneSurfaceHostContext`.

The retained-host app visibility is intentional because `welcome_recent_pointer.rs` and `host_lifecycle.rs` both need to drive the same Welcome recent layout/state writeback path without duplicating sizing or hover-state policy.

## Detail Scrolls

`app/pointer_layout/detail_scrolls.rs` owns retained detail scroll layouts for the Console, Inspector, and browser asset details pane. It computes scroll extents from the status line, inspector content, and selected asset details, syncs each `ScrollSurface`, and writes scroll offsets back to `PaneSurfaceHostContext`.

Keeping these scroll surfaces in a child module separates pane detail scrolling from shell-level menu/tab/drawer pointer layouts and keeps the root module from accumulating each panel's scroll extent policy.

## Boundary Rules

- Keep `app/pointer_layout.rs` as a structural module entry only.
- Keep menu layout sync and `UiHostContext` menu-state writeback in `app/pointer_layout/menu.rs`.
- Keep activity rail, host page, document tab, and drawer header layout sync in `app/pointer_layout/shell_chrome.rs`.
- Keep hierarchy layout sync and hierarchy hover/scroll writeback in `app/pointer_layout/hierarchy.rs`.
- Keep shared `PaneSurfaceHostContext` access in `app/pointer_layout/pane_surface_context.rs`.
- Keep `app/pointer_layout/asset_surfaces.rs` as the structural asset-surface layout entry.
- Keep asset tree/content/reference/used-by layout sync in `app/pointer_layout/asset_surfaces/sync.rs`.
- Keep asset surface hover/scroll UI writeback in `app/pointer_layout/asset_surfaces/ui_writeback.rs`.
- Keep asset surface state accessors, asset workspace snapshot selection, and asset reference list layout helpers in `app/pointer_layout/asset_surfaces/state.rs`.
- Keep Welcome recent-project list size resolution, layout sync, and hover/scroll UI writeback in `app/pointer_layout/welcome_recent.rs`.
- Keep Console, Inspector, and browser asset detail scroll layout sync plus scroll-offset UI writeback in `app/pointer_layout/detail_scrolls.rs`.
- Keep asset tree pointer event dispatch in `app/asset_tree_pointer.rs`, content pointer dispatch entry points in `app/asset_content_pointer.rs`, shared content-list target preparation in `app/asset_content_pointer/target.rs`, reference/used-by dispatch entry points in `app/asset_reference_pointer.rs`, and shared reference-list target preparation in `app/asset_reference_pointer/target.rs`.
- Keep asset surface state declarations in `app/asset_surface_pointer_state.rs`; do not move state DTO definitions into layout sync modules.

## Validation Notes

The 2026-06-18 asset-surface split reduced `pointer_layout.rs` from 403 lines to 256 lines. `pointer_layout/asset_surfaces.rs` is 154 lines and owns asset surface layout sync, pane-surface host state writeback, asset surface pointer-state accessors, asset workspace snapshot selection, and asset reference list layout construction. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pointer-layout asset-surface ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 asset-surface subowner split reduced `pointer_layout/asset_surfaces.rs` from 154 lines to a 3-line structural entry. `asset_surfaces/state.rs` is 51 lines and owns asset surface state lookup, mutable state lookup, workspace snapshot selection, and reference-list layout helpers. `asset_surfaces/sync.rs` is 43 lines and owns activity/browser asset layout synchronization across tree/content/references/used-by bridges. `asset_surfaces/ui_writeback.rs` is 51 lines and owns pane-surface hover/scroll writeback for activity and browser asset surfaces.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pointer-layout asset-surface subowner ownership scan, and scoped `git diff --check`, all of which passed except for existing CRLF conversion warnings in the dirty worktree. Focused `cargo check` was not rerun for this slice because independent `zircon_runtime` Cargo test processes were still active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 welcome/detail-scroll split reduced `pointer_layout.rs` from 284 lines to 146 lines. `pointer_layout/welcome_recent.rs` is 74 lines and owns Welcome recent-project list sizing, layout sync, and hover/action state writeback. `pointer_layout/detail_scrolls.rs` is 77 lines and owns Console, Inspector, and browser asset detail scroll layout sync and scroll-offset state writeback. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pointer-layout welcome/detail-scroll ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 menu/chrome/hierarchy split reduced `pointer_layout.rs` from 135 lines to 7 lines. `pointer_layout/menu.rs` is 51 lines and owns menu layout sync plus menu-state writeback. `pointer_layout/shell_chrome.rs` is 56 lines and owns activity rail, host page, document tab, and drawer header layout sync. `pointer_layout/hierarchy.rs` is 34 lines and owns hierarchy layout sync plus hierarchy state writeback. `pointer_layout/pane_surface_context.rs` is 9 lines and owns the retained app-internal pane-surface context accessor.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pointer-layout menu/chrome/hierarchy ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 143 warnings, `zircon_editor` 63 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
