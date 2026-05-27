---
related_code:
  - zircon_runtime/src/core/framework/input/mod.rs
  - zircon_runtime/src/core/framework/input/button_input_state.rs
  - zircon_runtime/src/core/framework/input/input_button.rs
  - zircon_runtime/src/core/framework/input/input_event.rs
  - zircon_runtime/src/core/framework/input/mouse_wheel.rs
  - zircon_runtime/src/core/framework/input/file_drag_drop.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/src/core/framework/input/input_frame_snapshot.rs
  - zircon_runtime/src/core/framework/input/input_snapshot.rs
  - zircon_runtime/src/core/framework/input/gamepad.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/core/framework/input/touch.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_runtime/src/prelude.rs
implementation_files:
  - zircon_runtime/src/core/framework/input/button_input_state.rs
  - zircon_runtime/src/core/framework/input/input_button.rs
  - zircon_runtime/src/core/framework/input/input_event.rs
  - zircon_runtime/src/core/framework/input/mouse_wheel.rs
  - zircon_runtime/src/core/framework/input/file_drag_drop.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/src/core/framework/input/input_frame_snapshot.rs
  - zircon_runtime/src/core/framework/input/gamepad.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/core/framework/input/touch.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
plan_sources:
  - user: 2026-05-16 Bevy-style platform/window/winit/gilrs/input parity plan
  - user: 2026-05-16 continue Bevy-style platform/window/input stable prelude completion
  - chat: ZirconEngine Bevy 式 Platform / Window / Input / Gilrs 完成度计划
  - dev/bevy/crates/bevy_input/src/button_input.rs
  - dev/bevy/crates/bevy_input/src/keyboard.rs
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_input/src/touch.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/crates/bevy_gilrs/src/lib.rs
  - dev/bevy/crates/bevy_gilrs/src/gilrs_system.rs
  - dev/bevy/crates/bevy_gilrs/src/converter.rs
tests:
  - zircon_runtime/src/input/tests/input_manager.rs
  - zircon_runtime/src/input/tests/boundary.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/entry/tests/mod.rs
doc_type: module-detail
---

# Runtime Input State

## Purpose

`zircon_runtime::core::framework::input` owns neutral runtime input contracts. M2 expands the old cursor/button/wheel snapshot into a Bevy-style state model without removing the legacy `InputSnapshot` fields consumed by existing runtime and app tests.

The design follows Bevy's input split:

- `ButtonInputState<T>` mirrors Bevy `ButtonInput<T>` semantics with durable `pressed` state plus per-frame `just_pressed` and `just_released` transitions.
- `InputEvent` remains the append-only neutral event stream, now covering cursor position, cursor enter/leave, file drag/drop, window status, mouse motion, Bevy-style wheel x/y/unit events, keyboard, IME composition/delete requests, outgoing IME host requests, focus loss, touch, gamepad connection, gamepad button, and gamepad axis events.
- `InputFrameSnapshot` is the new full-frame state view for systems that need transitions, cursor-in-window state, current-frame file drag/drop and window status events, active touches, connected gamepads, processed gamepad axes, processed analog button values, current-frame gamepad rumble requests, IME preedit/commit/delete-surrounding/host-request state, precise wheel events, and motion accumulators.
- `InputSnapshot` remains the compatibility view: cursor, pressed buttons, and scalar wheel accumulator.

## Runtime Manager Behavior

`DefaultInputManager::begin_frame()` clears transient button transitions plus wheel and mouse-motion accumulators. It does not release currently pressed buttons. Callers that do not use `begin_frame()` keep the previous behavior where wheel accumulates until inspected.

Cursor events keep `cursor_position` and `cursor_inside_window` separate. `CursorMoved` updates position, while `CursorEntered` and `CursorLeft` only update whether the host cursor is inside the window. This mirrors Bevy's split between `CursorMoved`, `CursorEntered`, and `CursorLeft` messages in `dev/bevy/crates/bevy_window/src/event.rs`.

File drag/drop events are current-frame messages. `FileDragDropEvent::Hovered` and `Dropped` carry a UTF-8 path string, and `Cancelled` records that the host drag operation left or was cancelled. `begin_frame()` clears `InputFrameSnapshot::file_drag_drop_events`; it does not store a durable hovered-file set because the asset/UI layer owns any longer-lived import workflow.

Window status events are current-frame messages. `WindowStatusEvent` mirrors Bevy window-event style messages for moved, occluded, theme changed, backend scale-factor changed, logical scale-factor changed, close requested, and destroyed notifications. `begin_frame()` clears `InputFrameSnapshot::window_status_events`; longer-lived window policy remains owned by the host/runtime profile layer.

Button events update `ButtonInputState<InputButton>`. Repeated press events do not produce repeated `just_pressed`, and repeated release events do not produce repeated `just_released`.

Mouse-wheel events now follow Bevy's `MouseWheel` / `AccumulatedMouseScroll` split from `dev/bevy/crates/bevy_input/src/mouse.rs`: `MouseWheelEvent` carries `MouseScrollUnit::Line` or `Pixel` plus horizontal and vertical deltas, while `InputFrameSnapshot::mouse_wheel_accumulator`, `mouse_wheel_unit`, and `mouse_wheel_events` preserve current-frame precise scroll data. `wheel_accumulator` remains as a compatibility scalar; pixel events use Zircon's existing `LEGACY_PIXEL_SCROLL_SCALE` so older camera controls keep their previous vertical-scroll feel.

Keyboard events update both physical and logical button views when the host provides a logical key. `KeyboardFocusLost` releases only keyboard buttons, leaving mouse and gamepad state intact. This matches Bevy's distinction that keyboard state is window-focus dependent while gamepad state is not.

IME events update a separate composition state. `ImeEvent::Enabled` and `Disabled` track whether the host IME path is active. `Preedit` stores the current composing text plus optional byte cursor range, and an empty preedit clears composition. `Commit` clears preedit and records committed text in `InputFrameSnapshot::ime_commits` for the current frame. `DeleteSurrounding` records a before/after byte-count request in `InputFrameSnapshot::ime_delete_surrounding` for the current frame without mutating text directly. Outgoing `ImeHostRequest` values record enable/disable, cursor area, and surrounding-text requests in `InputFrameSnapshot::ime_host_requests`. `InputManager::drain_ime_host_requests()` is the one-shot host handoff path used by the dynamic runtime ABI; it clears only outgoing IME host requests and leaves the normal input event stream intact. `begin_frame()` also clears committed text, delete-surrounding requests, and outgoing host requests; it keeps IME enabled/preedit state.

Touch events keep a map of active touches. Started and moved phases update the active point; ended and cancelled phases remove it.

Gamepad connection events track connected gamepad ids. Disconnect clears that gamepad's axes, clears its analog button values, and releases its pressed gamepad buttons. This keeps stale physical-device state from surviving device removal while leaving keyboard focus-loss behavior window-scoped.

Gamepad button and axis values intentionally enter the runtime as raw host readings. `GamepadButtonAxisSettings` clamps analog button values into `[0.0, 1.0]`, applies a low zone of `0.05`, a high zone of `0.95`, and ignores processed changes below `0.01`. `GamepadButtonSettings` then applies Bevy-style digital hysteresis: a gamepad button presses at `0.75` and releases at `0.65`. `GamepadAxisSettings` applies an axis deadzone of `[-0.05, 0.05]`, livezone bounds of `[-1.0, 1.0]`, and a processed-value change threshold of `0.01`. These defaults mirror Bevy's split where the gilrs backend emits raw events and `bevy_input::gamepad::GamepadSettings` owns filtering.

Gamepad rumble requests are runtime-to-host requests. Runtime systems submit `InputEvent::GamepadRumbleRequest`; `InputFrameSnapshot::gamepad_rumble_requests` exposes the current-frame view, and `InputManager::drain_gamepad_rumble_requests()` is the one-shot handoff used by the dynamic runtime host-request ABI. The request intensity is clamped when converted to the stable ABI so invalid caller values cannot leak to a native backend.

## Compatibility

Existing callers can continue to call `snapshot()` and inspect `pressed_buttons`. New code should call `frame_snapshot()` when it needs Bevy-style transitions, mouse motion, touch, or gamepad state.

The stable runtime prelude now exposes the neutral input contracts, the default input manager, and the `InputModule` descriptor alongside the platform capability matrix. Runtime modules can therefore depend on Bevy-style input vocabulary without reaching through the concrete input module path.

## Event Log Harness

M6 adds a hardware-free log harness in `zircon_runtime/src/input/tests/input_manager.rs`: `input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad`. The test submits one mixed frame through `DefaultInputManager` only: window status, keyboard, cursor, raw mouse motion, mouse wheel, mouse button, touch, gamepad connection, gamepad button, and gamepad axis events all enter as `InputEvent` values.

The harness then builds its log from `InputFrameSnapshot`, not from the submitted fixture list. Window messages come from `window_status_events`; keyboard, mouse button, and gamepad button entries come from `ButtonInputState::just_pressed_inputs()`; mouse cursor, motion, and wheel entries come from the frame accumulators; touch entries come from `active_touches`; and gamepad connection/axis entries come from `connected_gamepads` and `gamepad_axes`. The same test drains `InputEventRecord` sequence numbers and checks they are contiguous from `1..=12`, so the example verifies both state reduction and append-only event recording on the normal runtime input manager path.

This is intentionally a test harness rather than a native desktop example binary. It gives CI the M6 example coverage without depending on a physical window, keyboard, mouse, touch device, or controller. Real winit/gilrs smoke testing remains optional because hardware availability cannot be a workspace gate.

## Runtime Preview Host Translation

M3 wires the runtime preview host through the existing `ZrRuntimeEventV1` ABI instead of importing `zircon_runtime::input` into `zircon_app`.

`zircon_app::entry::runtime_entry_app::ApplicationHandler` now translates winit window events as follows:

- `WindowEvent::PointerMoved` remains a pointer move for mouse-like sources, but touch sources become ABI touch moved events with the winit finger id.
- `WindowEvent::PointerEntered` and `PointerLeft` become ABI cursor boundary events, matching Bevy's `bevy_winit/src/state.rs` forwarding into `bevy_window::CursorEntered` and `CursorLeft`.
- Winit file drag events become ABI file drag/drop events. Bevy's checked-in `dev/bevy/crates/bevy_winit/src/state.rs` maps `WindowEvent::DroppedFile`, `HoveredFile`, and `HoveredFileCancelled` into `bevy_window::FileDragAndDrop`; Zircon's current winit 0.31 beta dependency names the same host capability `DragEntered`, `DragDropped`, and `DragLeft`. Zircon maps entered paths to `file_hovered`, dropped paths to `file_dropped`, and drag-left/cancelled to `file_drag_cancelled`.
- Winit window status events become ABI window status events. Bevy defines `WindowMoved`, `WindowOccluded`, `WindowThemeChanged`, `WindowBackendScaleFactorChanged`, `WindowScaleFactorChanged`, `WindowCloseRequested`, and `WindowDestroyed` in `dev/bevy/crates/bevy_window/src/event.rs`, then forwards host events from `dev/bevy/crates/bevy_winit/src/state.rs`. Zircon's local winit 0.31 beta names those host inputs `Moved`, `Occluded`, `ThemeChanged`, `ScaleFactorChanged`, `CloseRequested`, and `Destroyed`, and maps them to `ZrRuntimeEventV1::window_moved`, `window_occluded`, `window_theme_changed`, `window_backend_scale_factor_changed`, `window_scale_factor_changed`, `window_close_requested`, and `window_destroyed`.
- `WindowEvent::PointerButton` remains a mouse button for mouse-like sources, but touch button press/release becomes ABI touch started/ended.
- `WindowEvent::PointerLeft` for touch becomes ABI touch cancelled so runtime state can clear active touches when the platform cancels tracking without a release.
- `WindowEvent::KeyboardInput` becomes ABI keyboard pressed/released with a deterministic physical key code and optional text payload.
- `WindowEvent::MouseWheel` becomes ABI mouse-wheel data with x/y deltas and a Line/Pixel unit. This mirrors Bevy's `dev/bevy/crates/bevy_winit/src/state.rs` mapping of winit `MouseScrollDelta::LineDelta(x, y)` and `PixelDelta(p)` into `bevy_input::mouse::MouseWheel`.
- `WindowEvent::Ime` becomes ABI IME enabled/disabled/preedit/commit/delete-surrounding events. This follows Bevy's `bevy_winit` path, which forwards winit IME events into `bevy_window::Ime` messages instead of folding composition into keyboard input. Zircon keeps winit `DeleteSurrounding` as a neutral runtime event even though Bevy's checked-in window event enum does not currently expose that variant.
- `WindowEvent::Focused(false)` becomes ABI lifecycle background, which the runtime uses to clear keyboard state.
- `DeviceEvent::PointerMotion` becomes ABI raw mouse motion. This mirrors Bevy's `bevy_winit` path, where winit device motion is forwarded as `bevy_input::mouse::MouseMotion` instead of being treated as cursor position. Bevy's checked-in source still names the winit variant `DeviceEvent::MouseMotion`; Zircon's current winit 0.31 beta dependency exposes the same raw delta as `PointerMotion`.

`zircon_runtime::dynamic_api::session` consumes those ABI events into the richer runtime input state:

- Touch ABI events submit `InputEvent::Touch` and still drive the preview camera controller directly, without pretending touch is a mouse button in `InputFrameSnapshot`.
- Cursor entered/left ABI events submit `InputEvent::CursorEntered` and `InputEvent::CursorLeft`, updating `InputFrameSnapshot::cursor_inside_window` without changing the last known cursor position.
- File drag/drop ABI events submit `InputEvent::FileDragDrop`, appending `FileDragDropEvent` values to `InputFrameSnapshot::file_drag_drop_events` for the current frame.
- Window status ABI events submit `InputEvent::WindowStatus`, appending `WindowStatusEvent` values to `InputFrameSnapshot::window_status_events` for the current frame.
- Keyboard ABI events submit `InputEvent::KeyboardInput` so `DefaultInputManager` owns physical key state, text payload, and frame transitions. Text is not used as a logical key identity because text is usually absent on release events.
- IME ABI events submit `InputEvent::Ime` so composition and delete-surrounding requests are available to text widgets without being confused with physical key presses.
- Outgoing IME host requests are drained through optional `ZrRuntimeApiV1::drain_host_requests` as a JSON `ZrRuntimeHostRequestBatchV1`. `zircon_app::entry::runtime_library::RuntimeSession` decodes the batch, and the native preview host applies enable, disable, cursor-area, and surrounding-text requests to winit `Window::request_ime_update`. This follows Bevy's window-owned `ime_enabled` / `ime_position` configuration surface in `dev/bevy/crates/bevy_window/src/window.rs`, while using the richer local winit 0.31 `ImeRequest`, `ImeCapabilities`, `ImeRequestData`, and `ImeSurroundingText` API for native host application.
- Outgoing gamepad rumble requests are drained through the same optional host-request API as `ZrRuntimeHostRequestV1::GamepadRumble`. On desktop `gamepad-gilrs`, the native preview host now maps requests to gilrs force-feedback effects (`Strong`/`Weak`) and tracks active effect lifetimes so `Stop` requests, gamepad disconnects, and app shutdown clear playback handles deterministically. Missing gamepads, disconnected pads, unsupported force-feedback capability, and gilrs force-feedback channel failures are reported as host warnings; the ABI and runtime queue contract remain unchanged.
- Background, suspended, and low-memory lifecycle states submit `InputEvent::KeyboardFocusLost`.
- Mouse-motion ABI events submit `InputEvent::MouseMotion`, which is accumulated into `InputFrameSnapshot::mouse_motion_accumulator` and reset by `begin_frame()`. This follows Bevy's split between raw `MouseMotion` events and frame-local `AccumulatedMouseMotion`.
- Mouse-wheel ABI events with a Line/Pixel unit submit `InputEvent::MouseWheel`; legacy unit-less ABI events still submit `WheelScrolled` so older hosts keep working. The dynamic session validates wheel x/y values as finite before appending precise current-frame wheel state.

## Raw Mouse Motion

M5 adds the first device-event input path. The runtime preview app now implements `ApplicationHandler::device_event` and forwards only raw pointer motion through `ZrRuntimeEventV1::mouse_motion`. Other device events remain ignored until the runtime has an explicit neutral contract for them.

This is intentionally separate from `WindowEvent::PointerMoved`: pointer movement reports logical cursor position, while device mouse motion reports raw physical delta. The distinction is the same one documented in `dev/bevy/crates/bevy_input/src/mouse.rs`, and it keeps future pointer-lock/high-precision camera controls from depending on cursor coordinates.

## Mouse Wheel

M14 adds Bevy-style wheel unit fidelity. Bevy defines `MouseScrollUnit`, `MouseWheel { unit, x, y }`, and `AccumulatedMouseScroll { unit, delta }` in `dev/bevy/crates/bevy_input/src/mouse.rs`, then forwards winit `LineDelta(x, y)` and `PixelDelta(p)` without dropping the horizontal axis in `dev/bevy/crates/bevy_winit/src/state.rs`.

Zircon mirrors that in the runtime input contract with `MouseScrollUnit` and `MouseWheelEvent`. The ABI keeps `ZrRuntimeEventV1::mouse_wheel(delta)` as a vertical line-scroll compatibility constructor, and adds `mouse_wheel_delta(unit, x, y)` for full-fidelity host input. `zircon_app` now uses the full-fidelity constructor for both line and pixel scroll paths. Runtime systems that still read `InputSnapshot::wheel_accumulator` get the same scalar view, while systems that read `InputFrameSnapshot` can inspect raw x/y deltas and the last frame unit.

## Cursor Boundary

M8 adds the Bevy-style cursor boundary path. Bevy registers `CursorEntered` and `CursorLeft` messages in `dev/bevy/crates/bevy_window/src/lib.rs`, defines the event payloads in `dev/bevy/crates/bevy_window/src/event.rs`, and Zircon forwards current winit `WindowEvent::PointerEntered` / `PointerLeft` into the same cursor-boundary ABI concepts. Zircon mirrors that as a neutral bool in `InputFrameSnapshot::cursor_inside_window` plus two ABI constructors, `ZrRuntimeEventV1::cursor_entered` and `ZrRuntimeEventV1::cursor_left`.

## File Drag And Drop

M9 adds the Bevy-style file drag/drop path. Bevy defines `bevy_window::FileDragAndDrop` in `dev/bevy/crates/bevy_window/src/event.rs` with dropped, hovered, and cancelled variants, registers it as a window event, and forwards host events from `dev/bevy/crates/bevy_winit/src/state.rs`. Zircon mirrors that with `FileDragDropEvent` in the runtime input contract and `ZrRuntimeEventV1::file_hovered`, `file_dropped`, and `file_drag_cancelled` in the stable runtime ABI.

The runtime preview host intentionally adapts to the local winit 0.31 beta API: `DragEntered { paths, .. }` emits one hovered event per path, `DragDropped { paths, .. }` emits one dropped event per path, and `DragLeft { .. }` emits a cancel event. `DragMoved` currently has only position in this winit version, so it is ignored until Zircon needs a drag-position contract. Runtime systems read the current-frame events from `InputFrameSnapshot::file_drag_drop_events`.

## Window Status Events

M11 adds the first non-input window status event family. Bevy keeps these in `bevy_window` instead of `bevy_input`: `WindowCloseRequested`, `WindowDestroyed`, `WindowOccluded`, `WindowMoved`, and `WindowThemeChanged` are defined in `dev/bevy/crates/bevy_window/src/event.rs` and forwarded from `dev/bevy/crates/bevy_winit/src/state.rs`.

Zircon mirrors the same host/runtime split with `WindowStatusEvent` and ABI constructors on `ZrRuntimeEventV1`. The runtime preview host forwards local winit `CloseRequested`, `Destroyed`, `Moved`, `Occluded`, and `ThemeChanged` into the dynamic runtime session, which reduces them into current-frame `InputFrameSnapshot::window_status_events`.

M12 adds Bevy's dedicated scale-factor event split. Bevy declares both `WindowBackendScaleFactorChanged` and `WindowScaleFactorChanged` in `dev/bevy/crates/bevy_window/src/event.rs`, and `dev/bevy/crates/bevy_winit/src/state.rs::react_to_scale_factor_change` always emits the backend notification while emitting the logical notification only when Bevy's window-resolution override policy allows it. Zircon does not yet expose a runtime scale-factor override setting, so the preview host forwards local winit `WindowEvent::ScaleFactorChanged` to both ABI constructors. The dynamic session validates that the scale factor is finite and positive before appending `WindowStatusEvent::BackendScaleFactorChanged` and `WindowStatusEvent::ScaleFactorChanged`. `ZrRuntimeViewportMetricsV1` and resize events still own actual surface size and framebuffer metrics; these status events are for systems that need to observe scale changes separately from resize state.

## IME Composition

M6 adds the Bevy-style IME window message path. Bevy defines `bevy_window::Ime` in `dev/bevy/crates/bevy_window/src/event.rs` with `Preedit`, `Commit`, `Enabled`, and `Disabled`, and `dev/bevy/crates/bevy_winit/src/state.rs` maps `WindowEvent::Ime` into those messages. Zircon mirrors that split with `ImeEvent` and `ImePreedit` in the runtime input contracts, while the preview host translates through `ZrRuntimeEventV1` instead of importing runtime implementation types.

M7 adds Zircon's explicit `ImeEvent::DeleteSurrounding` path for winit `Ime::DeleteSurrounding`. The app ABI uses `ZrRuntimeEventV1::ime_delete_surrounding` with `key_code` as `before_bytes` and `scan_code` as `after_bytes`, then `zircon_runtime::dynamic_api::session` reduces it into the runtime input manager. This is intentionally a request visible in the current frame, not a text mutation, because the text buffer owner lives in the UI/text-editing layer.

M13 adds the neutral outgoing host-request side for native IME control. Bevy keeps IME activation and candidate placement on `Window::ime_enabled` and `Window::ime_position` in `dev/bevy/crates/bevy_window/src/window.rs`, while winit 0.31 exposes richer request data for cursor area and surrounding text. Zircon mirrors that direction with `ImeHostRequest`: enable, disable, cursor area, and UTF-8 surrounding text are current-frame requests stored in `InputFrameSnapshot::ime_host_requests`.

The runtime ABI carries these requests on the IME event family with `ime_request_enable`, `ime_request_disable`, `ime_cursor_area`, and `ime_surrounding_text`. `zircon_runtime::dynamic_api::session` validates cursor areas as finite positive rectangles and validates surrounding-text cursor/anchor offsets as UTF-8 byte boundaries before submitting `InputEvent::ImeHostRequest`. The UI dispatch contract still owns higher-level widget intent as `UiInputMethodRequest`; this runtime input lane is the lower transport contract that the native host consumes through winit's IME request API.

M15 closes the native desktop preview loop with optional `ZrRuntimeApiV1::drain_host_requests`. The runtime drains outgoing IME requests into `ZrRuntimeHostRequestBatchV1` JSON, owns the returned byte buffer with the same free-callback pattern as frame/profile outputs, and leaves normal input events untouched. `zircon_app::entry::runtime_library::RuntimeSession` treats the function as optional for older dynamic runtimes, decodes the batch when present, validates the ABI version, and frees the buffer. `RuntimeEntryApp::about_to_wait` applies the drained requests to the current winit window using `ImeRequest::Enable`, `ImeRequest::Update`, `ImeRequest::Disable`, `ImeCapabilities`, `ImeRequestData`, and `ImeSurroundingText`. This keeps Zircon aligned with Bevy's window-owned IME policy while using the richer local winit 0.31 API shape for cursor-area and surrounding-text updates.

## Gilrs Runtime Preview Host Backend

M4 adds the first native gamepad backend in the runtime preview host. This follows Bevy's split where `bevy_gilrs::GilrsPlugin` owns gilrs startup/polling, then feeds neutral Bevy input events through `bevy_input::gamepad`.

`zircon_app::entry::runtime_entry_app::gamepad` is compiled behind `gamepad-gilrs`. It initializes `gilrs::Gilrs` with default filters disabled and manual state updates, matching Bevy's `GilrsBuilder::with_default_filters(false).set_update_state(false)` shape in `dev/bevy/crates/bevy_gilrs/src/lib.rs`. Existing connected gamepads are announced before polling, matching Bevy's startup connection pass in `gilrs_event_startup_system`.

Each winit wait cycle polls gilrs and translates events through the ABI instead of importing runtime input state into the app:

- `Connected` and `Disconnected` become `ZR_RUNTIME_EVENT_KIND_GAMEPAD_CONNECTION_V1`, carrying gamepad id, name, vendor id, and product id.
- `ButtonPressed`, `ButtonRepeated`, `ButtonReleased`, and `ButtonChanged` become `ZR_RUNTIME_EVENT_KIND_GAMEPAD_BUTTON_V1`, carrying stable Zircon button codes and the analog value. `ButtonChanged` forwards the raw analog value without an app-side `value >= 0.5` threshold; runtime input state applies the Bevy-style button axis and digital hysteresis settings described above.
- `AxisChanged` becomes `ZR_RUNTIME_EVENT_KIND_GAMEPAD_AXIS_V1`, carrying stable Zircon axis codes and value.

`zircon_runtime::dynamic_api::session` reduces those ABI events into `InputEvent::GamepadConnection`, `InputEvent::GamepadButton`, and `InputEvent::GamepadAxis`. The runtime keeps Bevy-style durable gamepad state in `InputFrameSnapshot`: connected gamepads, pressed gamepad buttons, per-frame transitions, processed analog button values, and processed latest axis values. Disconnect still clears that gamepad's axes, analog button values, and pressed buttons.

Current intentional gaps are browser Gamepad API support, additional non-mouse device events, and editor/native host convergence. Browser gamepad must remain a separate backend instead of being treated as a gilrs alias.
