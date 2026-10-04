---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events/click.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events/click/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target/prepare.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target/state.rs
  - zircon_editor/src/ui/retained_host/app/asset_surface_pointer_state.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/bridge.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/dispatch.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/layout.rs
  - zircon_editor/src/ui/retained_host/asset_pointer/reference/route.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events/click.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events/click/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/press.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target/prepare.rs
  - zircon_editor/src/ui/retained_host/app/asset_reference_pointer/target/state.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app asset-reference pointer events ownership scan
  - app asset-reference pointer event subowner ownership scan
  - app asset-reference pointer click dispatch subowner ownership scan
  - app asset-reference pointer press ownership scan
  - app asset-reference pointer target ownership scan
  - app asset-reference pointer target subowner ownership scan
  - scoped git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Asset Reference Pointer Dispatch

`app/asset_reference_pointer.rs` is the structural retained-host app entry for asset reference-list interactions. The child modules handle the references and used-by lists on both activity Assets and Asset Browser surfaces, callback-source focus, incompatible drag clearing before press dispatch, shared runtime Asset Surface click forwarding, returned host effects, hover/scroll pointer state writes, and asset drag payload creation when a reference row starts a drag.

The module is the host action boundary for reference-list interactions. It should not own generic asset surface layout synchronization or pointer bridge internals; those stay in `app/pointer_layout/asset_surfaces.rs` and `zircon_editor/src/ui/retained_host/asset_pointer/reference/*`.

## Event Dispatch

`app/asset_reference_pointer/events.rs` owns the retained-host press/release event gate for reference and used-by lists. It clears active asset drag payloads on primary-button release and forwards primary-button press to the press child.

`app/asset_reference_pointer/events/click.rs` owns click target preparation and prepared reference-list synchronization before click dispatch. `app/asset_reference_pointer/events/click/dispatch.rs` owns click forwarding through `callback_dispatch::dispatch_shared_asset_reference_pointer_click(...)`, returned host effects, and clicked pointer-state writeback. `app/asset_reference_pointer/events/motion.rs` owns move/scroll prepared bridge dispatch and hover/scroll state writeback.

Keeping event dispatch in a child module leaves the root as a structural owner and separates normal pointer event flow from drag payload construction and target preparation. The event subowner split keeps click callback dispatch distinct from hover/scroll motion updates.

## Press Drag Source

`app/asset_reference_pointer/press.rs` owns reference-list press handling that can create an asset drag payload. It clears incompatible scene/object drag payloads, prepares and dispatches the reference-list press target, writes pointer state, derives a reference/used-by drag payload from the active snapshot, clears asset drag payload state for non-item routes, and reports the drag source summary to the status line.

Keeping press drag-source construction in a child module separates drag source setup from click forwarding, move hover updates, scroll updates, and target preparation.

## Target Preparation

`app/asset_reference_pointer/target.rs` is the structural reference-list target entry. It owns `PreparedAssetReferencePointerTarget` and declares child owners for preparation, prepared bridge sync/dispatch, and pointer-state writeback.

`target/prepare.rs` resolves the correct `AssetWorkspaceSnapshot` from the surface mode, resolves callback-reported or cached list size through the retained-host callback surface sizing helpers, and owns the drag-clear-on-target-error policy. `target/dispatch.rs` synchronizes the `AssetReferenceListPointerBridge` with a fresh `AssetReferenceListPointerLayout`, dispatches prepared bridge events, and reports target errors. `target/state.rs` writes returned `AssetListPointerState` back into the selected reference/used-by list and mirrors the state back into the UI.

The child module centralizes the error policy that was duplicated across press, click, move, and scroll paths. Press preparation clears the active asset drag payload on target errors; click/move/scroll paths preserve the drag payload unless their dispatch result explicitly changes it.

## Boundary Rules

- Keep user-facing retained-host press/release gates in `app/asset_reference_pointer/events.rs`, click target preparation in `app/asset_reference_pointer/events/click.rs`, click bridge forwarding/effects in `app/asset_reference_pointer/events/click/dispatch.rs`, and hover/scroll motion updates in `app/asset_reference_pointer/events/motion.rs`.
- Keep reference/used-by press dispatch, drag payload creation, non-item drag clearing, incompatible scene/object drag clearing, and drag-source status text in `app/asset_reference_pointer/press.rs`.
- Keep `PreparedAssetReferencePointerTarget` and target child declarations in `app/asset_reference_pointer/target.rs`.
- Keep reference-list target preparation and drag-clear-on-target-error policy in `app/asset_reference_pointer/target/prepare.rs`.
- Keep prepared reference-list bridge synchronization and bridge dispatch wrapping in `app/asset_reference_pointer/target/dispatch.rs`.
- Keep reference-list pointer-state writeback in `app/asset_reference_pointer/target/state.rs`.
- Keep asset surface state declarations and reference-list accessors in `app/asset_surface_pointer_state.rs`.
- Keep pure reference pointer bridge, layout, route, target, and dispatch DTO behavior under `zircon_editor/src/ui/retained_host/asset_pointer/reference/`.

## Validation Notes

The 2026-06-19 events split reduced `asset_reference_pointer.rs` from 187 lines to 3 lines. `asset_reference_pointer/events.rs` originally owned reference/used-by press/release gates, click forwarding, move hover updates, scroll updates, and event effect/state writeback. The later event subowner split reduced `events.rs` to a 25-line press/release gate and moved click forwarding plus effect writeback to `events/click.rs` and move/scroll state writeback to `events/motion.rs`. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset content/reference pointer events ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only before the current unrelated runtime compile blocker. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 target split reduced `asset_reference_pointer.rs` from 336 lines to 235 lines. `asset_reference_pointer/target.rs` is 124 lines and owns the shared target preparation, bridge sync, prepared dispatch, and state writeback helpers.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-reference pointer target ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-18 press-drag split reduced `asset_reference_pointer.rs` from 245 lines to 196 lines. `asset_reference_pointer/press.rs` is 65 lines and owns reference/used-by press dispatch, active asset drag payload creation/clearing, incompatible scene/object drag clearing, and drag-source status text. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-reference/content pointer press ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 target subowner split reduced `asset_reference_pointer/target.rs` from 124 lines to an 11-line structural entry. `target/prepare.rs` is 51 lines and owns target snapshot/list-size preparation plus drag-clear-on-error behavior. `target/dispatch.rs` is 68 lines and owns prepared list synchronization plus bridge dispatch wrapping. `target/state.rs` is 18 lines and owns pointer-state writeback and UI mirror refresh.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-reference pointer target subowner ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. A fresh `cargo check` was not rerun for this slice because the immediately preceding focused editor check failed before reaching editor code on current `zircon_runtime` duplicate method definitions in `scene/dynamic_scene/session/path_capture.rs`; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The follow-up 2026-06-19 owner-split batch compile validation reran `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 click dispatch subowner split reduced `asset_reference_pointer/events/click.rs` from 93 lines to a 37-line click-preparation owner. `asset_reference_pointer/events/click/dispatch.rs` is 73 lines and owns asset-surface bridge availability checks, activity/browser references/used-by bridge selection, shared click dispatch, returned pointer-state writeback, returned host effects, and click-target diagnostics.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-reference pointer click dispatch subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.
