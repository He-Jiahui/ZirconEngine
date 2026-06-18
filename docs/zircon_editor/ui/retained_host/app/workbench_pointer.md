---
related_code:
  - zircon_editor/src/ui/retained_host/app.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/document_tabs.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/floating_window.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/shell_chrome.rs
  - zircon_editor/src/ui/retained_host/app/callback_wiring.rs
  - zircon_editor/src/ui/retained_host/activity_rail_pointer/mod.rs
  - zircon_editor/src/ui/retained_host/document_tab_pointer/mod.rs
  - zircon_editor/src/ui/retained_host/drawer_header_pointer/mod.rs
  - zircon_editor/src/ui/retained_host/host_page_pointer/mod.rs
  - zircon_editor/src/ui/retained_host/shell_pointer/mod.rs
implementation_files:
  - zircon_editor/src/ui/retained_host/app/workbench_pointer.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/document_tabs.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/floating_window.rs
  - zircon_editor/src/ui/retained_host/app/workbench_pointer/shell_chrome.rs
plan_sources:
  - docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md
  - user: 2026-06-18 editor UI architecture implementation, feature first and tests deferred
tests:
  - cargo fmt -p zircon_editor
  - cargo fmt -p zircon_editor --check
  - app Workbench pointer ownership scan
  - cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never
doc_type: module-detail
---

# Retained Host Workbench Pointer Boundary

## Purpose

The Workbench pointer app boundary owns retained-host callbacks for the outer editor shell controls that are not pane-content callbacks. These callbacks take native/template pointer data, reuse the committed pointer layout, dispatch through shared retained-host pointer bridges, and apply resulting Workbench effects.

The boundary is split by UI region:

- `workbench_pointer/shell_chrome.rs` owns activity rail, host page tabs, and drawer header tab clicks.
- `workbench_pointer/document_tabs.rs` owns document tab activation and document tab close clicks.
- `workbench_pointer/floating_window.rs` owns native floating-window header focus routing.
- `workbench_pointer.rs` is a structural module entry only.

## Related Files

- `zircon_editor/src/ui/retained_host/app/callback_wiring.rs` registers the native/template callbacks that call these retained-host methods.
- `zircon_editor/src/ui/retained_host/activity_rail_pointer/mod.rs`, `document_tab_pointer/mod.rs`, `drawer_header_pointer/mod.rs`, and `host_page_pointer/mod.rs` provide the bridge/layout types used by these dispatch methods.
- `zircon_editor/src/ui/retained_host/shell_pointer/mod.rs` provides floating-window hit routing for the floating header focus path.

## Behavior Model

Every Workbench pointer callback first calls `use_committed_pointer_layout()`. That keeps pointer dispatch on the last committed bridge frames and prevents expensive Workbench recomputation inside native callback handlers.

Shell chrome clicks validate the callback payload, then dispatch through their matching shared pointer bridge:

- activity rail side strings are parsed through `HostActivityRailPointerSide::parse(...)`;
- host page tab indices and drawer header tab indices reject negative values with a status-line error;
- valid callbacks dispatch through the matching `callback_dispatch::dispatch_shared_*` function.

Document tab clicks and close clicks use the document tab pointer bridge and then call `note_focused_floating_window_surface(...)`. This preserves floating-window focus ownership when a document tab event comes from a native child window.

Floating-window header clicks use the committed shell pointer bridge to find a floating-window route at the pointer position. If the route belongs to a floating window or floating-window edge, the callback dispatches the built-in floating-window focus command and records the focused native window.

## Design and Rationale

The old single file mixed several Workbench regions that change independently. The split keeps shell chrome, document tabs, and floating-window focus behavior in separate files while preserving the same callback method names expected by generated host globals and callback wiring.

The methods are visible only inside `crate::ui::retained_host::app`, matching their role as app-local callback targets. The root module stays declarative so future Workbench pointer surfaces can be added as new children instead of extending a mixed callback file.

## Control Flow

Shell chrome flow:

1. Callback wiring invokes the region-specific retained-host method.
2. The host reuses committed pointer layout diagnostics.
3. Payload validation happens at the app boundary.
4. The shared retained-host pointer dispatcher emits host effects.
5. Effects update runtime/layout state through the normal retained-host effect pipeline.

Document tab flow:

1. Document tab or close callbacks validate tab index and surface key.
2. The document tab pointer bridge dispatches activation or close intent.
3. Dispatch effects are applied.
4. The surface key updates floating-window focus tracking.

Floating-window header flow:

1. The shell pointer bridge resolves the route at the clicked point.
2. Non-floating routes are ignored.
3. Floating-window routes dispatch built-in focus and update host focus state.

## Edge Cases and Constraints

- Negative tab indices never enter the lower-level dispatch bridge.
- Floating-window header clicks are no-ops when hit testing resolves to drag targets, document edges, resize handles, or empty space.
- The split does not rename any generated callback methods or stable Workbench pointer callback ids.
- These callbacks should not recompute Workbench layout directly; layout updates are consumed by tick/refresh.

## Test Coverage

Implementation-slice validation covers formatting, ownership scanning, scoped diff checking, and `cargo check -p zircon_editor --lib --locked --jobs 1 --message-format short --color never`. Existing retained-host app tests still exercise document tab, host page, activity rail, and floating-window header callbacks. Full Cargo tests remain deferred to the milestone testing stage per the user's instruction.

## Plan Sources

This boundary belongs to `docs/plans/zircon_editor/editor_ui/08-workbench-shell-on-runtime-ui.md`, M3.S2, which requires Workbench shell interactions to be owned by narrow retained-host modules while runtime UI surfaces remain the source of rendered shell controls.

## Open Issues or Follow-up

- The milestone testing stage still needs the declared `zircon_editor` unit/integration test commands.
- Additional Workbench chrome pointer surfaces should be added as child modules under this folder instead of growing the root file.
