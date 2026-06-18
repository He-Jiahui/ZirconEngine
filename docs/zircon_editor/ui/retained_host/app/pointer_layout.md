---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs
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
  - zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/welcome_recent.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app pointer-layout asset-surface ownership scan
  - app pointer-layout welcome/detail-scroll ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Pointer Layouts

`app/pointer_layout.rs` owns retained-host pointer layout synchronization for shell-level Workbench surfaces. It keeps the top-level sync methods for menu state, activity rail, host pages, document tabs, drawer headers, hierarchy rows, and the shared pane-surface host accessor used by child modules.

The module bridges computed Workbench model/layout frames into pointer bridges and writes hover/scroll state back into `UiHostContext` or `PaneSurfaceHostContext` globals. It should stay focused on layout/state synchronization and avoid accumulating pointer event dispatch behavior, which belongs in the dedicated pointer action modules.

## Asset Surfaces

`app/pointer_layout/asset_surfaces.rs` owns asset surface pointer layout synchronization for the activity Assets pane and the Asset Browser pane. It syncs folder tree, content list, references list, and used-by list layouts from `AssetWorkspaceSnapshot`, writes each surface mode's hover/scroll state back to the pane-surface host globals, and exposes app-internal accessors used by `asset_tree_pointer.rs`, `asset_content_pointer.rs`, `asset_content_pointer/target.rs`, `asset_reference_pointer.rs`, and `asset_reference_pointer/target.rs`.

The helper methods use `pub(in crate::ui::retained_host::app)` because sibling pointer dispatch modules need to read or mutate the shared `AssetSurfacePointerState`, but the API should not widen beyond the retained-host app subtree.

## Welcome Recent

`app/pointer_layout/welcome_recent.rs` owns Welcome recent-project list pointer layout synchronization. It resolves the pane size from the retained template bridge when necessary, syncs recent project paths from the editor chrome snapshot, updates the bridge with current scroll/hover state, and writes hovered row/action state back to `PaneSurfaceHostContext`.

The retained-host app visibility is intentional because `welcome_recent_pointer.rs` and `host_lifecycle.rs` both need to drive the same Welcome recent layout/state writeback path without duplicating sizing or hover-state policy.

## Detail Scrolls

`app/pointer_layout/detail_scrolls.rs` owns retained detail scroll layouts for the Console, Inspector, and browser asset details pane. It computes scroll extents from the status line, inspector content, and selected asset details, syncs each `ScrollSurface`, and writes scroll offsets back to `PaneSurfaceHostContext`.

Keeping these scroll surfaces in a child module separates pane detail scrolling from shell-level menu/tab/drawer pointer layouts and keeps the root module from accumulating each panel's scroll extent policy.

## Boundary Rules

- Keep menu, activity rail, host page, document tab, drawer header, and hierarchy layout sync in `app/pointer_layout.rs`.
- Keep asset tree/content/reference/used-by layout sync, asset surface hover/scroll UI writeback, asset surface state accessors, and asset reference list layout helpers in `app/pointer_layout/asset_surfaces.rs`.
- Keep Welcome recent-project list size resolution, layout sync, and hover/scroll UI writeback in `app/pointer_layout/welcome_recent.rs`.
- Keep Console, Inspector, and browser asset detail scroll layout sync plus scroll-offset UI writeback in `app/pointer_layout/detail_scrolls.rs`.
- Keep asset tree pointer event dispatch in `app/asset_tree_pointer.rs`, content pointer dispatch entry points in `app/asset_content_pointer.rs`, shared content-list target preparation in `app/asset_content_pointer/target.rs`, reference/used-by dispatch entry points in `app/asset_reference_pointer.rs`, and shared reference-list target preparation in `app/asset_reference_pointer/target.rs`.
- Keep asset surface state declarations in `app/asset_surface_pointer_state.rs`; do not move state DTO definitions into layout sync modules.

## Validation Notes

The 2026-06-18 asset-surface split reduced `pointer_layout.rs` from 403 lines to 256 lines. `pointer_layout/asset_surfaces.rs` is 154 lines and owns asset surface layout sync, pane-surface host state writeback, asset surface pointer-state accessors, asset workspace snapshot selection, and asset reference list layout construction. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pointer-layout asset-surface ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 welcome/detail-scroll split reduced `pointer_layout.rs` from 284 lines to 146 lines. `pointer_layout/welcome_recent.rs` is 74 lines and owns Welcome recent-project list sizing, layout sync, and hover/action state writeback. `pointer_layout/detail_scrolls.rs` is 77 lines and owns Console, Inspector, and browser asset detail scroll layout sync and scroll-offset state writeback. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app pointer-layout welcome/detail-scroll ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
