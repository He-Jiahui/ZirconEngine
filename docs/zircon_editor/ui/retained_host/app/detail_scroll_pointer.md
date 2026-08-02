---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/asset_browser.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/console.rs
  - zircon_editor/src/ui/retained_host/app/detail_scroll_pointer/inspector.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/detail_scrolls.rs
  - zircon_editor/src/ui/retained_host/app/helpers/callback_surface/surface_size.rs
  - zircon_editor/src/ui/retained_host/console_output/viewport_size.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/presentation.rs
  - zircon_editor/src/ui/retained_host/scroll_surface_host.rs
  - zircon_editor/src/ui/retained_host/detail_pointer/scroll_surface_pointer_layout.rs
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

`app/detail_scroll_pointer/console.rs` owns Console scroll callbacks. It commits the latest pointer layout, focuses the callback source window, resolves the Console callback surface size, syncs the Console scroll layout from the bounded Console output, handles scroll input, and writes the resulting state back to UI globals. Native scroll routing resolves that callback surface from the projected `ConsoleOutputPaintMetadata.viewport`, so hit testing and the maximum scroll use the same body height as clipping and scrollbar paint instead of the enclosing pane height. Console content extent is the logical line count multiplied by the shared fixed Runtime Text row height; it no longer guesses wrapped height from character width.

Console synchronization uses the explicit follow-tail policy on `ScrollSurfaceHostState`: output growth advances to the new maximum only while the previous offset was at the previous maximum. Scrolling upward suspends following and preserves the user's reading position; returning to the tail enables following again. Generic Inspector and Asset Browser synchronization retains its existing position-preserving behavior. Both policies read the pointer bridge state back after synchronization, so content shrink and Clear Console publish the clamped offset instead of leaving stale host state.

The authoritative host recompute path initializes the Console scroll-surface size from the already-applied `ConsoleOutputPaintMetadata.viewport` before follow-tail synchronization. This keeps the first overflowing output and later window/layout changes correct even when no scroll callback has occurred. The host contract exposes a narrow borrowed query instead of cloning the full presentation; callback-source floating/native windows take priority, followed by local docks and remaining floating surfaces. The scroll callback keeps its immediate surface-size update for pointer input between recomputes.

`app/detail_scroll_pointer/inspector.rs` owns Inspector scroll callbacks. It resolves Inspector callback surface size, syncs Inspector layout, handles scroll input, and writes Inspector scroll state back.

`app/detail_scroll_pointer/asset_browser.rs` owns Asset Browser selected-asset detail scroll callbacks. It resolves Asset Browser callback surface size, reads the current asset-browser snapshot, syncs asset-detail layout, handles scroll input, and writes detail scroll state back.

## Boundary Rules

- Keep `app/detail_scroll_pointer.rs` as a structural entry only.
- Keep Console scroll event handling in `app/detail_scroll_pointer/console.rs`.
- Keep Inspector scroll event handling in `app/detail_scroll_pointer/inspector.rs`.
- Keep Asset Browser selected-asset detail scroll event handling in `app/detail_scroll_pointer/asset_browser.rs`.
- Keep scroll extent computation and pane-surface UI writeback helpers in `app/pointer_layout/detail_scrolls.rs`.
- Keep maximum-offset geometry on `ScrollSurfacePointerLayout`, shared by bridge clamping and host follow-tail policy.
- Keep follow-tail selection on the Console call site; do not apply it implicitly to all scroll surfaces.
- Keep recompute-time Console viewport discovery in `console_output/viewport_size.rs`, reading projected metadata through the narrow host-window query.
- Keep callback surface-size fallback policy in `app/helpers/callback_surface/surface_size.rs`.

## Validation Notes

The 2026-06-19 detail-scroll pointer panel subowner split reduced `detail_scroll_pointer.rs` from 93 lines to a 3-line structural entry. `detail_scroll_pointer/console.rs` is 33 lines, `detail_scroll_pointer/inspector.rs` is 32 lines, and `detail_scroll_pointer/asset_browser.rs` is 34 lines.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app detail-scroll pointer console/inspector/asset-browser subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-08-01 Console follow-tail forward fix added source regressions for initial tail positioning, growth while following, preserving an upward-scrolled position, resuming follow after returning to the tail, and bridge-clamp readback after content shrink. Independent review then found that production recompute did not initialize the Console surface until its first scroll callback. The forward fix now reads the applied projection viewport during every authoritative recompute and adds a real-host regression covering first overflow without pointer input. Targeted Rust 2024 formatting, ownership scans, and scoped diff checks passed; narrow independent re-review reported P0-P2 = 0. Managed Cargo and screenshot validation remain pending and are not claimed here.
