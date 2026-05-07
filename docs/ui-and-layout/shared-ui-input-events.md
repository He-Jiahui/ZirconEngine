---
related_code:
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/metadata.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/event.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime_interface/src/ui/surface/navigation/event_kind.rs
  - zircon_runtime_interface/src/ui/layout/geometry.rs
  - zircon_runtime_interface/src/ui/component/drag.rs
  - zircon_runtime_interface/src/ui/component/event.rs
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime/src/ui/tests/popup_tooltip_state.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/tests/contracts.rs
implementation_files:
  - zircon_runtime_interface/src/ui/dispatch/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/metadata.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/event.rs
  - zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/text/edit_state.rs
  - zircon_runtime/src/ui/text/mod.rs
  - zircon_runtime/src/ui/surface/property_mutation.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/pointer_click_semantics.rs
  - zircon_runtime/src/ui/tests/popup_tooltip_state.rs
plan_sources:
  - docs/superpowers/plans/2026-05-06-ui-complete-input-events.md
  - user: 2026-05-06 implement Milestone 1 shared input contract foundation only
  - user: 2026-05-06 continue Milestone 2 runtime surface reply/effect application
tests:
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - cargo test -p zircon_runtime_interface --lib ui_input --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never
  - cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never
  - post-review-correction: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (45 passed, 0 failed, 3 filtered out)
  - quality-fix-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (18 passed, 0 failed, 869 filtered out)
  - quality-fix-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (36 passed, 0 failed, 851 filtered out)
  - quality-fix-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - post-review-fix-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (19 passed, 0 failed, 869 filtered out)
  - post-review-fix-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (36 passed, 0 failed, 852 filtered out)
  - post-review-fix-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - owner-cutover-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (49 passed, 0 failed, 3 filtered out)
  - owner-cutover-validation: cargo check -p zircon_runtime_interface --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed)
  - owner-cutover-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (20 passed, 0 failed, 869 filtered out)
  - owner-cutover-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (36 passed, 0 failed, 853 filtered out)
  - owner-cutover-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - owner-safety-final-validation: rustfmt --edition 2021 --config skip_children=true --check zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/input/state.rs zircon_runtime/src/ui/surface/input/validation.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/input/effect.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/event_routing.rs zircon_runtime/src/ui/tests/runtime_input_ownership.rs (passed)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (7 passed, 0 failed, 897 filtered out)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (20 passed, 0 failed, 884 filtered out)
  - owner-safety-final-validation: cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (38 passed, 0 failed, 866 filtered out)
  - owner-safety-final-validation: cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never (passed with existing warnings)
  - 2026-05-07-m5-route-validation: rustfmt --edition 2021 --check zircon_runtime_interface/src/ui/dispatch/input/reply.rs zircon_runtime_interface/src/ui/dispatch/input/mod.rs zircon_runtime_interface/src/ui/dispatch/mod.rs zircon_runtime_interface/src/ui/dispatch/pointer/event.rs zircon_runtime_interface/src/ui/dispatch/pointer/component_event.rs zircon_runtime_interface/src/tests/contracts.rs zircon_runtime/src/ui/surface/input/effect.rs zircon_runtime/src/ui/surface/input/mod.rs zircon_runtime/src/ui/surface/input/dispatch.rs zircon_runtime/src/ui/surface/surface.rs zircon_runtime/src/ui/tests/mod.rs zircon_runtime/src/ui/tests/runtime_input_ownership.rs zircon_runtime/src/ui/tests/pointer_click_semantics.rs (passed)
  - 2026-05-07-m5-route-validation: cargo test -p zircon_runtime_interface --lib contracts --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (54 passed, 0 failed, 6 filtered out)
  - 2026-05-07-m5-route-validation: cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (8 passed, 0 failed, 923 filtered out)
  - 2026-05-07-m5-route-validation: cargo test -p zircon_runtime --lib pointer_click_semantics --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (1 passed, 0 failed, 930 filtered out)
  - 2026-05-07-m5-route-validation: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (20 passed, 0 failed, 911 filtered out)
  - 2026-05-07-m5-popup-tooltip-validation: cargo test -p zircon_runtime --lib popup_tooltip_state --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (2 passed, 0 failed, 931 filtered out)
  - 2026-05-07-m5-drag-analog-validation: cargo test -p zircon_runtime --lib runtime_input_ownership --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (11 passed, 0 failed)
  - 2026-05-07-m5-drag-analog-validation: cargo test -p zircon_runtime --lib drag_drop_ --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (2 passed, 0 failed)
  - 2026-05-07-m5-drag-analog-validation: cargo test -p zircon_runtime --lib analog_input_suppresses_repeated_values_before_routing --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m5 --message-format short --color never (1 passed, 0 failed)
  - 2026-05-07-m5-native-validation: cargo test -p zircon_editor --lib native_input_translation --locked --jobs 1 --target-dir E:\zircon-build\targets-ui-m4 --message-format short --color never (5 passed, 0 failed)
doc_type: module-detail
---

# Shared UI Input Events

`zircon_runtime_interface::ui::dispatch::input` owns the neutral shared input-event DTOs for the M5 event-system contract. It is a folder-backed declaration subtree: `mod.rs` only declares and re-exports child modules, while each child file owns one declaration family.

## Contract Shape

`metadata.rs` defines shared event metadata: timestamp, sequence, user/device/window/surface ids, pointer id, modifiers, and the synthetic-event flag.

`event.rs` defines `UiInputEvent` variants for pointer, keyboard, text, IME, navigation, analog, drag/drop, popup, and tooltip timer input. It reuses existing shared DTOs where they already exist: `UiPointerEvent`, `UiNavigationEventKind`, `UiPoint`, and `UiDragPayload`. Pointer input can carry optional `UiPreciseScrollDelta` x/y metadata with line or pixel units while legacy `UiPointerEvent::scroll_delta` remains the scalar fallback. `UiPointerEvent.click_count` carries host-supplied click count for double-click semantics and defaults to one for old payloads. IME ranges use `UiTextByteRange`, whose offsets are explicit UTF-8 byte positions into the event text, and drag/drop events can carry a `UiDragSessionId` for cross-event correlation.

`reply.rs` defines `UiDispatchReply`, `UiDispatchDisposition`, and `UiDispatchPhase`. Replies are per-dispatch transient commands, not durable widget state, and can carry ordered `UiDispatchEffect` entries. Each reply may now record the handler node and Slate-style phase (`Preprocess`, `PreviewTunnel`, `Direct`, `Target`, `Bubble`, or `DefaultAction`). `UiDispatchReply::merge_route(...)` is the shared propagation combiner: unhandled and passthrough replies continue, while handled and blocked replies stop the route and prevent later tunnel/bubble/default effects from being applied.

`effect.rs` defines `UiDispatchEffect` for focus, pointer capture/release, pointer lock, high precision pointer, drag/drop, navigation, popup, tooltip, input method requests, dirty/redraw, and component event emission. Effects are also transient commands; runtime/editor persistence must go through normal tree, component, or host state paths. Owner-clearing effects carry the owner target (`ClearFocus`, `ReleasePointerCapture`, and `UnlockPointer`) so stale replies cannot clear another node's focus/IME, capture/high-precision, or pointer-lock state. Drag/drop effects carry target, pointer, optional session id, optional point, and optional payload so runtime diagnostics can distinguish simultaneous drags. Input-method requests carry surface-space cursor and composition rectangles rather than an origin-only anchor, giving host candidate windows enough geometry to follow both the caret and active composition spans. Input-method disable is encoded as `RequestInputMethod { kind: Disable }` so host requests keep one typed input-method channel.

`result.rs` defines `UiInputDispatchResult` plus diagnostics, applied/rejected effects, host requests, and component event reporting. Applied and rejected effects report the reply `effect_index` for duplicate-effect correlation. Host requests use the dedicated `UiDispatchHostRequestKind` enum instead of accepting arbitrary local effects. The result type is representational in Milestone 1 and becomes the runtime surface input result in Milestone 2.

## Runtime Surface Application

`zircon_runtime::ui::surface::input` is the runtime M2 consumer for the shared input contract. It is folder-backed so `surface.rs` remains the retained surface orchestration boundary instead of becoming the implementation sink for every input family.

`state.rs` adds `UiSurfaceInputState` beside the existing `UiFocusState`. It tracks per-surface input ownership that is not durable widget state: captured pointer id, high-precision owner, pointer-lock owner/policy, input-method owner, the latest input-method request geometry, popup stack entries, pending/visible tooltip state, a shared drag/drop lifecycle record, and analog control values. Capture cleanup is owner-aware: capture loss clears the shared pointer id, while high precision is cleared only for the released or replaced captor. Popup effects open/close/toggle entries in a surface-local stack, and tooltip effects arm, show, hide, or cancel a single surface-local tooltip record before the host turns those results into platform UI.

`UiSurfaceDragDropState` stores the active drag source, target, pointer id, session id, current surface-space point, optional payload, and accepted/rejected result. `begin_drag_drop(...)` rejects a second concurrent drag and records the source owner; `update_drag_drop(...)`, `accept_drag_drop(...)`, `reject_drag_drop(...)`, and `end_drag_drop(...)` all validate pointer/session ownership so stale drag events cannot clear a live newer drag. `UiSurfaceAnalogControlState` stores the last routed value per control. Repeated values within the small equality threshold are suppressed before routing and recorded with an `analog_repeat_suppressed` diagnostic note, which prevents gamepad stick noise from forcing repeated hover/action work.

`validation.rs` keeps the runtime owner predicate shared between effect application and owner-routed dispatch. It accepts only existing, enabled owners with render-visible self and ancestor chains, so text/IME ownership cannot outlive hidden or collapsed parents.

`effect.rs` applies ordered `UiDispatchEffect` values once through `UiSurface::apply_dispatch_reply(...)`. It mutates existing focus/capture/navigation/tree dirty state, validates node ownership before accepting target-owned effects, records applied/rejected effects by `effect_index`, emits component event reports for accepted component events, and converts host-owned effects into typed `UiDispatchHostRequestKind` requests for the editor or runtime host. `UiSurface::apply_dispatch_reply_steps(...)` consumes phased reply steps through the shared merge contract before applying effects, so preview/tunnel handling can stop later bubble/default effects at the same runtime seam. Safety checks reject invalid input-method owners before host requests, reject stale input-method reset/update/disable requests whose owner no longer matches the surface input state, reject pointer-capture release requests whose target/pointer id do not match the current capture, clear stale pointer ids on direct capture, reject stale pointer unlock requests from non-owners, reject stale high-precision disable requests from non-owners, and reject high-precision enable unless the same owner already has live pointer capture.

Drag/drop effects now mutate the same surface input state as capture and high precision. Begin validates the source owner and captures the pointer for the drag source; update changes target/point/payload only for the current session; accept/reject store the drop result; complete/cancel clear drag state and release the capture/high-precision owner that started the drag. This is the runtime-side cleanup contract for editor drag overlays, drawer tab detaches, asset drops, and future runtime drag targets: stale end events are rejected rather than clearing the current drag.

`dispatch.rs` adds `UiSurface::dispatch_input_event(...)` as the shared input entry point. Pointer and navigation events delegate to existing runtime dispatchers and then wrap their legacy results in `UiInputDispatchResult`. Keyboard events route through the focused path and record focused-route diagnostics. Text events route to a valid input-method owner or, after clearing stale IME ownership, to the focused node. IME cancel clears the input-method owner and records an explicit diagnostic note; stale IME owners are also cleared and reported instead of remaining sticky. Pointer scroll keeps the legacy scalar fallback while preserving the original shared `UiPointerInputEvent`, including optional precise x/y/unit scroll metadata, in the returned `UiInputDispatchResult.event`. Analog events update surface-local analog control state before routing; unchanged values are returned as `Unhandled` with `routed = false`, so hosts can keep polling devices without rebuilding presentation on unchanged input.

2026-05-07 M6 closes the editable text mutation gap on this same route. `UiInputEvent::Text` and IME preedit/commit/cancel no longer stop at owner diagnostics: they build `UiEditableTextState`, apply `UiTextEditAction`, mutate focused/input-method owner node properties, and emit component event reports for the authored Change/Submit bindings. Preedit records composition range/text plus restore text, cancel restores the saved text and clears the input-method owner, and commit replaces the composition span before firing the submit/commit report. `property_mutation.rs` maps value edits to layout+render+text dirtiness, while caret, selection, and composition edits stay render+text only.

Focused validation for this closure used `E:\zircon-build\targets-ui-m6`: runtime `event_routing` passed 22 / 0 with shared text and IME mutation regressions; runtime `text_layout` passed 11 / 0; runtime-interface `render_contracts` passed 27 / 0; and editor/runtime text paint focused gates confirmed the shared render DTO still carries caret, selection, composition, rich runs, and font/atlas resource identity.

## Milestone Boundaries

Milestone 1 intentionally does not modify runtime or editor behavior. It only makes the shared DTO vocabulary serializable and constructible in `zircon_runtime_interface` so later milestones can route through a common input contract instead of adding another editor- or runtime-owned event vocabulary.

Milestone 2 changes runtime surface behavior only at the shared `UiSurface` seam. It preserves existing pointer click, hover, capture, scroll, and navigation tests while adding shared reply/effect application and focused keyboard/text/IME diagnostics. It does not implement M6 caret, selection, shaping, or text rendering, and it does not add editor-native event translation; those remain later milestones.

Contract tests in `zircon_runtime_interface/src/tests/contracts.rs` construct every event family and every effect family. They also round-trip pointer, keyboard, IME, drag/drop, popup, tooltip, and input-method request payloads through serde JSON.

Runtime tests in `zircon_runtime/src/ui/tests/event_routing.rs`, `zircon_runtime/src/ui/tests/runtime_input_ownership.rs`, `zircon_runtime/src/ui/tests/pointer_click_semantics.rs`, and `zircon_runtime/src/ui/tests/popup_tooltip_state.rs` cover focus/capture/high-precision reply effects, owner-targeted clear-focus behavior, rejected focus preserving the current IME owner, navigation and focus changes clearing previous IME owners, direct clear-focus cleanup, focus/capture rejection through hidden ancestors, phased reply propagation stopping later bubble effects, direct capture clearing stale pointer ids before high precision can enable, stale capture release rejection, stale pointer unlock rejection, stale high-precision disable rejection, high-precision enable requiring live capture, capture transfer clearing the previous captor's high precision, navigation/input-method host requests, input-method reset/update/disable current-owner checks, invalid and stale input-method owner rejection, focused keyboard diagnostics, text owner routing and fallback after stale IME cleanup, hidden-ancestor owner rejection, IME owner cleanup, scroll diagnostics, double-click component event generation on the shared route, popup stack open/toggle state, and tooltip pending/visible/cancel state. The broader `shared_core` filter preserves the existing shared layout/hit/focus/navigation baseline while the runtime surface input state is present.
