---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/viewport/pointer_event.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/hit_controls.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/viewport/pointer_event.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/hit_controls.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app viewport pointer/toolbar ownership scan
  - app viewport toolbar projection hit-control ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Viewport App Boundary

## Purpose

The retained-host viewport app boundary is the native callback side of the editor viewport. It keeps the public callback methods on `RetainedEditorHost` stable while separating two concerns that used to share one file:

- viewport content pointer events, including native integer event decoding, world-space UI routing, focus transfer, and runtime pointer dispatch;
- viewport toolbar pointer events, including toolbar surface sizing, bridge layout refresh, and shared toolbar control dispatch.

This split supports the 08 M3.S2 Workbench shell migration goal: app-level callbacks remain thin entry points, while pointer routing and toolbar sizing are owned by named files under the viewport subtree.

## Related Files

- `zircon_editor/src/ui/retained_host/app/viewport.rs` is now structural. It declares the viewport app children only.
- `zircon_editor/src/ui/retained_host/app/viewport/pointer_event.rs` owns native viewport pointer decoding and world-space UI pointer routing.
- `zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer.rs` owns viewport toolbar click dispatch and toolbar surface-size resolution.
- `zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs` still owns presentation-time toolbar surface-frame projection into pane data.
- `zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/hit_controls.rs` owns projection control id to stable viewport toolbar hit-control id mapping.
- `zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs` wires native/template callback registration to the retained-host viewport methods.

## Behavior Model

Viewport pointer input enters through `RetainedEditorHost::viewport_pointer_event(...)`. The method first uses the committed pointer layout instead of rebuilding UI inside the native event callback. It then maps host integer event kinds and buttons into `UiPointerEvent` values. Unknown event kinds or button ids produce a status-line error and stop before runtime dispatch.

Non-move viewport pointer events focus the current callback source window before dispatch. This keeps floating-window viewport input associated with the window that emitted the callback. The pointer event then gets a world-space UI routing opportunity through the viewport controller. If a world-space UI control accepts the event, the host updates the status line for down, scroll, up, or cancel and does not forward the same event into the regular viewport pointer bridge.

Toolbar input enters through `RetainedEditorHost::viewport_toolbar_pointer_clicked(...)`. The method computes the toolbar surface size for the requested surface key, recomputes the toolbar template bridge layout, syncs the toolbar pointer bridge with that surface size, then dispatches the click through the shared viewport toolbar callback path. Dispatch effects are applied through the same retained-host effect pipeline used by other Workbench callbacks.

## Design and Rationale

The root `viewport.rs` file is deliberately small because the viewport subtree has more than one behavior family. Pointer input and toolbar sizing both belong to the viewport app boundary, but they change for different reasons:

- pointer input changes when native event semantics, world-space UI routing, or callback focus ownership changes;
- toolbar input changes when Workbench pane hosts, drawer/floating-window sizing, or toolbar bridge dispatch changes.

Keeping them in separate files makes the next extension point obvious. Additional viewport-specific app concerns can land beside these children instead of accumulating in a mixed root file.

The methods remain `pub(in crate::ui::retained_host::app)` so existing callback wiring and app-local tests can call the same retained-host entry points without exposing them beyond the retained-host app boundary.

## Control Flow

Viewport pointer flow:

1. Native/template callback invokes `viewport_pointer_event(...)`.
2. The host publishes invalidation diagnostics and reuses committed pointer layout state.
3. Host integers are mapped into `UiPointerEventKind`, `UiPointerButton`, point, and scroll delta.
4. Non-move events focus the callback source window.
5. World-space UI gets first chance to consume the event.
6. Unconsumed events route through `callback_dispatch::dispatch_viewport_pointer_event(...)`.
7. Runtime dispatch effects update the retained-host state, or dispatch errors update the status line.

Viewport toolbar flow:

1. Toolbar callback invokes `viewport_toolbar_pointer_clicked(...)`.
2. The host resolves the surface size for document, drawer, exclusive-page, or floating-window hosts.
3. Toolbar layout and pointer bridge state are synchronized for that surface.
4. Shared toolbar dispatch applies control-specific effects through the retained-host effect pipeline.

Viewport toolbar projection flow:

1. Lifecycle recompute calls `attach_viewport_toolbar_surface_frames_to_ui(...)` after the Workbench presentation has been assembled.
2. The projection owner derives document, drawer, bottom, and floating-window toolbar sizes from the current host presentation.
3. Scene/Game panes with visible toolbars recompute the toolbar template bridge and receive a toolbar surface frame.
4. `hit_controls.rs` maps template projection controls such as `SetTool`, `SetProjectionMode`, and `AlignView` to the stable viewport toolbar action ids used by pointer dispatch.

## Edge Cases and Constraints

- Floating-window toolbar sizing resolves through native floating-window content frames before falling back to docked geometry.
- Drawer toolbar sizing uses drawer content frames first and drawer shell frames second; zero-width frames collapse to a minimum one-pixel toolbar surface.
- Unknown viewport pointer event ids and button ids are surfaced as status-line diagnostics rather than silently ignored.
- World-space UI pointer consumption is exclusive. Once a world-space UI control accepts a viewport pointer event, the regular viewport pointer bridge does not receive the same event.
- The split does not change callback names, Slint/global binding names, callback registration paths, or stable viewport toolbar hit-control ids.

## Test Coverage

Implementation-slice validation currently covers formatting, ownership scans, scoped diff checks, and the latest practical `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` status. Existing retained-host app tests still reference `viewport_pointer_event(...)`, `viewport_toolbar_pointer_clicked(...)`, `viewport_toolbar_surface_size(...)`, and viewport toolbar projection behavior; the full Cargo test matrix remains deferred to the milestone testing stage per the user's instruction.

The 2026-06-19 viewport toolbar projection hit-control split reduced `viewport_toolbar_projection.rs` from 207 lines to 151 lines. `viewport_toolbar_projection/hit_controls.rs` is 59 lines and owns projection control id to stable toolbar hit-control id mapping for tool, transform-space, projection-mode, align-view, display/grid/snap toggles, preview toggles, frame selection, and play-mode controls. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app viewport toolbar projection hit-control ownership scan, and scoped `git diff --check`, all of which passed except for the existing CRLF conversion warning on the plan file. Current `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` and the narrower `cargo check -p zircon_editor --lib --no-default-features --locked --jobs 1 --message-format short --color never` are blocked before editor code by unrelated `zircon_runtime` post-process render errors in the active worktree: multiple `execute_*` modules are missing `UVec2` imports, and `gpu/post_process.rs` supplies one extra argument to `execute_post_process`.

## Plan Sources

This module belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, where retained-host Workbench shell behavior is being converged into runtime UI backed surfaces with narrow app owners.

## Open Issues or Follow-up

- The milestone testing stage still needs the declared `zircon_editor` test commands after the remaining feature-first implementation slices finish.
- `viewport_toolbar_projection.rs` should stay focused on pane-data projection. Add future toolbar control id mappings to `viewport_toolbar_projection/hit_controls.rs`, not to the projection root.
