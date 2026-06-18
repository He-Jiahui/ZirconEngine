---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/dispatch.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/route.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app asset-reference pointer events ownership scan
  - app asset-reference pointer press ownership scan
  - app asset-reference pointer target ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Asset Reference Pointer Dispatch

`app/asset_reference_pointer.rs` is the structural retained-host app entry for asset reference-list interactions. The child modules handle the references and used-by lists on both activity Assets and Asset Browser surfaces, callback-source focus, incompatible drag clearing before press dispatch, shared runtime Asset Surface click forwarding, returned host effects, hover/scroll pointer state writes, and asset drag payload creation when a reference row starts a drag.

The module is the host action boundary for reference-list interactions. It should not own generic asset surface layout synchronization or pointer bridge internals; those stay in `app/pointer_layout/asset_surfaces.rs` and `zircon_editor/src/ui/retained_host/asset_pointer/reference/*`.

## Event Dispatch

`app/asset_reference_pointer/events.rs` owns the retained-host event entry points for reference and used-by lists. It normalizes press/release gates, forwards primary-button press to the press child, prepares reference-list targets for click/move/scroll, dispatches clicks through `callback_dispatch::dispatch_shared_asset_reference_pointer_click(...)`, applies returned effects, and writes hover/scroll state from prepared bridge dispatch.

Keeping event dispatch in a child module leaves the root as a structural owner and separates normal pointer event flow from drag payload construction and target preparation.

## Press Drag Source

`app/asset_reference_pointer/press.rs` owns reference-list press handling that can create an asset drag payload. It clears incompatible scene/object drag payloads, prepares and dispatches the reference-list press target, writes pointer state, derives a reference/used-by drag payload from the active snapshot, clears asset drag payload state for non-item routes, and reports the drag source summary to the status line.

Keeping press drag-source construction in a child module separates drag source setup from click forwarding, move hover updates, scroll updates, and target preparation.

## Target Preparation

`app/asset_reference_pointer/target.rs` owns reference-list target preparation for the host action boundary. It resolves the correct `AssetWorkspaceSnapshot` from the surface mode, resolves callback-reported or cached list size through the retained-host callback surface sizing helpers, synchronizes the `AssetReferenceListPointerBridge` with a fresh `AssetReferenceListPointerLayout`, dispatches prepared bridge events, and writes returned `AssetListPointerState` back into the selected reference/used-by list.

The child module centralizes the error policy that was duplicated across press, click, move, and scroll paths. Press preparation clears the active asset drag payload on target errors; click/move/scroll paths preserve the drag payload unless their dispatch result explicitly changes it.

## Boundary Rules

- Keep user-facing retained-host entry points (`asset_reference_pointer_event`, `asset_reference_pointer_clicked`, `asset_reference_pointer_moved`, and `asset_reference_pointer_scrolled`) in `app/asset_reference_pointer/events.rs`.
- Keep reference/used-by press dispatch, drag payload creation, non-item drag clearing, incompatible scene/object drag clearing, and drag-source status text in `app/asset_reference_pointer/press.rs`.
- Keep reference-list target preparation, layout synchronization, bridge dispatch wrapping, drag-clear-on-target-error policy, and pointer-state writeback in `app/asset_reference_pointer/target.rs`.
- Keep asset surface state declarations and reference-list accessors in `app/asset_surface_pointer_state.rs`.
- Keep pure reference pointer bridge, layout, route, target, and dispatch DTO behavior under `zircon_editor/src/ui/retained_host/asset_pointer/reference/`.

## Validation Notes

The 2026-06-19 events split reduced `asset_reference_pointer.rs` from 187 lines to 3 lines. `asset_reference_pointer/events.rs` is 185 lines and owns reference/used-by press/release gates, click forwarding, move hover updates, scroll updates, and event effect/state writeback. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset content/reference pointer events ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 target split reduced `asset_reference_pointer.rs` from 336 lines to 235 lines. `asset_reference_pointer/target.rs` is 124 lines and owns the shared target preparation, bridge sync, prepared dispatch, and state writeback helpers.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-reference pointer target ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 press-drag split reduced `asset_reference_pointer.rs` from 245 lines to 196 lines. `asset_reference_pointer/press.rs` is 65 lines and owns reference/used-by press dispatch, active asset drag payload creation/clearing, incompatible scene/object drag clearing, and drag-source status text. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-reference/content pointer press ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
