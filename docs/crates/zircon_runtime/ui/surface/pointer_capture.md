---
related_code:
  - zircon_runtime/src/ui/surface/input/state/mod.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_capture.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_press.rs
  - zircon_runtime/src/ui/surface/input/pointer.rs
  - zircon_runtime/src/ui/surface/input/effect/focus_pointer.rs
  - zircon_runtime/src/ui/surface/input/text_pointer.rs
  - zircon_runtime/src/ui/surface/input/window_pump.rs
  - zircon_runtime/src/ui/surface/surface/event_routing/pointer_ownership.rs
  - zircon_runtime/src/ui/surface/surface/pointer_component_events/state_invalidation.rs
  - zircon_runtime/src/ui/surface/surface/interaction_state.rs
  - zircon_runtime/src/ui/surface/focus.rs
implementation_files:
  - zircon_runtime/src/ui/surface/input/state/pointer_capture.rs
  - zircon_runtime/src/ui/surface/input/state/pointer_press.rs
  - zircon_runtime/src/ui/surface/surface/event_routing/pointer_ownership.rs
plan_sources:
  - docs/plans/optimize/zircon_runtime/82/2026-09-27-pointer-button-capture-ownership.md
tests:
  - zircon_runtime/src/ui/tests/event_routing/pointer_state.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_capture_routes.rs
  - zircon_runtime/src/ui/tests/runtime_input_reply_routes/pointer_capture_routes/ownership_lifecycle.rs
  - zircon_runtime/src/ui/tests/widget_range_navigation.rs
  - zircon_runtime/src/ui/tests/widget_scrollbar_behavior.rs
doc_type: module-detail
---

# Pointer press and capture ownership

`UiFocusState` remains the public node-only projection. Runtime retains press and capture records separately by `UiPointerId`; an input event projects only its pointer's records into surface routing. Calls without pointer metadata use the default pointer id. The dispatch context is restored after the complete component/text route, including error returns.

## Button admission

| State | Admission and release |
|---|---|
| Press | Down records the node and button. A different button cannot replace an unreleased press. Matching Up clears it; Primary click additionally requires the original pressed node in the physical hit path. Existing same-Primary repeated Down may retarget the press. |
| Capture acquired by Down | Capture retains the initiating button. Repeated capture of the same owner preserves that qualification; a foreign-button Down cannot transfer capture to a different owner. Only matching Up automatically releases it. |
| Capture acquired without a button event | The legacy unqualified capture accepts any Up. Programmatic capture/release remains available. |
| Explicit `ReleasePointerCapture` | Validates pointer id and owner, releases that capture and its qualification, and leaves a still-held press intact. |
| Cancel | Ends that pointer's press and capture regardless of button. Other pointers, including captures of the same node, remain independent. |

Foreign-button events still reach normal custom handlers. Their edge does not activate default widgets, end text/range/scrollbar dragging, or open the text context menu. Hit testing, hover and focus updates continue. Surface reply application rejects a cross-owner foreign-button capture before mutation; atomic reply rollback restores both ownership maps and emits no host requests from the aborted transaction.

The public component `pressed` flag remains set while any retained Primary press belongs to that node. Captured movement stays routed outside the hit rectangle. Terminal routing preserves the old route in diagnostics while subsequent input uses the remaining live pointer records.

Multi-pointer coverage here verifies press and capture ownership. Widget drag state remains keyed by node; simultaneous multi-touch widget dragging and native multi-device qualification require their own contracts and acceptance.

## Lifecycle

Detached, removed or disabled owners lose all their press/capture records, including records not currently projected in `UiFocusState`. Hot reload resets both maps. Window focus loss and application deactivation use the shared interaction cleanup after transient UI dismissal; Closed/Destroyed also clear remaining pointers after the existing Cancel path. CursorLeft keeps the existing single-pointer Cancel when a cursor position is available and the global fallback cleanup when no position is available.

Window cleanup clears all presses, captures, drag state and high-precision ownership, and updates node/component pressed flags. Pointer-lock and other input services retain their existing independently owned lifecycle.

## Boundaries

The lower route and capture methods live in `surface/event_routing/pointer_ownership.rs`; dispatch composition stays in `surface/event_routing.rs`. Pointer state lives in Runtime, without new button fields in the public `UiFocusState` DTO or Editor-specific recapture rules. This contract covers pointer/button ownership. Native capture generations and Editor59's scale/performance gates remain separate acceptance work.
