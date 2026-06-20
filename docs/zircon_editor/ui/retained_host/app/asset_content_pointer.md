---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/click.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/prepare.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/state.rs
  - zircon_editor/src/ui/retained_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/dispatch.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/content/route.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/click.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/prepare.rs
  - zircon_editor/src/ui/retained_host/app/asset_content_pointer/target/state.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app asset-content pointer events ownership scan
  - app asset-content pointer event subowner ownership scan
  - app asset-content pointer press ownership scan
  - app asset-content pointer target ownership scan
  - app asset-content pointer target subowner ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Asset Content Pointer Dispatch

`app/asset_content_pointer.rs` is the structural retained-host app entry for asset content-list pointer interactions on both activity Assets and Asset Browser surfaces. The child modules handle press, click, move, and scroll entry points, callback-source focus, incompatible drag clearing before a drag press, shared runtime Asset Surface click forwarding, returned host effects, hover/scroll pointer state writes, and asset drag payload creation when an asset content row starts a drag.

The module is the host action boundary for asset content-list interactions. It should not own generic asset surface layout synchronization or pointer bridge internals; those stay in `app/pointer_layout/asset_surfaces.rs` and `zircon_editor/src/ui/retained_host/asset_pointer/content/*`.

## Event Dispatch

`app/asset_content_pointer/events.rs` owns the retained-host press/release event gate for content lists. It clears active asset drag payloads on primary-button release and forwards primary-button press to the press child.

`app/asset_content_pointer/events/click.rs` owns click forwarding through `callback_dispatch::dispatch_shared_asset_content_pointer_click(...)`, returned host effects, and clicked pointer-state writeback. `app/asset_content_pointer/events/motion.rs` owns move/scroll prepared bridge dispatch and hover/scroll state writeback.

Keeping event dispatch in a child module leaves the root as a structural owner and separates normal pointer event flow from drag payload construction and target preparation. The event subowner split keeps click callback dispatch distinct from hover/scroll motion updates.

## Press Drag Source

`app/asset_content_pointer/press.rs` owns content-list press handling that can create an asset drag payload. It clears incompatible scene/object drag payloads, prepares and dispatches the content-list press target, writes pointer state, derives a drag payload from the active snapshot, clears asset drag payload state for non-item routes, and reports the drag source summary to the status line.

Keeping press drag-source construction in a child module separates drag source setup from click forwarding, move hover updates, scroll updates, and target preparation.

## Target Preparation

`app/asset_content_pointer/target.rs` is the structural content-list target entry. It owns `PreparedAssetContentPointerTarget` and declares child owners for preparation, prepared bridge sync/dispatch, and pointer-state writeback.

`target/prepare.rs` resolves the correct `AssetWorkspaceSnapshot` from the surface mode, resolves callback-reported or cached content-list size through the retained-host callback surface sizing helpers, and owns the drag-clear-on-target-error policy. `target/dispatch.rs` synchronizes the `AssetContentListPointerBridge` with a fresh `AssetContentListPointerLayout`, dispatches prepared bridge events, and reports target errors. `target/state.rs` writes returned `AssetListPointerState` back into the selected asset surface content list and mirrors the state back into the UI.

The child module centralizes the error policy that was duplicated across press, click, move, and scroll paths. Press preparation clears the active asset drag payload on target errors; click/move/scroll paths preserve the drag payload unless their dispatch result explicitly changes it.

## Boundary Rules

- Keep user-facing retained-host press/release gates in `app/asset_content_pointer/events.rs`, click forwarding in `app/asset_content_pointer/events/click.rs`, and hover/scroll motion updates in `app/asset_content_pointer/events/motion.rs`.
- Keep content-list press dispatch, drag payload creation, non-item drag clearing, incompatible scene/object drag clearing, and drag-source status text in `app/asset_content_pointer/press.rs`.
- Keep `PreparedAssetContentPointerTarget` and target child declarations in `app/asset_content_pointer/target.rs`.
- Keep content-list target preparation and drag-clear-on-target-error policy in `app/asset_content_pointer/target/prepare.rs`.
- Keep prepared content-list bridge synchronization and bridge dispatch wrapping in `app/asset_content_pointer/target/dispatch.rs`.
- Keep content-list pointer-state writeback in `app/asset_content_pointer/target/state.rs`.
- Keep asset surface state declarations and content-list fields in `app/asset_surface_pointer_state.rs`.
- Keep pure content pointer bridge, layout, route, target, and dispatch DTO behavior under `zircon_editor/src/ui/retained_host/asset_pointer/content/`.

## Validation Notes

The 2026-06-19 events split reduced `asset_content_pointer.rs` from 140 lines to 3 lines. `asset_content_pointer/events.rs` originally owned content-list press/release gates, click forwarding, move hover updates, scroll updates, and event effect/state writeback. The later event subowner split reduced `events.rs` to a 24-line press/release gate and moved click forwarding plus effect writeback to `events/click.rs` and move/scroll state writeback to `events/motion.rs`. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset content/reference pointer events ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only before the current unrelated runtime compile blocker. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 target split reduced `asset_content_pointer.rs` from 269 lines to 181 lines. `asset_content_pointer/target.rs` is 98 lines and owns the shared target preparation, bridge sync, prepared dispatch, and state writeback helpers.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-content pointer target ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 press-drag split reduced `asset_content_pointer.rs` from 191 lines to 149 lines. `asset_content_pointer/press.rs` is 57 lines and owns content-list press dispatch, active asset drag payload creation/clearing, incompatible scene/object drag clearing, and drag-source status text. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-reference/content pointer press ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 target subowner split reduced `asset_content_pointer/target.rs` from 98 lines to an 11-line structural entry. `target/prepare.rs` is 47 lines and owns target snapshot/content-size preparation plus drag-clear-on-error behavior. `target/dispatch.rs` is 48 lines and owns prepared content-list synchronization plus bridge dispatch wrapping. `target/state.rs` is 15 lines and owns pointer-state writeback and UI mirror refresh.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-content pointer target subowner ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. A fresh `cargo check` was not rerun for this slice because the current focused editor check is blocked before editor code by `zircon_runtime` duplicate method definitions in `scene/dynamic_scene/session/path_capture.rs`; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The follow-up 2026-06-19 owner-split batch compile validation reran `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
