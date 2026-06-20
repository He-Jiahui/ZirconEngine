---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/click.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/scroll.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/drag_source.rs
  - zircon_editor/src/ui/retained_host/app/pointer_layout.rs
  - zircon_editor/src/ui/retained_host/hierarchy_pointer/mod.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/click.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/motion.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/scroll.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/target.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/drag_source.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app hierarchy-pointer target/events/drag-source ownership scan
  - app hierarchy-pointer events subowner ownership scan
  - git diff --check
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Hierarchy Pointer

## Purpose

The retained-host hierarchy pointer boundary owns native/template callbacks for the Scene Hierarchy panel. It keeps the public `RetainedEditorHost` callback methods stable while splitting target preparation, pointer event dispatch, and scene-node drag payload construction into separate app-local owners.

This supports the 08 M3.S2 retained-host cleanup by making `app/hierarchy_pointer.rs` a structural module entry instead of a mixed event and payload helper file.

## Related Files

- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer.rs` declares the hierarchy pointer child modules.
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/target.rs` owns committed pointer-layout reuse, callback surface-size resolution, hierarchy bridge layout sync, and optional callback-source focus.
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events.rs` declares the hierarchy pointer event child modules only.
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/drag.rs` owns primary-button drag probing and scene drag payload activation.
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/click.rs` owns shared hierarchy click dispatch and dispatch-effect application.
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/motion.rs` owns hover movement against the hierarchy pointer bridge.
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/events/scroll.rs` owns hierarchy pointer scroll dispatch.
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/drag_source.rs` owns scene-node drag payload construction from hierarchy pointer routes.
- `zircon_editor/src/ui/retained_host/app/pointer_layout.rs` still owns persistent hierarchy pointer state projection back into the pane surface host.

## Behavior Model

Hierarchy pointer input first prepares the target surface from committed Workbench layout. The target owner resolves callback width and height, falls back through the existing callback surface-size policy, snapshots scene hierarchy entries from the runtime editor snapshot, and syncs the hierarchy pointer bridge before dispatch.

Pointer down on the primary button starts a scene drag probe. It clears incompatible asset/object drag payloads, routes through the hierarchy pointer bridge, writes the latest pointer state to UI, and builds a `SceneInstance` drag payload when the hovered route is a scene node. Pointer release clears the active scene drag payload. Other non-primary or non-down event shapes are ignored, matching the previous callback behavior.

Click dispatch uses `callback_dispatch::dispatch_shared_hierarchy_pointer_click(...)` so selection and hierarchy actions still flow through the shared retained-host effect pipeline. Move and scroll use the hierarchy pointer bridge directly, updating hover and scroll state without dispatching unrelated Workbench actions.

## Design and Rationale

The three child files change for different reasons:

- `target.rs` changes when Workbench region geometry, callback surface fallback, or hierarchy layout projection changes.
- `events/drag.rs` changes when scene drag probing semantics change.
- `events/click.rs` changes when shared hierarchy click dispatch behavior changes.
- `events/motion.rs` and `events/scroll.rs` change when hover or scroll bridge behavior changes.
- `drag_source.rs` changes when scene-node drag metadata or drag payload URI policy changes.

Keeping those boundaries separate prevents future hierarchy panel interaction work from rebuilding a mixed file around unrelated drag metadata and surface preparation details.

## Edge Cases and Constraints

- Primary-button release clears the active scene drag payload even when no target is prepared.
- Drag start clears active asset and object drag payloads before probing the hierarchy route.
- Target preparation can focus the callback source window for press, click, and scroll callbacks, while hover move keeps focus unchanged.
- Scene drag payloads are emitted only for `HierarchyPointerRoute::Node` routes that still exist in the current scene entry snapshot.

## Test Coverage

Implementation-slice validation covers formatting, ownership scans, scoped diff checks, and current practical Cargo check status. `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, app hierarchy-pointer ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` pass in the current worktree. Cargo still emits existing warning noise from active runtime/editor work. Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

The 2026-06-19 events subowner split reduced `hierarchy_pointer/events.rs` from 109 lines to a 4-line structural entry. `events/drag.rs` is 47 lines and owns scene drag probing, `events/click.rs` is 26 lines and owns shared click dispatch, `events/motion.rs` is 22 lines and owns hover movement, and `events/scroll.rs` is 23 lines and owns scroll movement.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app hierarchy-pointer events subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

## Plan Sources

This module belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, where retained-host Workbench shell behavior is being converged into runtime UI backed surfaces with narrow app owners.

## Open Issues or Follow-up

- Keep scene-node drag metadata in `drag_source.rs`, target/layout sync in `target.rs`, and concrete callback event dispatch in the `events/` child files.
- The milestone testing stage still needs the declared `zircon_editor` test commands after the remaining feature-first implementation slices finish.
