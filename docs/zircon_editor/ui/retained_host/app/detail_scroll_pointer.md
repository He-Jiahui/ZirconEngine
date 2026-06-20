---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/asset_browser.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/console.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/inspector.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/asset_browser.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/console.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/inspector.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app detail-scroll pointer console/inspector/asset-browser subowner ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never
doc_type: module-detail
---

# Detail Scroll Pointer Events

`app/detail_scroll_pointer.rs` is the structural retained-host entry for detail-pane scroll callbacks. It declares the child event owners only; scroll extent calculation and UI writeback remain in `app/pointer_layout/detail_scrolls.rs`.

## Panel Owners

`app/detail_scroll_pointer/console.rs` owns Console scroll callbacks. It commits the latest pointer layout, focuses the callback source window, resolves the Console callback surface size, syncs the Console scroll layout from the current status line, handles scroll input, and writes the resulting state back to UI globals.

`app/detail_scroll_pointer/inspector.rs` owns Inspector scroll callbacks. It resolves Inspector callback surface size, syncs Inspector layout, handles scroll input, and writes Inspector scroll state back.

`app/detail_scroll_pointer/asset_browser.rs` owns Asset Browser selected-asset detail scroll callbacks. It resolves Asset Browser callback surface size, reads the current asset-browser snapshot, syncs asset-detail layout, handles scroll input, and writes detail scroll state back.

## Boundary Rules

- Keep `app/detail_scroll_pointer.rs` as a structural entry only.
- Keep Console scroll event handling in `app/detail_scroll_pointer/console.rs`.
- Keep Inspector scroll event handling in `app/detail_scroll_pointer/inspector.rs`.
- Keep Asset Browser selected-asset detail scroll event handling in `app/detail_scroll_pointer/asset_browser.rs`.
- Keep scroll extent computation and pane-surface UI writeback helpers in `app/pointer_layout/detail_scrolls.rs`.
- Keep callback surface-size fallback policy in `app/helpers/callback_surface/surface_size.rs`.

## Validation Notes

The 2026-06-19 detail-scroll pointer panel subowner split reduced `detail_scroll_pointer.rs` from 93 lines to a 3-line structural entry. `detail_scroll_pointer/console.rs` is 33 lines, `detail_scroll_pointer/inspector.rs` is 32 lines, and `detail_scroll_pointer/asset_browser.rs` is 34 lines.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app detail-scroll pointer console/inspector/asset-browser subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
