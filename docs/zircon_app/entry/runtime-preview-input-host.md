---
related_code:
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/window_events/dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/cursor.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/motion.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/button.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/wheel.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/device.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/event.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/payload.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/hovered.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/dropped.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/cancelled.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/routing.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_runtime/src/core/framework/input/input_event.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/tests/input_manager.rs
implementation_files:
  - zircon_app/src/entry/runtime_entry_app/window_events/dispatch.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/cursor.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/motion.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/button.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/wheel.rs
  - zircon_app/src/entry/runtime_entry_app/pointer_input/device.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/event.rs
  - zircon_app/src/entry/runtime_entry_app/keyboard_input/payload.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/hovered.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/dropped.rs
  - zircon_app/src/entry/runtime_entry_app/file_drag_drop/cancelled.rs
  - zircon_app/src/entry/runtime_entry_app/ime_input/routing.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_input/src/keyboard.rs
  - dev/bevy/crates/bevy_input/src/touch.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/bevy/crates/bevy_gilrs/src/gilrs_system.rs
tests:
  - zircon_runtime/src/input/tests/input_manager.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/mod.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/pointer.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/keyboard.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/file_drag_drop.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/ime.rs
  - zircon_app/src/entry/tests/runtime_entry_input_guards/protocol.rs
  - zircon_app/src/entry/tests/runtime_entry_device_guards/dispatch.rs
  - zircon_app/src/entry/tests/runtime_entry_source_guards/window_events.rs
doc_type: module-detail
---

# Runtime Preview Input Host

## Purpose

`zircon_app::entry::runtime_entry_app` owns the native host side of the runtime preview input lane. It converts concrete winit and optional gilrs observations into the stable `ZrRuntimeEventV1` ABI, then the dynamic runtime session reduces those ABI events into `zircon_runtime::input::DefaultInputManager`.

The app crate must not own durable input state. It only translates host events, submits them to the loaded runtime session, drains runtime-to-host requests such as IME updates and gamepad rumble, and keeps the event-loop policy aligned with the selected runtime profile.

## Host Event Families

Window events are dispatched from `window_events/dispatch.rs` into focused child modules. Pointer and mouse routing is split by responsibility: cursor boundary and touch cancellation live in `pointer_input/cursor.rs`, logical pointer and touch movement in `motion.rs`, mouse/touch button forwarding in `button.rs`, line/pixel wheel forwarding in `wheel.rs`, and raw device motion in `device.rs`. This mirrors Bevy's split where winit owns collection and `bevy_input` / `bevy_window` own neutral event vocabulary.

Keyboard forwarding is isolated in `keyboard_input/`. The event helper converts the physical key, action, repeat flag, and optional text payload into ABI keyboard events, while the payload helper owns byte-slice construction. IME routing is separate under `ime_input/` so composition, lifecycle, commit, and delete-surrounding events do not masquerade as ordinary keyboard input.

File drag/drop forwarding is isolated under `file_drag_drop/`. Hovered paths, dropped paths, and cancellation each have a focused helper, preserving the boundary that asset import behavior belongs to later asset systems instead of the app entry dispatcher.

The optional desktop gamepad backend is isolated under `gamepad/`. The host module initializes gilrs, polling drains connection/button/axis events, the events module builds stable ABI values, codes owns button/axis mapping, and rumble owns gilrs force-feedback effect creation plus cleanup. Desktop gilrs remains a concrete host backend; the neutral gamepad state belongs to `zircon_runtime`.

## M6 Log Harness

The M6 example is the runtime test `input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad`. It is documented here because it is the hardware-free acceptance companion to the native host path. The app entry modules prove that real winit/gilrs events are translated into the ABI families; the harness proves that the corresponding runtime event families can be logged from one `InputFrameSnapshot` after entering the same `DefaultInputManager` path.

The harness covers window status, keyboard, cursor, raw mouse motion, mouse wheel, mouse button, touch, gamepad connection, gamepad button, and gamepad axis in one frame. It verifies the reduced snapshot log and the append-only event-record sequence. This keeps the example deterministic in CI while still mapping directly to the host families above.

## Acceptance Boundary

M6 acceptance requires both sides of the split to remain true:

- `zircon_app` continues to translate host events through `ZrRuntimeEventV1` and optional host-request batches.
- `zircon_runtime` continues to reduce those ABI events into `InputEvent`, `InputFrameSnapshot`, button transitions, touch state, gamepad state, and event records.
- Platform/profile validation continues to prove that default client/editor host profiles include the window/input/gilrs capability surface while server/headless profiles stay explicit and dependency-light.

Real controller, IME, and window smoke tests can still be useful manual checks, but they are not required for CI because the M6 harness and platform matrix checks avoid hardware dependency.
