---
related_code:
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/ui/dispatch/input_manager/outcome.rs
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs
  - zircon_runtime/src/ui/dispatch/input_manager/routing.rs
  - zircon_runtime/src/ui/dispatch/input_manager/timers.rs
  - zircon_runtime/src/ui/surface/mod.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/window/mod.rs
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
implementation_files:
  - zircon_runtime/src/ui/dispatch/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/mod.rs
  - zircon_runtime/src/ui/dispatch/input_manager/manager.rs
  - zircon_runtime/src/ui/dispatch/input_manager/outcome.rs
  - zircon_runtime/src/ui/dispatch/input_manager/pointer_table.rs
  - zircon_runtime/src/ui/dispatch/input_manager/routing.rs
  - zircon_runtime/src/ui/dispatch/input_manager/timers.rs
  - zircon_runtime/src/ui/surface/surface.rs
plan_sources:
  - user: 2026-06-12 implement editor UI architecture from docs/plans/zircon_editor/editor_ui
  - docs/plans/zircon_editor/editor_ui/index.md
  - docs/plans/zircon_editor/editor_ui/01-slate-input-dispatch-core.md
tests:
  - zircon_runtime/src/ui/tests/runtime_input_manager.rs
  - 2026-06-12: cargo check -p zircon_runtime_interface --lib --locked (passed)
  - 2026-06-12: cargo test -p zircon_runtime_interface --lib ui_layout_style_and_debug_packet_contracts_round_trip_with_defaults --locked --target-dir target/codex-editor-ui (passed)
  - 2026-06-12: target/codex-editor-ui-runtime/debug/deps/zircon_runtime-de6f737e1b69a0f9.exe runtime_input_manager --nocapture --test-threads=1 (passed, 3 passed)
  - 2026-06-12: cargo test -p zircon_runtime --lib runtime_input_manager --locked --jobs 1 --target-dir target/codex-editor-ui-runtime --message-format short --color never was blocked during rebuild by unrelated unresolved import crate::core::frame_clock in zircon_runtime/src/core/runtime/state/runtime_inner.rs.
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
- `UiInputTimerState` for manager-owned dispatch ticks.

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

Concrete handlers do not iterate this constant yet. The constant is a contract anchor and testable authority list so later pointer, keyboard, IME, drag-drop, tooltip, and editor command routes converge on one order instead of each path re-declaring its own precedence.

## Batch Outcome

`UiInputDispatchOutcome` aggregates a window input batch into:

- ordered `UiInputDispatchResult` values,
- merged host requests from all results,
- a `redraw_requested` summary derived from `UiSurfaceWindowState.redraw_requested` or applied `DirtyRedraw` effects.

This gives the editor host a single result object for window-pump batches while preserving individual event diagnostics. The outcome is deliberately not serialized in this slice because it is runtime-local aggregation, not a cross-crate contract.

## Current Limits

`UiInputManager::tick` currently records `last_tick` and returns no injected events. The plan expects future slices to inject tooltip timers, repeat actions, IME updates, drag-hover updates, and command-clock events from this owner. Until then, surface input timer modules remain the behavioral implementation.

The manager also does not yet mutate `UiActivePointerTable` during dispatch. The table is exposed now so later pointer normalization can capture mouse, touch, pen, and multi-pointer state before events enter surface routing.
