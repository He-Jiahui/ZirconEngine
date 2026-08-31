---
title: Runtime12 Physical Input Before UI Ownership RED
date: 2026-08-24
status: red_source_complete_dynamic_pending
parent_plan: docs/plans/zircon_runtime/runtime/12-input-stack-and-action-mapping.md
canonical_review: docs/plans/optimize/zircon_runtime/99r-runtime-input-device-event-frame-state-action-map-focus-gamepad-recording-replay-host-product-integration-current-source-review.md
session: runtime12-physical-first-red-r1-20260824
---

# Runtime12 Physical Input Before UI Ownership RED

## Scope

This M117.0 / Runtime12 slice freezes one behavior-level reproduction for `INP-P0-002` and
`INP-G03`. It does not modify the dirty shared production owners
`zircon_runtime/src/dynamic_api/session/events.rs` or
`zircon_runtime/src/dynamic_api/session/runtime_ui.rs`.

The existing Runtime12 gamepad-storm failure remains open. It owns App producer batching and is not
closed by this UI ownership test.

## Architecture Finding

Current dynamic-session mouse handling dispatches to Runtime UI before calling
`submit_input_event`. A propagation stop returns immediately, so a release can be hidden from the
physical `InputManager`. The deterministic scenario is:

1. press a Runtime UI slider and require the physical state plus press journal to update before UI
   ownership stops semantic propagation;
2. release outside the slider so only its real pointer capture can own the UI route;
3. require the physical state to become released and the release event to remain journaled.

Unreal keeps physical `FKeyState` (`bDown`, event counts and raw value) separate from Enhanced Input
mapping/action consumption, including explicit held-key rebuild policy. Bevy's mouse/keyboard input
systems and Fyrox's OS-event handling likewise update engine physical state directly from every
qualified event before downstream semantic consumption. Zircon therefore requires physical fact
publication before the UI/gameplay ownership decision; moving the same early return elsewhere is
not an acceptable fix.

## Delivered Evidence

- `physical_input_ownership::ui_capture_release_commits_physical_state_before_propagation_stop`
  creates a real template-backed Runtime UI slider, releases outside its bounds, and checks the live
  session `InputManager`;
- the fixture and generated project remain under the test executable's approved D/E/F target tree;
- no compatibility path, alternate input state or test-only production hook was added.

## Status

| Item | Status | Evidence |
|---|---|---|
| Current source and owner review | complete | Runtime117 supersedes Runtime56; Runtime12 owns physical-first routing |
| Unreal/Bevy/Fyrox source comparison | complete | physical state is separate from mapping/UI consumption |
| Behavior RED source | complete | new real-session capture/release regression test |
| RED execution | pending | first sealed Button-consumption ticket was superseded by stricter real-capture source review; replacement manifest pending |
| Production fix | pending ownership transfer | dirty `session/events.rs` remains outside this exact scope |
| Performance claim | not applicable | correctness gate only; no optimization or benchmark claim |

The next implementation session may change production ordering only after the current
`events.rs` UI-metadata blob is transferred to its actual owner or an explicit combined owner is
approved. GREEN must preserve UI capture semantics while always committing qualified physical
press/release facts first.
