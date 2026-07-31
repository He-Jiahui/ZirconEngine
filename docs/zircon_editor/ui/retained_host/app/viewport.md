---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/viewport/pointer_event.rs
  - zircon_editor/src/ui/retained_host/app/viewport/pointer_event/mapping.rs
  - zircon_editor/src/ui/retained_host/app/viewport/pointer_event/world_space.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/click.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/hit_controls.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/docked.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/floating.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/pane_frame.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring/pane_surface/viewport.rs
  - zircon_editor/src/ui/retained_host/callback_dispatch/mod.rs
  - zircon_editor/src/ui/retained_host/viewport_toolbar_pointer/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/viewport.rs
  - zircon_editor/src/ui/retained_host/app/viewport/pointer_event.rs
  - zircon_editor/src/ui/retained_host/app/viewport/pointer_event/mapping.rs
  - zircon_editor/src/ui/retained_host/app/viewport/pointer_event/world_space.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/click.rs
  - zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/hit_controls.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/docked.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/floating.rs
  - zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/pane_frame.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app viewport pointer/toolbar ownership scan
  - app viewport pointer-event mapping/world-space ownership scan
  - app viewport toolbar pointer click/size ownership scan
  - app viewport toolbar projection hit-control ownership scan
  - app viewport toolbar projection surface-frame ownership scan
  - app viewport toolbar projection surface-frame subowner ownership scan
  - scoped git diff --check
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
- `zircon_editor/src/ui/retained_host/app/viewport/pointer_event.rs` owns the viewport pointer callback flow: committed-layout reuse, non-move focus transfer, world-space UI routing, and regular runtime pointer dispatch.
- `zircon_editor/src/ui/retained_host/app/viewport/pointer_event/mapping.rs` owns native integer viewport pointer kind/button decoding into `UiPointerEvent`.
- `zircon_editor/src/ui/retained_host/app/viewport/pointer_event/world_space.rs` owns status-line text for world-space UI pointer consumption.
- `zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer.rs` is the structural viewport toolbar pointer entry.
- `zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/click.rs` owns viewport toolbar click dispatch.
- `zircon_editor/src/ui/retained_host/app/viewport/toolbar_pointer/size.rs` owns toolbar surface-size resolution for document, drawer, exclusive-page, and floating-window hosts.
- `zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection.rs` is the structural entry for viewport toolbar projection.
- `zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/hit_controls.rs` owns projection control id to stable viewport toolbar hit-control id mapping.
- `zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames.rs` is the structural entry for presentation-time toolbar surface-frame projection into pane data.
- `zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/docked.rs` owns document, left, right, and bottom dock toolbar frame attachment.
- `zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/floating.rs` owns floating-window toolbar frame attachment and floating-window model writeback.
- `zircon_editor/src/ui/retained_host/app/viewport_toolbar_projection/surface_frames/pane_frame.rs` owns Scene/Game pane eligibility, toolbar size clamping, bridge layout recompute, stable hit-control id projection, and invalid toolbar-frame clearing.
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

The viewport pointer callback entry now keeps dispatch flow separate from small pure policies: `pointer_event/mapping.rs` changes when host integer event semantics change, while `pointer_event/world_space.rs` changes when world-space UI status copy changes. The toolbar pointer entry stays structural: `click.rs` changes when pointer bridge dispatch or effect application changes, while `size.rs` changes when Workbench host geometry or fallback sizing changes. This keeps input routing separate from decoding, status text, and projection geometry policy.

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
2. The structural projection owner loads the current host presentation, then delegates docked regions and floating windows to their child owners.
3. The docked child derives document, left, right, and bottom toolbar widths from content frames plus the optional componentized document toolbar width.
4. The floating child rebuilds the floating-window model rows after attaching toolbar frame data to eligible active panes.
5. `pane_frame.rs` accepts only visible Scene/Game toolbars, recomputes the template bridge, writes the projected toolbar surface frame, or clears stale toolbar frame data when eligibility/layout fails.
6. `hit_controls.rs` maps template projection controls such as `ActivateSceneMode`, `SetProjectionMode`, and `AlignView` to stable viewport toolbar action ids. Custom scene-mode ids remain intact across projection and pointer dispatch.

## Edge Cases and Constraints

- Floating-window toolbar sizing resolves through native floating-window content frames before falling back to docked geometry.
- Drawer toolbar sizing uses drawer content frames first and drawer shell frames second; zero-width frames collapse to a minimum one-pixel toolbar surface.
- Unknown viewport pointer event ids and button ids are surfaced as status-line diagnostics rather than silently ignored.
- World-space UI pointer consumption is exclusive. Once a world-space UI control accepts a viewport pointer event, the regular viewport pointer bridge does not receive the same event.
- The split does not change callback names, Slint/global binding names, callback registration paths, or stable viewport toolbar hit-control ids.

## Test Coverage

Implementation-slice validation currently covers formatting, ownership scans, scoped diff checks, and the latest practical `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` status. Existing retained-host app tests still reference `viewport_pointer_event(...)`, `viewport_toolbar_pointer_clicked(...)`, `viewport_toolbar_surface_size(...)`, and viewport toolbar projection behavior; the full Cargo test matrix remains deferred to the milestone testing stage per the user's instruction.

The 2026-06-19 viewport toolbar projection hit-control split reduced `viewport_toolbar_projection.rs` from 207 lines to 151 lines. `viewport_toolbar_projection/hit_controls.rs` is 59 lines and owns projection control id to stable toolbar hit-control id mapping for tool, transform-space, projection-mode, align-view, display/grid/snap toggles, preview toggles, frame selection, and play-mode controls. Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app viewport toolbar projection hit-control ownership scan, and scoped `git diff --check`, all of which passed except for the existing CRLF conversion warning on the plan file. Current `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never` and the narrower `cargo check -p zircon_editor --lib --no-default-features --locked --jobs 1 --message-format short --color never` are blocked before editor code by unrelated `zircon_runtime` post-process render errors in the active worktree: multiple `execute_*` modules are missing `UVec2` imports, and `gpu/post_process.rs` supplies one extra argument to `execute_post_process`.

The 2026-06-19 viewport toolbar projection surface-frame split reduced `viewport_toolbar_projection.rs` from 151 lines to 3 lines. `viewport_toolbar_projection/surface_frames.rs` is 150 lines and owns document/left/right/bottom/floating-window toolbar surface-frame attachment, toolbar size derivation, pane eligibility, layout recompute, and host presentation writeback. `viewport_toolbar_projection/hit_controls.rs` remains 59 lines and owns stable hit-control id mapping.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app viewport toolbar projection surface-frame ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 142 warnings, `zircon_editor` 63 warnings). The first cargo check exposed app-sibling visibility after the projection entry moved; `attach_viewport_toolbar_surface_frames_to_ui(...)` is kept app-internal with `pub(in crate::ui::retained_host::app)`. Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 viewport toolbar projection surface-frame subowner split reduced the previously documented `viewport_toolbar_projection/surface_frames.rs` owner from 150 lines to a 22-line structural entry. `surface_frames/docked.rs` is 79 lines and owns document/left/right/bottom dock frame attachment; `surface_frames/floating.rs` is 34 lines and owns floating-window row projection/writeback; `surface_frames/pane_frame.rs` is 38 lines and owns pane eligibility, toolbar size, bridge layout recompute, projected frame creation, and stale-frame clearing.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app viewport toolbar projection surface-frame subowner ownership scan, and scoped `git diff --check`, all of which passed except for existing CRLF conversion warnings in the dirty worktree. Focused `cargo check` was not rerun for this slice because independent `zircon_runtime` Cargo test processes were still active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 viewport toolbar pointer click/size split reduced `viewport/toolbar_pointer.rs` from 124 lines to a 2-line structural entry. `toolbar_pointer/click.rs` is 46 lines and owns toolbar pointer bridge sync plus shared toolbar click dispatch. `toolbar_pointer/size.rs` is 82 lines and owns floating-window, document, drawer, exclusive-page, and fallback toolbar surface-size resolution.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app viewport toolbar pointer click/size ownership scan, and scoped `git diff --check`; scoped diff check only reported the existing CRLF working-tree conversion warning. Focused `cargo check` was not rerun for this slice because an independent `zircon_runtime` Cargo test process was active; full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

The 2026-06-19 viewport pointer-event mapping/world-space split reduced `viewport/pointer_event.rs` from 97 lines to a 46-line callback-flow owner. `pointer_event/mapping.rs` is 47 lines and owns native integer event/button decoding. `pointer_event/world_space.rs` is 13 lines and owns world-space UI pointer status text.

Validation used `cargo fmt -p zircon_editor`, `cargo fmt -p zircon_editor --check`, an app viewport pointer-event mapping/world-space ownership scan, scoped `git diff --check`, and `cargo check -p zircon_editor --lib --locked --jobs 1 --target-dir D:\cargo-targets\zircon-editor-ui-owner-split-0619 --message-format short --color never`, which passed with existing warning noise only (`zircon_runtime` 141 warnings, `zircon_editor` 65 warnings). Full Cargo test matrix remains deferred to the milestone validation stage per the user's instruction.

## Plan Sources

This module belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, where retained-host Workbench shell behavior is being converged into runtime UI backed surfaces with narrow app owners.

## Open Issues or Follow-up

- The milestone testing stage still needs the declared `zircon_editor` test commands after the remaining feature-first implementation slices finish.
- `viewport/pointer_event.rs` should stay focused on callback flow. Add future host integer decoding policy to `pointer_event/mapping.rs`, world-space UI status text to `pointer_event/world_space.rs`, docked-region toolbar frame behavior to `surface_frames/docked.rs`, floating-window toolbar frame behavior to `surface_frames/floating.rs`, single-pane frame projection behavior to `surface_frames/pane_frame.rs`, and future toolbar control id mappings to `viewport_toolbar_projection/hit_controls.rs`.
