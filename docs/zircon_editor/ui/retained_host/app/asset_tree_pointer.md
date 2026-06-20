---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/click.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/motion.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/scroll.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs
  - zircon_editor/src/ui/retained_host/app/assets.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/click.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/motion.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/scroll.rs
  - zircon_editor/src/ui/retained_host/app/asset_tree_pointer/target.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app asset-tree pointer target/dispatch ownership scan
  - app asset-tree pointer dispatch subowner ownership scan
  - git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Asset Tree Pointer

## Purpose

The retained-host asset tree pointer boundary owns pointer callbacks for the Activity and Asset Browser folder trees. It keeps the app-facing `RetainedEditorHost` methods stable while separating tree target preparation from click/move/scroll dispatch.

This split supports the 08 M3.S2 retained-host cleanup: `app/asset_tree_pointer.rs` is now a structural entry, and tree target preparation can be reused by all asset tree pointer actions without repeating snapshot, size fallback, and bridge sync logic.

## Related Files

- `zircon_editor/src/ui/retained_host/app/asset_tree_pointer.rs` declares the asset tree pointer child modules only.
- `zircon_editor/src/ui/retained_host/app/asset_tree_pointer/target.rs` owns asset tree target preparation: committed pointer layout reuse, callback-source focus, workspace snapshot lookup, callback surface size fallback, tree size writeback, and tree bridge layout sync.
- `zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch.rs` declares the asset tree pointer dispatch child modules only.
- `zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/click.rs` owns click dispatch for Activity and Browser asset tree surfaces, including runtime asset surface bridge forwarding and dispatch effects.
- `zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/motion.rs` owns hover/move dispatch against the already-synced tree bridge.
- `zircon_editor/src/ui/retained_host/app/asset_tree_pointer/dispatch/scroll.rs` owns scroll dispatch against the already-synced tree bridge.
- `zircon_editor/src/ui/retained_host/app/pointer_layout/asset_surfaces.rs` writes pointer hover/scroll state back into the visible Workbench surfaces after dispatch.

## Behavior Model

Every asset tree pointer path first calls `prepare_asset_tree_pointer_target(...)`. That helper keeps pointer callbacks on the last committed layout, focuses the callback source window, resolves the asset workspace snapshot for the requested surface mode, resolves the effective tree surface size from callback dimensions or stored layout fallback, writes the size into the surface state, and syncs the folder tree pointer bridge with the current snapshot and pointer state.

Click dispatch then ensures the runtime asset surface bridge is available and routes the click through `dispatch_shared_asset_tree_pointer_click(...)` for either the Activity or Browser tree bridge. Successful dispatch writes the returned pointer state back, updates visible UI hover/scroll state, and applies any runtime dispatch effects.

Move and scroll dispatch use the already-synced tree bridge directly. Successful dispatch writes the returned pointer state back and applies the asset pointer state to the UI.

## Design and Rationale

The old file repeated target preparation across click, move, and scroll. Keeping that logic in `target.rs` makes the invariant explicit: all tree pointer actions must share the same snapshot, size fallback, committed-layout, and callback-focus preparation before dispatch.

The dispatch subtree remains separate because click dispatch crosses into the runtime asset surface bridge and can return UI effects, while move and scroll only update local pointer state. The structural `dispatch.rs` entry keeps those route owners grouped without duplicating target setup.

## Edge Cases and Constraints

- Unknown surface modes write a status-line diagnostic and abort before bridge mutation.
- A missing asset surface bridge aborts click dispatch before runtime UI forwarding.
- Target preparation must run before move/scroll so hover and scroll operate on the current folder tree layout.
- Surface modes remain the existing string contract: `activity` and `browser`.

## Test Coverage

Implementation-slice validation covers formatting, ownership scans, scoped diff checks, and the current practical Cargo check status. `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` is currently blocked before editor code by unrelated active-worktree `zircon_runtime` post-process render errors. Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

The 2026-06-19 dispatch subowner split reduced `asset_tree_pointer/dispatch.rs` from 112 lines to a 3-line structural entry. `dispatch/click.rs` is 55 lines and owns runtime bridge click forwarding plus dispatch effects, `dispatch/motion.rs` is 32 lines and owns tree hover movement, and `dispatch/scroll.rs` is 33 lines and owns tree scroll movement.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app asset-tree pointer dispatch subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

## Plan Sources

This module belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, where retained-host Workbench shell behavior is being converged into runtime UI backed surfaces with narrow app owners.

## Open Issues or Follow-up

- Keep future shared asset tree target preparation in `target.rs`; route-specific click/move/scroll behavior belongs in the `dispatch/` child files.
