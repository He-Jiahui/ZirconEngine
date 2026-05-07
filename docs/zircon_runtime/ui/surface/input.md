---
related_code:
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime_interface/src/ui/dispatch/input/mod.rs
  - zircon_runtime_interface/src/ui/dispatch/input/event.rs
  - zircon_runtime_interface/src/ui/dispatch/input/reply.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/ui/dispatch/input/result.rs
  - zircon_runtime_interface/src/tests/contracts.rs
implementation_files:
  - zircon_runtime/src/ui/surface/input/mod.rs
  - zircon_runtime/src/ui/surface/input/state.rs
  - zircon_runtime/src/ui/surface/input/validation.rs
  - zircon_runtime/src/ui/surface/input/effect.rs
  - zircon_runtime/src/ui/surface/input/dispatch.rs
  - zircon_runtime/src/ui/surface/surface.rs
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime_interface/src/ui/dispatch/input/effect.rs
  - zircon_runtime_interface/src/tests/contracts.rs
plan_sources:
  - docs/superpowers/specs/2026-05-06-ui-complete-input-events-design.md
  - docs/superpowers/plans/2026-05-06-ui-complete-input-events.md
  - user: 2026-05-06 continue Milestone 2 runtime surface reply/effect application
tests:
  - zircon_runtime/src/ui/tests/event_routing.rs
  - zircon_runtime/src/ui/tests/runtime_input_ownership.rs
  - zircon_runtime/src/ui/tests/shared_core.rs
  - cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo test -p zircon_runtime --lib shared_core --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture
  - cargo check -p zircon_runtime --lib --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never
  - post-review-correction: cargo test -p zircon_runtime --lib event_routing --locked --jobs 1 --target-dir E:\zircon-build\targets\ui-complete-input-events --message-format short --color never -- --nocapture (16 passed, 0 failed, 860 filtered out)
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
doc_type: module-detail
---

# Runtime UI Surface Input

`zircon_runtime::ui::surface::input` applies the shared M5 input contract to retained runtime UI surfaces. The module is intentionally below `surface/` because `UiSurface` owns focus, capture, navigation, tree dirty state, and retained frame data; hosts only translate native input into shared DTOs and consume host requests returned by dispatch.

## Module Shape

`mod.rs` is structural. It exposes `UiSurfaceInputState` publicly through `zircon_runtime::ui::surface` and keeps `apply_dispatch_reply(...)` plus `dispatch_input_event(...)` crate-private implementation details called by `UiSurface` methods.

`state.rs` stores transient per-surface input ownership that does not belong on individual widgets. The current M2 fields are the captured pointer id, high-precision owner, pointer-lock owner/policy, input-method owner, and latest input-method request. Pointer-capture cleanup is owner-aware: the shared captured pointer id is cleared for capture loss, and high-precision ownership is cleared only when it belongs to the released or replaced captor.

`validation.rs` centralizes input-owner checks used by effect application and owner-routed dispatch. A valid owner must exist, be enabled, be render-visible itself, and have a render-visible ancestor chain. This keeps text/IME/high-precision ownership aligned with the same visibility semantics used by arranged hit/render paths.

`effect.rs` consumes `UiDispatchReply` values. It applies ordered `UiDispatchEffect` entries, records applied and rejected effects with their source `effect_index`, validates target nodes before mutating surface state, marks dirty flags for dirty/redraw requests, and creates typed host requests for pointer lock, pointer unlock, high precision pointer, popup, tooltip, and input method effects. Input-method enable requests can establish a valid owner; reset/update/disable requests require the current IME owner and are rejected before host requests when stale. Owner-sensitive effects now carry their owner target in the shared DTO: `ClearFocus { target, .. }`, `ReleasePointerCapture { target, pointer_id, .. }`, and `UnlockPointer { target, .. }`. Runtime application rejects stale targets before clearing focus/IME, capture/high-precision, or pointer-lock state. `SetFocus` and navigation clear a previous IME owner only after focus successfully changes, direct capture clears stale pointer ids, and `UseHighPrecisionPointer { enabled: true }` requires live capture for the same owner.

`dispatch.rs` is the shared entry point adapter. Pointer and navigation events keep delegating to the existing runtime dispatchers, preserving their behavior while projecting the result into `UiInputDispatchResult`. Pointer scroll results keep the original shared pointer input event, including optional precise x/y/unit wheel metadata, even while legacy dispatch consumes the scalar fallback. Keyboard input records the focused route. Text input uses a valid IME owner when present, clears stale IME ownership, then falls back to the focused node. IME input clears stale or cancelled IME ownership and reports an `owner route rejected` diagnostic when an invalid stored owner was present. Other owner-routed families validate that the stored owner still exists, is enabled, and has render-visible ancestors before reporting a handled route.

## Surface Integration

`UiSurface` now stores `input: UiSurfaceInputState` with serde defaults so old retained surface snapshots can deserialize without input-state fields. Public methods `UiSurface::apply_dispatch_reply(...)` and `UiSurface::dispatch_input_event(...)` keep the shared runtime seam explicit while the implementation remains in the child module.

Pointer capture release paths clear the shared pointer id in addition to clearing `UiFocusState::captured`. When the released owner also owns high precision, `clear_pointer_capture_for(owner)` clears both states; otherwise high precision is left alone so a stale release cannot clear another owner's raw-input state.

## Validation Scope

Focused runtime coverage lives in `zircon_runtime/src/ui/tests/event_routing.rs` and `zircon_runtime/src/ui/tests/runtime_input_ownership.rs`. The latter was split out because `event_routing.rs` is already above the large-file warning threshold. Together they verify focus/capture/high-precision reply effects, owner-targeted clear-focus behavior, rejected focus preserving the current IME owner, navigation/focus changes clearing previous IME owners, direct clear-focus cleanup, focus/capture rejection through hidden ancestors, direct capture clearing stale pointer ids before high precision can enable, stale pointer-capture release rejection, stale pointer-lock unlock rejection, high-precision enable requiring live capture, stale high-precision disable rejection, capture transfer clearing the previous captor's high precision, navigation plus host-owned input effects, input-method reset/update/disable current-owner checks, invalid input-method owner rejection, focused keyboard diagnostics, text owner routing/fallback after stale IME cleanup, stale IME owner-route rejection, hidden-ancestor owner rejection, IME owner cleanup, and pointer scroll diagnostics plus precise scroll metadata preservation through the shared input result path.

The M2 scope deliberately does not implement M6 text layout, caret, selection, shaping, or editor-native keyboard/IME translation. Those systems should consume this shared input state and result contract instead of adding host-owned focus, capture, or IME semantics.
