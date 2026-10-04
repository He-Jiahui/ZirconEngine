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
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/identity.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/input_owner.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/hierarchy_pointer_route.rs
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
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/identity.rs
  - zircon_editor/src/ui/retained_host/app/hierarchy_pointer/input_owner.rs
  - zircon_editor/src/ui/retained_host/host_contract/window/hierarchy_pointer_route.rs
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
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/identity.rs` owns the private press-time world, document activation, history context, and callback-window identity for hierarchy reparent and scene-instance payload consumption.
- `zircon_editor/src/ui/retained_host/app/hierarchy_pointer/input_owner.rs` owns the live native pointer, source-window, button, and terminal identity for hierarchy reparent gestures.
- `zircon_editor/src/ui/retained_host/host_contract/window/hierarchy_pointer_route.rs` supplies a weak host source handle and checks whether a native point still routes to the hierarchy pane.
- `zircon_editor/src/ui/retained_host/app/pointer_layout.rs` still owns persistent hierarchy pointer state projection back into the pane surface host.

## Behavior Model

Hierarchy pointer input first prepares the target surface from committed Workbench layout. The target owner resolves callback width and height, falls back through the existing callback surface-size policy, snapshots scene hierarchy entries from the runtime editor snapshot, and syncs the hierarchy pointer bridge before dispatch.

Pointer down on the primary button starts a scene drag probe. It clears incompatible asset/object drag payloads, routes through the hierarchy pointer bridge, writes the latest pointer state to UI, and builds a `SceneInstance` drag payload when the hovered route is a scene node. A different pointer cannot replace or activate an active hierarchy gesture. Native Move records pointer ownership before pane callbacks while the computed tooltip target is delivered afterward; a foreign Move can still update hover. Touch-like and untranslated Move cannot advance a mouse-owned hierarchy gesture even though native pane routing still runs. The owner button release can submit a reparent only while the source window and pointer still match; a second release cannot submit again. If native chrome capture consumes that Up before the hierarchy callback, post-dispatch cleanup ends the owner gesture without Reparent. Native input observed outside the hierarchy pane or in another window cancels the owner gesture before a late hierarchy callback can submit it. Pointer cancel, secondary press, Escape, source-window focus loss, and window close share the same terminal cleanup.

The press records the active World domain and gateway generation, document and activation revision, history context, and source window. Reparent release and SceneInstance reference drop recheck that identity before consuming node IDs or the payload. A stale press clears its gesture and payload; hierarchy refresh also retires a stale press before projecting a replacement World. Refresh checks world and document authority without treating the callback-free refresh as another source window. This private check does not scan scene nodes or change the shared drag payload format.

A stale SceneInstance drop stops the Asset, Instance, or Object reference-field action before its demonstration fallback runs. When there is no active drag payload, the existing static demonstration fallback still runs.

Click dispatch uses `callback_dispatch::dispatch_shared_hierarchy_pointer_click(...)` so selection and hierarchy actions still flow through the shared retained-host effect pipeline. Move and scroll use the hierarchy pointer bridge directly, updating hover and scroll state without dispatching unrelated Workbench actions.

## Design and Rationale

The three child files change for different reasons:

- `target.rs` changes when Workbench region geometry, callback surface fallback, or hierarchy layout projection changes.
- `events/drag.rs` changes when scene drag probing semantics change.
- `events/click.rs` changes when shared hierarchy click dispatch behavior changes.
- `events/motion.rs` and `events/scroll.rs` change when hover or scroll bridge behavior changes.
- `drag_source.rs` changes when scene-node drag metadata or drag payload URI policy changes.
- `input_owner.rs` changes when native window, pointer, or terminal ownership changes.

Keeping those boundaries separate prevents future hierarchy panel interaction work from rebuilding a mixed file around unrelated drag metadata and surface preparation details.

## Edge Cases and Constraints

- An owner primary-button release clears the active scene drag payload even when no target is prepared. A release from another pointer leaves its gesture intact.
- Drag start clears active asset and object drag payloads before probing the hierarchy route.
- Target preparation can focus the callback source window for press, click, and scroll callbacks, while hover move keeps focus unchanged.
- Scene drag payloads are emitted only for `HierarchyPointerRoute::Node` routes that still exist in the current scene entry snapshot.

## Test Coverage

The original 2026-06-19 ownership split recorded formatting, ownership scans, scoped diff checks, and a Cargo check for that earlier source snapshot. Those results do not validate the current Editor1022 identity changes. This R slice has scoped Rust formatting, diff, and exact-preservation checks; grouped managed Cargo tests and native product acceptance remain pending.

Editor1025 adds a two-`UiHostWindow` native dispatch fixture and tests outside-pane and cross-window release, foreign-pointer isolation, source-window focus loss, Escape, cancel, secondary press, close, and late duplicate release. The host fixture exercises production routing and callback wiring but does not prove operating-system capture acknowledgement or loss delivery; those product checks remain open.

Native tab and resize capture still lack physical pointer identity. A foreign tab press can consume the hierarchy owner's Up; the post-dispatch fallback cancels that hierarchy gesture once, while independent success of concurrent gestures remains pending a separate native capture change.

The 2026-06-19 events subowner split reduced `hierarchy_pointer/events.rs` from 109 lines to a 4-line structural entry. At that snapshot, `events/drag.rs` was 47 lines, `events/click.rs` was 26 lines, `events/motion.rs` was 22 lines, and `events/scroll.rs` was 23 lines. Their ownership remains split across drag, click, hover, and scroll behavior.

For that 2026-06-19 snapshot, validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app hierarchy-pointer events subowner ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`; the recorded warning counts were `zircon_runtime` 141 and `zircon_editor` 65. Those historical receipts do not cover the current R source.

## Plan Sources

This module belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, where retained-host Workbench shell behavior is being converged into runtime UI backed surfaces with narrow app owners.

## Open Issues or Follow-up

- Keep scene-node drag metadata in `drag_source.rs`, target/layout sync in `target.rs`, and concrete callback event dispatch in the `events/` child files.
- The milestone testing stage still needs the declared `zircon_editor` test commands after the remaining feature-first implementation slices finish.
