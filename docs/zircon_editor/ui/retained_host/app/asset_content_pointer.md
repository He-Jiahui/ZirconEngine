---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/dispatch.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/route.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app asset-content pointer events ownership scan
  - app asset-content pointer press ownership scan
  - app asset-content pointer target ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Asset Content Pointer Dispatch

`app/asset_content_pointer.rs` is the structural retained-host app entry for asset content-list pointer interactions on both activity Assets and Asset Browser surfaces. The child modules handle press, click, move, and scroll entry points, callback-source focus, incompatible drag clearing before a drag press, shared runtime Asset Surface click forwarding, returned host effects, hover/scroll pointer state writes, and asset drag payload creation when an asset content row starts a drag.

The module is the host action boundary for asset content-list interactions. It should not own generic asset surface layout synchronization or pointer bridge internals; those stay in `app/pointer_layout/asset_surfaces.rs` and `zircon_editor/src/ui/retained_host/asset_pointer/content/*`.

## Event Dispatch

`app/asset_content_pointer/events.rs` owns the retained-host event entry points for content lists. It normalizes press/release gates, forwards primary-button press to the press child, prepares content-list targets for click/move/scroll, dispatches clicks through `callback_dispatch::dispatch_shared_asset_content_pointer_click(...)`, applies returned effects, and writes hover/scroll state from prepared bridge dispatch.

Keeping event dispatch in a child module leaves the root as a structural owner and separates normal pointer event flow from drag payload construction and target preparation.

## Press Drag Source

`app/asset_content_pointer/press.rs` owns content-list press handling that can create an asset drag payload. It clears incompatible scene/object drag payloads, prepares and dispatches the content-list press target, writes pointer state, derives a drag payload from the active snapshot, clears asset drag payload state for non-item routes, and reports the drag source summary to the status line.

Keeping press drag-source construction in a child module separates drag source setup from click forwarding, move hover updates, scroll updates, and target preparation.

## Target Preparation

`app/asset_content_pointer/target.rs` owns content-list target preparation for the host action boundary. It resolves the correct `AssetWorkspaceSnapshot` from the surface mode, resolves callback-reported or cached content-list size through the retained-host callback surface sizing helpers, synchronizes the `AssetContentListPointerBridge` with a fresh `AssetContentListPointerLayout`, dispatches prepared bridge events, and writes returned `AssetListPointerState` back into the selected asset surface content list.

The child module centralizes the error policy that was duplicated across press, click, move, and scroll paths. Press preparation clears the active asset drag payload on target errors; click/move/scroll paths preserve the drag payload unless their dispatch result explicitly changes it.

## Boundary Rules

- Keep user-facing retained-host entry points (`asset_content_pointer_event`, `asset_content_pointer_clicked`, `asset_content_pointer_moved`, and `asset_content_pointer_scrolled`) in `app/asset_content_pointer/events.rs`.
- Keep content-list press dispatch, drag payload creation, non-item drag clearing, incompatible scene/object drag clearing, and drag-source status text in `app/asset_content_pointer/press.rs`.
- Keep content-list target preparation, layout synchronization, bridge dispatch wrapping, drag-clear-on-target-error policy, and pointer-state writeback in `app/asset_content_pointer/target.rs`.
- Keep asset surface state declarations and content-list fields in `app/asset_surface_pointer_state.rs`.
- Keep pure content pointer bridge, layout, route, target, and dispatch DTO behavior under `zircon_editor/src/ui/retained_host/asset_pointer/content/`.

## Validation Notes

The 2026-06-19 events split reduced `asset_content_pointer.rs` from 140 lines to 3 lines. `asset_content_pointer/events.rs` is 138 lines and owns content-list press/release gates, click forwarding, move hover updates, scroll updates, and event effect/state writeback. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset content/reference pointer events ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 target split reduced `asset_content_pointer.rs` from 269 lines to 181 lines. `asset_content_pointer/target.rs` is 98 lines and owns the shared target preparation, bridge sync, prepared dispatch, and state writeback helpers.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-content pointer target ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 press-drag split reduced `asset_content_pointer.rs` from 191 lines to 149 lines. `asset_content_pointer/press.rs` is 57 lines and owns content-list press dispatch, active asset drag payload creation/clearing, incompatible scene/object drag clearing, and drag-source status text. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-reference/content pointer press ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
