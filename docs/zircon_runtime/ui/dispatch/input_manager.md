---
related_code:
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager/tests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/outcome.rs
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs
  - zircon_runtime/src/ui/dispatch/input_manager/routing.rs
  - zircon_runtime/src/ui/dispatch/input_manager/timers.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/route_authority.rs
  - zircon_runtime/src/ui/surface/input/tooltip_timer.rs
  - zircon_runtime/src/ui/surface/input/toast_timer.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/toast_timer.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/window/mod.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_dispatch_input_manager_tests.rs
implementation_files:
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager/tests.rs
  - zircon_runtime/src/ui/dispatch/input_manager/outcome.rs
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs
  - zircon_runtime/src/ui/dispatch/input_manager/routing.rs
  - zircon_runtime/src/ui/dispatch/input_manager/timers.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/route_authority.rs
  - zircon_runtime/src/ui/surface/input/tooltip_timer.rs
  - zircon_runtime/src/ui/surface/input/toast_timer.rs
  - zircon_runtime/src/ui/surface/surface/default_interactions/toast_timer.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
plan_sources:
  - docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/index.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
tests:
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager/tests.rs
  - zircon_runtime/src/tests/runtime_absorption/structure_convention/production_file_budget/ui_dispatch_input_manager_tests.rs
  - 2026-06-12: cargo check -p zircon_runtime_interface --lib --locked (passed)
  - 2026-06-12: cargo test -p zircon_runtime_interface --lib ui_layout_style_and_debug_packet_contracts_round_trip_with_defaults --locked --target-dir target/codex-editor-ui (passed)
  - 2026-06-12: target/codex-editor-ui-runtime/debug/deps/zircon_runtime-de6f737e1b69a0f9.exe runtime_input_manager --nocapture --test-threads=1 (passed, 3 passed)
  - 2026-06-12: cargo test -p zircon_runtime --lib runtime_input_manager --locked --jobs 1 --target-dir target/codex-editor-ui-runtime --message-format short --color never was blocked during rebuild by unrelated unresolved import crate::core::frame_clock in zircon_runtime/src/core/runtime/state/runtime_inner.rs.
  - 2026-06-15: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/input/event.rs zircon_runtime_interface/src/ui/dispatch/input/mod.rs zircon_runtime_interface/src/ui/dispatch/mod.rs zircon_runtime_interface/src/tests/contracts.rs zircon_runtime/src/ui/dispatch/input_manager/timers.rs zircon_runtime/src/ui/dispatch/input_manager/manager.rs zircon_runtime/src/ui/surface/surface/default_interactions.rs zircon_runtime/src/ui/surface/surface/default_interactions/toast_timer.rs zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/toast_timer.rs zircon_runtime/src/ui/surface/input/route_policy.rs zircon_runtime/src/ui/surface/input/owner_route.rs zircon_runtime/src/ui/surface/input/editable_text/ime_context.rs zircon_runtime/src/ui/tests/runtime_input_reply_routes.rs zircon_runtime/src/ui/tests/runtime_input_reply_routes/keyboard_navigation_routes.rs (M3.S2 Snackbar/Toast input-manager auto-hide timer dispatch: passed)
  - 2026-06-15: git diff --check -- touched tracked ToastTimer dispatch Rust/docs/session files (passed with LF-to-CRLF warnings only); conflict marker and trailing-whitespace scans passed with no matches.
  - 2026-06-15: cargo test -p zircon_runtime --lib toast_timer --locked and cargo test -p zircon_runtime_interface --lib ui_input_payloads_round_trip_through_serde --locked were deferred because active cargo/rustc lanes were present in the shared Windows workspace.
doc_type: module-detail
---

# UI Input Manager

`zircon_runtime::ui::dispatch::UiInputManager` is the runtime owner for editor-style input dispatch state. It is intentionally placed in `ui::dispatch`, not `ui::surface::input`, because it coordinates event routing across pointer dispatch, navigation dispatch, active pointer ownership, timers, and window-pump batches while leaving leaf behavior in the existing surface input modules.

The first slice is a non-invasive manager shell. Existing `UiSurface::dispatch_input_event`, `dispatch_window_input_pump_event`, and `dispatch_window_input_pump_batch` entry points remain available for callers that still pass `UiPointerDispatcher` and `UiNavigationDispatcher` directly. New `*_with_manager` entry points let editor/runtime hosts move toward a single input owner without changing the lower-level dispatch replies.

## Ownership

`UiInputManager` owns:

- `UiPointerDispatcher` for pointer handlers and pointer-route callbacks.
- `UiNavigationDispatcher` for keyboard/gamepad/navigation handlers.
- `UiActivePointerTable` for per-pointer source, last known position, pressed-button mask, capture target, and primary-pointer status.
- `UiInputTimerState` for manager-owned dispatch ticks, including menu typeahead expiry, submenu hover readiness, Tooltip delayed-open, and Snackbar/Toast auto-hide expiry.

`UiSurface` still owns arranged-tree state, popup/tooltip state, focus state, component states, window state, dirty flags, and the actual dispatch effect application. The manager passes its dispatchers into `ui::surface::input` so the existing routing implementation remains the single behavior authority until later slices move pointer capture, preview tunneling, and timer injection behind the manager.

## Route Authority

`UI_INPUT_ROUTE_ORDER` fixes the cross-cutting order required by the editor UI plan:

1. pointer capture
2. popup stack
3. preview tunnel
4. direct target
5. bubble path
6. focus path
7. default action

`zircon_runtime::ui::surface::input::route_authority` now consumes this constant for normalized `UiInputEvent` dispatch. `surface/input/dispatch.rs` collects each leaf dispatch result, calls `annotate_authoritative_input_dispatch`, and records a `route_authority=runtime_09_m1_1_ui_input_route_authority;policy=...;stages=...` diagnostic note derived from `UI_INPUT_ROUTE_ORDER`.

Direct `dispatch_pointer_event` and `dispatch_navigation_event` methods still exist on `UiSurface` and `RuntimeUiManager` as leaf owner helpers for existing low-level callers and tests. They are not the normalized `UiInputEvent` route; Runtime 09 tracks them under `runtime_09_m1_1_direct_pointer_navigation_routes_are_leaf_owner_helpers` until those call sites are migrated or retired.

## Batch Outcome

`UiInputDispatchOutcome` aggregates a window input batch into:

- ordered `UiInputDispatchResult` values,
- merged host requests from all results,
- a `redraw_requested` summary derived from `UiSurfaceWindowState.redraw_requested` or applied `DirtyRedraw` effects.

This gives the editor host a single result object for window-pump batches while preserving individual event diagnostics. The outcome is deliberately not serialized in this slice because it is runtime-local aggregation, not a cross-crate contract.

## Timer Dispatch

The manager injects synthetic input events from retained deadlines rather than asking component reducers to observe wall-clock time directly. Menu/MenuList typeahead expiry and submenu hover readiness already use this pattern; the M3.S2 Snackbar/Toast slice adds the same owner for auto-hide.

`UiInputTimerState` stores Toast deadlines by target node together with the `toast_id` that was current when the timer was armed. `UiInputManager::arm_timers_from_component_events(...)` arms/replaces/clears that deadline from `toast_queue` payloads, `current_toast_id`, `auto_hide_duration_ms`/`autoHideDuration`, open/close events, or retained Snackbar/Toast state discovered through `UiSurface::toast_timer_for_component_node(...)`. `tick(...)` drains expired Toast timers and dispatches `UiInputEvent::ToastTimer`; `surface/input/toast_timer.rs` turns a matching current id into `Commit { property: "expired_toast_id" }` and annotates stale timers as ignored.

The event is part of the public runtime-interface input contract through `UiToastTimerInputEvent`, so hosts and tests can observe the same route as internally injected manager ticks.

Tooltip delayed-open uses the same ownership boundary. Component hover reports arm retained tooltip identity and a manager deadline; `tick(...)` dispatches `UiTooltipTimerInputEvent::Elapsed`, and the surface rejects it when the retained owner/id no longer matches. `DEFAULT_TOOLTIP_DELAY_MS` is exported from `ui::dispatch` so generic surface metadata and host-resolved candidates share one 150 ms default; authored `tooltip_delay_ms` may override it, including `0` for an intentional immediate hint. A successful delayed Show starts the manager-owned `DEFAULT_TOOLTIP_INTRO_DURATION_MS = 100` timeline. During that timeline `next_frame_visible_delay(...)` returns at most a 16 ms sample delay, `tooltip_intro_progress(...)` exposes the current normalized progress, and completion or dismissal clears the timeline. Tooltip `transition_progress` and `transition_status` mutations are render-only invalidations, so those samples do not rebuild layout, hit testing, text, or input state. A host that resolves richer presentation metadata can call `arm_tooltip_candidate(...)` and `dismiss_tooltip(...)`; these APIs do not give the host a second timer or visible-state authority. The host must wake and tick the same manager using the original input timestamp domain, then project its custom popup and intro progress only after the retained tooltip becomes visible.

## Runtime 15 M4 UI dispatch input manager test owner split

Status: `runtime_15_ui_dispatch_input_manager_tests_owner_split_static_passed_cargo_deferred`.

Runtime 15 M4 keeps the production input manager behavior in `ui/dispatch/input_manager/manager.rs` and moves only its inline test owner to `ui/dispatch/input_manager/manager/tests.rs`. The parent continues to own `UiInputManager`, `dispatch_input_event(...)`, `dispatch_window_input_pump_batch(...)`, `tick(...)`, timer arming/drain helpers, active pointer helpers, and timestamp extraction. The child test owner keeps the existing submenu hover, popup typeahead, Toast auto-hide, Tooltip hover/timer, and fixture coverage.

`runtime_15_ui_dispatch_input_manager_tests_are_child_owner` verifies the parent still mounts `#[cfg(test)] mod tests;`, the seven moved tests do not return to the production file, both owners stay under the Runtime 15 file budget, and Runtime 15/status/UI/module docs contain the completion anchors. This is a structure-only split; it does not change route ordering, event payloads, timer deadlines, host requests, or dispatch replies. Cargo remains deferred by the Runtime 15 slice cadence while external cargo/rustc lanes are active.

## Current Limits

`UiInputManager::tick` now injects the implemented component timer families but still does not own every planned timed behavior. Repeat actions, IME updates, drag-hover updates, and command-clock events remain future slices, and surface input timer modules still contain the leaf route behavior.

The manager also does not yet mutate `UiActivePointerTable` during dispatch. The table is exposed now so later pointer normalization can capture mouse, touch, pen, and multi-pointer state before events enter surface routing.
