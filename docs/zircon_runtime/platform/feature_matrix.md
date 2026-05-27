---
related_code:
  - zircon_runtime/src/platform/mod.rs
  - zircon_runtime/src/platform/config.rs
  - zircon_runtime/src/platform/capability/mod.rs
  - zircon_runtime/src/platform/capability/status.rs
  - zircon_runtime/src/platform/capability/backends.rs
  - zircon_runtime/src/platform/capability/report.rs
  - zircon_runtime/src/platform/capability/matrix/mod.rs
  - zircon_runtime/src/platform/capability/matrix/policy.rs
  - zircon_runtime/src/platform/capability/matrix/linux.rs
  - zircon_runtime/src/platform/capability/matrix/input.rs
  - zircon_runtime/src/platform/capability/matrix/window.rs
  - zircon_runtime/src/platform/capability/matrix/gamepad.rs
  - zircon_runtime/src/platform/feature_selection.rs
  - zircon_runtime/src/platform/target.rs
  - zircon_runtime/src/platform/service_types.rs
  - zircon_runtime/src/core/framework/window/mod.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/input/mod.rs
  - zircon_runtime/src/core/framework/input/ime.rs
  - zircon_runtime/src/core/framework/input/input_frame_snapshot.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_runtime_interface/src/runtime_api.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/mod.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/mod.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/control_flow.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/mod.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/host.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/rumble.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/events.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad/codes.rs
  - zircon_app/src/entry/runtime_entry_app/host_requests/routing.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
implementation_files:
  - zircon_runtime/src/platform/capability/mod.rs
  - zircon_runtime/src/platform/capability/status.rs
  - zircon_runtime/src/platform/capability/backends.rs
  - zircon_runtime/src/platform/capability/report.rs
  - zircon_runtime/src/platform/capability/matrix/mod.rs
  - zircon_runtime/src/platform/capability/matrix/policy.rs
  - zircon_runtime/src/platform/capability/matrix/linux.rs
  - zircon_runtime/src/platform/capability/matrix/input.rs
  - zircon_runtime/src/platform/capability/matrix/window.rs
  - zircon_runtime/src/platform/capability/matrix/gamepad.rs
  - zircon_runtime/src/platform/feature_selection.rs
  - zircon_runtime/src/platform/target.rs
  - zircon_runtime/src/platform/config.rs
  - zircon_runtime/src/platform/service_types.rs
  - zircon_runtime/src/core/framework/window/mod.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/input/runtime/default_input_manager.rs
  - zircon_runtime/src/input/runtime/input_state.rs
  - zircon_runtime/src/dynamic_api/session.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/runtime_library/loaded_runtime.rs
  - zircon_app/src/entry/runtime_library/runtime_session.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/mod.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler/hooks.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/mod.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy/control_flow.rs
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
  - user: 2026-05-16 continue Bevy-style app bootstrap platform config completion
  - chat: ZirconEngine Bevy 式 Platform / Window / Input / Gilrs 完成度计划
  - dev/bevy/Cargo.toml
  - dev/bevy/crates/bevy_input/src/lib.rs
  - dev/bevy/crates/bevy_window/src/lib.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/bevy/crates/bevy_winit/src/lib.rs
  - dev/bevy/crates/bevy_winit/src/state.rs
  - dev/bevy/crates/bevy_winit/src/system.rs
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/docs/cargo_features.md
  - dev/bevy/crates/bevy_input/src/keyboard.rs
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_input/src/touch.rs
  - dev/bevy/crates/bevy_input/src/gestures.rs
  - dev/bevy/crates/bevy_input/src/lib.rs
  - dev/bevy/crates/bevy_winit/src/converters.rs
  - dev/bevy/crates/bevy_input/src/gamepad.rs
  - dev/bevy/crates/bevy_gilrs/src/lib.rs
  - dev/bevy/crates/bevy_gilrs/src/gilrs_system.rs
  - dev/bevy/crates/bevy_gilrs/src/rumble.rs
tests:
  - zircon_runtime/src/platform/tests/mod.rs
  - zircon_runtime/src/platform/tests/structure.rs
  - zircon_runtime/src/platform/tests/app_feature_manifest.rs
  - zircon_runtime/src/platform/tests/backend_tokens.rs
  - zircon_runtime/src/platform/tests/desktop_defaults.rs
  - zircon_runtime/src/platform/tests/diagnostic_status_consistency.rs
  - zircon_runtime/src/platform/tests/diagnostic_metadata.rs
  - zircon_runtime/src/platform/tests/event_loop_policy.rs
  - zircon_runtime/src/platform/tests/feature_gate_propagation.rs
  - zircon_runtime/src/platform/tests/feature_selection.rs
  - zircon_runtime/src/platform/tests/headless.rs
  - zircon_runtime/src/platform/tests/headless_synthetic_input.rs
  - zircon_runtime/src/platform/tests/linux.rs
  - zircon_runtime/src/platform/tests/matrix_cross_product.rs
  - zircon_runtime/src/platform/tests/status_semantics.rs
  - zircon_runtime/src/platform/tests/target_topology.rs
  - zircon_runtime/src/platform/tests/target_modes.rs
  - zircon_runtime/src/platform/tests/diagnostic_keys.rs
  - zircon_runtime/src/platform/tests/diagnostics.rs
  - zircon_runtime/src/platform/tests/cross_target.rs
  - zircon_runtime/src/platform/tests/gestures.rs
  - zircon_runtime/src/platform/tests/gamepad.rs
  - zircon_runtime/src/platform/tests/feature_manifest.rs
  - zircon_runtime/src/input/tests/input_manager.rs
  - zircon_runtime/src/input/tests/boundary.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/tests/prelude.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_runtime/src/tests/plugin_extensions/export_build_plan.rs
  - .github/workflows/ci.yml
doc_type: module-detail
---

# Platform Feature Matrix

## Purpose

`zircon_runtime::platform` now owns a Bevy-style declaration layer for platform, window, event-loop, input, and gamepad capability. This layer is descriptive: it tells callers and tests which host capability is available, feature-disabled, or not implemented for a target. Concrete OS polling and window creation stay in `zircon_app`.

The stable runtime prelude exports the matrix, target/backend enums, platform module descriptor, and input module/event contracts. App and module authors can use a default platform declaration without importing the lower platform and input modules piecemeal.

The matrix follows the same split Bevy uses:

- feature groups declare coarse platform defaults, as Bevy does with `default_platform` in `dev/bevy/Cargo.toml`;
- input sources are independent gates, matching `bevy_input` features for mouse, keyboard, gamepad, touch, and gestures;
- window/event-loop capability is separate from the app host backend, matching Bevy's split between `bevy_window` and `bevy_winit`;
- `gilrs` is modeled as the desktop gamepad backend, matching `bevy_gilrs`, without leaking gilrs types into runtime contracts.

## Runtime Shape

`PlatformFeatureSelection` is the feature snapshot. `from_compiled_features()` reads the active Cargo feature set, while `bevy_default_platform()` and `headless()` give tests and profile builders stable policy fixtures.

`PlatformTarget` is the runtime-facing target list: `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless`. It mirrors the export contract platform names where they overlap but lives in the platform module so plugin export policy does not become the owner of runtime host capability.

`PlatformCapabilityMatrix` produces a `PlatformCapabilityReport` for a target and `RuntimeTargetMode`. The report includes:

- window backend: `Winit`, `BrowserCanvas`, or `Headless`;
- monitor inventory backend: desktop/mobile winit monitor handles, future browser screen details, or unavailable;
- window event backend: desktop/mobile winit window events, future browser window events, or unavailable;
- window lifecycle backend: desktop/mobile winit close/focus/occlusion/theme/move/scale-factor events, future browser lifecycle events, or unavailable;
- window metrics backend: desktop/mobile winit resize and scale-factor events, future browser resize observer/canvas metrics, or unavailable;
- IME backend: native desktop winit IME, future mobile/browser IME host paths, or unavailable;
- keyboard event backend: desktop/mobile winit keyboard events and keyboard-focus-loss cleanup, future browser keyboard events, or unavailable;
- cursor boundary backend: desktop/mobile winit pointer-enter/leave events, future browser pointer events, or unavailable;
- cursor options backend: future desktop winit cursor visibility/grab/hit-test host requests, future browser cursor options, or unavailable;
- mouse button backend: desktop/mobile winit mouse button events, future browser pointer button events, or unavailable;
- mouse wheel backend: desktop/mobile winit wheel events with line/pixel units, future browser wheel events, or unavailable;
- touch event backend: desktop/mobile winit touch events with stable touch ids and phases, future browser touch events, or unavailable;
- gesture event backend: `input-gestures` feature gate, future macOS/iOS winit gesture events, future browser gesture events, or unavailable;
- pointer position backend: desktop/mobile winit pointer/cursor moved events, future browser pointer events, or unavailable;
- raw mouse motion backend: desktop winit device events, future browser pointer-lock motion, or unavailable;
- event-loop policy: `Game`, `DesktopApp`, `Mobile`, `Continuous`, or `Headless`;
- mouse, keyboard, touch, and gesture input backend declarations;
- gamepad backend declaration: desktop `Gilrs` in the default platform profile, browser `BrowserGamepadApi`, feature-disabled, or unavailable;
- gamepad event backend declaration: desktop `GilrsEventPolling`, browser `BrowserGamepadApiPolling`, feature-disabled, or unavailable;
- gamepad rumble backend declaration: desktop gilrs force-feedback, future browser haptics, feature-disabled, or unavailable;
- file drag/drop backend declaration: desktop `WinitWindowEvents`, future `BrowserDragEvents`, or unavailable;
- Linux protocol declarations for X11 and Wayland.

The capability implementation is folder-backed so the platform root stays structural and each declaration family has one owner. `capability/mod.rs` only wires child modules and curated exports; `status.rs` owns `CapabilityStatus`; `backends.rs` owns backend enum declarations and diagnostic keys; `report.rs` owns `PlatformCapabilityReport` diagnostic formatting; and `capability/matrix/` owns report construction split by policy, Linux protocols, input event families, window host families, and gamepad host families.

`report(...)` derives the default event-loop policy from topology: client desktop uses `Game`, editor host uses `DesktopApp`, mobile uses `Mobile`, and server/headless uses `Headless`. `report_with_event_loop_policy(...)` is the explicit opt-in path for Bevy-style update-policy selection, including `EventLoopPolicy::Continuous`. Server runtime and explicit headless targets still report `Headless` even when an explicit continuous policy is requested, because the topology has no active window event loop to poll.

`RuntimeTargetMode` remains visible through the same diagnostics as `platform.target_mode=client_runtime`, `server_runtime`, or `editor_host`. Client/editor modes may select windowed Bevy-style policies (`Game`, `DesktopApp`, `Mobile`, or explicit `Continuous`) when the target has a window backend. Server runtime is topology-authoritative and stays headless across host targets.

`PlatformConfig` keeps the existing `enabled` flag and adds target, runtime mode, and feature snapshot fields. `PLATFORM_CONFIG_KEY` is the runtime config-store key used by `zircon_app` bootstrap. `PlatformManager::capability_report()` is a thin access surface over that config. The primary window descriptor is stored separately under `PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY` by the app entry layer, so capability diagnostics can say which backend is available while window diagnostics say which primary-window policy was selected.

`BuiltinEngineEntry` stores a serialized `PlatformConfig` before module activation. Runtime/editor entries use the current host target and compiled feature snapshot; headless entries use `PlatformTarget::Headless` plus `PlatformFeatureSelection::headless()`. `RuntimeProfileId::Minimal` still stores the config for diagnostics, but marks it disabled because `MinimalPlugins` does not install `PlatformModule`.

`PlatformCapabilityReport::diagnostic_lines()` and `PlatformConfig::diagnostic_lines()` expose the same matrix through stable `platform.*` lines. The format is intentionally simple: supported entries use `supported:<backend>`, disabled feature gates use `feature_disabled:<feature>`, and not-yet-implemented host paths use `unavailable:<reason>`. `EntryModuleSelectionReport::diagnostic_lines()` now appends those lines after the entry target mode, so `EntryRunner::module_selection_diagnostics(...)` reports both the selected plugin/module group and the platform capability surface selected for that profile.

This mirrors Bevy's public diagnostic need without copying its internals: Bevy's `DefaultPlugins` are feature-gated and documented as user-controllable by disabling default features in `dev/bevy/crates/bevy_internal/src/default_plugins.rs`; `dev/bevy/docs/cargo_features.md` documents `default_platform`, `bevy_winit`, `bevy_gilrs`, `gamepad`, `x11`, `wayland`, and `web`; and `dev/bevy/crates/bevy_winit/src/winit_config.rs` separates game, desktop-app, mobile, and continuous update policies. Zircon keeps those decisions visible as capability diagnostics instead of relying on implicit Cargo feature knowledge.

The runtime-preview winit host maps this runtime-facing policy to winit `ControlFlow` in `zircon_app/src/entry/runtime_entry_app/event_loop_policy/control_flow.rs`: `Game` and `Continuous` use `ControlFlow::Poll`, while `DesktopApp`, `Mobile`, and `Headless` use `ControlFlow::Wait`. That mapping is app-host behavior and does not move concrete event-loop ownership into `zircon_runtime`.

## Target Policy

Desktop targets use `Winit` for windowing when `platform-window` and `platform-winit` are enabled. Linux separately declares `platform-x11` and `platform-wayland` so the engine can later validate X11/Wayland support independently instead of treating Linux as one opaque desktop path.

The platform matrix reports backend availability, while `zircon_runtime::core::framework::window::WindowDescriptor` now owns the neutral primary-window defaults that a backend can consume: primary-window handle, title, present mode, window mode, position, resolution, resize constraints, resizable/decorated/visible/focused flags, and physical/logical scale-factor conversion. This mirrors Bevy's `bevy_window::Window` and `WindowPlugin::primary_window` split from `bevy_winit` host creation. The descriptor is additive, is persisted through app bootstrap config, and is consumed by the runtime-preview app host for initial winit window attributes.

Monitor inventory is now declared beside the window backend as `platform.monitor_inventory`. Desktop and mobile winit targets report `supported:winit_monitor_handles`, because the concrete app host can query `ActiveEventLoop::primary_monitor()` and `available_monitors()` when applying `WindowMonitorSelection` and `WindowVideoModeSelection`. Browser targets currently report `unavailable:browser monitor inventory host backend is not implemented yet`, preserving room for a future Screen Details or browser-specific host path without pretending it is equivalent to winit. Headless/server targets report unavailable because they have no physical monitor host. This follows Bevy's split: `dev/bevy/crates/bevy_window/src/window.rs` defines `MonitorSelection` and `VideoModeSelection`, while `dev/bevy/crates/bevy_winit/src/winit_windows.rs` and `dev/bevy/crates/bevy_winit/src/system.rs` use winit monitor handles at the backend boundary.

Window event capability is declared as `platform.window_events`. Desktop and mobile winit targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards move, occlusion, theme, close, destroyed, focus, and scale-factor events. Browser targets remain `unavailable:browser window event host backend is not implemented yet` until a browser host maps DOM/canvas lifecycle into the same runtime ABI. Headless/server targets report unavailable because they have no concrete OS window. This follows Bevy's split between the event vocabulary in `dev/bevy/crates/bevy_window/src/event.rs` and concrete winit event collection in `dev/bevy/crates/bevy_winit/src/state.rs`.

Window lifecycle capability is declared as `platform.window_lifecycle`. Desktop and mobile winit targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards focus, occlusion, move, theme, scale-factor, close-request, and destroyed events into runtime `WindowStatusEvent` values. Browser targets remain `unavailable:browser window lifecycle host backend is not implemented yet` until a browser host maps page/canvas lifecycle and resize state into the same runtime ABI. Headless/server targets report unavailable because they have no OS window lifecycle to observe. This follows Bevy's `WindowFocused`, `WindowOccluded`, `WindowMoved`, `WindowThemeChanged`, `WindowBackendScaleFactorChanged`, `WindowScaleFactorChanged`, `WindowCloseRequested`, and `WindowDestroyed` vocabulary in `dev/bevy/crates/bevy_window/src/event.rs` plus the concrete forwarding path in `dev/bevy/crates/bevy_winit/src/state.rs`.

Window metrics capability is declared as `platform.window_metrics`. Desktop and mobile winit targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards local winit 0.31 beta `WindowEvent::Resized` and `ScaleFactorChanged` into `ZrRuntimeEventV1::viewport_metrics` plus scale-factor status events, then refreshes runtime viewport size and scale state. Browser targets remain `unavailable:browser window metrics host backend is not implemented yet` until a browser host maps canvas resize observation and device-pixel-ratio changes into the same runtime ABI. Headless/server targets report unavailable because they have no OS window or canvas metrics host. This follows Bevy's `WindowResized`, `WindowScaleFactorChanged`, and `WindowBackendScaleFactorChanged` vocabulary in `dev/bevy/crates/bevy_window/src/event.rs` plus the concrete `WindowEvent::Resized` / `ScaleFactorChanged` forwarding path in `dev/bevy/crates/bevy_winit/src/state.rs`.

IME capability is declared as `platform.ime`. Desktop winit targets report `supported:winit_ime`, matching the native runtime-preview host path that forwards composition/delete-surrounding events and applies outgoing `ImeHostRequest` values. Mobile winit targets currently report `unavailable:mobile ime host backend is not implemented yet`, because Zircon has not implemented a mobile activity text-input bridge. Browser targets report `unavailable:browser ime host backend is not implemented yet`, preserving room for a canvas/DOM text input bridge. Headless/server targets report unavailable because they have no text input host. This follows Bevy's window/input split: `dev/bevy/crates/bevy_window/src/event.rs` defines IME/window event vocabulary and `dev/bevy/crates/bevy_winit/src/state.rs` collects concrete winit IME events.

Keyboard event capability is declared as `platform.keyboard_events`. Winit-backed desktop and mobile targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards `WindowEvent::KeyboardInput` into `ZrRuntimeEventV1::keyboard`, then into runtime `InputEvent::KeyboardInput` and `KeyboardFocusLost` state cleanup. Browser targets report `unavailable:browser keyboard event host backend is not implemented yet` until a browser key event host feeds the same runtime ABI, and headless/server targets report unavailable because they have no physical keyboard event host. This follows Bevy's split where `dev/bevy/crates/bevy_input/src/keyboard.rs` defines `KeyboardInput`, `KeyboardFocusLost`, `KeyCode`, and `Key`, `dev/bevy/crates/bevy_winit/src/converters.rs` converts winit key events, and `dev/bevy/crates/bevy_winit/src/state.rs` forwards `WindowEvent::KeyboardInput`.

Cursor boundary capability is declared as `platform.cursor_boundary`. Desktop and mobile winit targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards pointer entered/left into runtime cursor-inside-window state. Browser targets report `unavailable:browser cursor boundary host backend is not implemented yet` until a browser host maps pointer enter/leave into the same runtime ABI. Headless/server targets report unavailable because they have no physical pointer boundary. This follows Bevy's `bevy_window::CursorEntered` / `CursorLeft` vocabulary and the concrete winit forwarding path in `dev/bevy/crates/bevy_winit/src/state.rs`.

Cursor options capability is declared as `platform.cursor_options`. Desktop winit targets currently report `unavailable:desktop cursor options host-request backend is not implemented yet`, because Zircon can observe cursor events but does not yet expose a runtime host-request path for cursor visibility, grab mode, hit-test, or cursor warping. Browser targets report `unavailable:browser cursor options host backend is not implemented yet`, and mobile/headless targets declare their missing or inapplicable host paths explicitly. This follows Bevy's split where `dev/bevy/crates/bevy_window/src/window.rs` defines `CursorOptions` and `CursorGrabMode`, while `dev/bevy/crates/bevy_winit/src/winit_windows.rs` applies initial cursor visibility/grab/hit-test settings and `dev/bevy/crates/bevy_winit/src/system.rs` applies changed cursor options through winit.

Mouse button capability is declared as `platform.mouse_buttons`. Winit-backed desktop and mobile targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards `WindowEvent::MouseInput` into `ZrRuntimeEventV1::mouse_button` and then into runtime `InputEvent::ButtonPressed` / `ButtonReleased` values for the neutral `ButtonInputState`. Browser targets report `unavailable:browser mouse button host backend is not implemented yet` until a browser pointer/mouse host path feeds the same runtime ABI, and headless/server targets report unavailable because they have no physical mouse button event host. This follows Bevy's split where `dev/bevy/crates/bevy_input/src/mouse.rs` defines `MouseButtonInput`, `MouseButton`, and button-state reduction, `dev/bevy/crates/bevy_winit/src/converters.rs` converts winit mouse buttons and element states, and `dev/bevy/crates/bevy_winit/src/state.rs` forwards `WindowEvent::MouseInput`.

Mouse wheel capability is declared as `platform.mouse_wheel`. Winit-backed desktop and mobile targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards `WindowEvent::MouseWheel` into `ZrRuntimeEventV1::mouse_wheel_delta`, then into runtime `MouseWheelEvent` values with `MouseScrollUnit::Line` or `MouseScrollUnit::Pixel`. Browser targets report `unavailable:browser mouse wheel host backend is not implemented yet` until a browser wheel/DOM host path feeds the same runtime ABI, and headless/server targets report unavailable because they have no physical wheel event host. This follows Bevy's split where `dev/bevy/crates/bevy_input/src/mouse.rs` defines `MouseWheel`, `MouseScrollUnit`, and accumulated scroll resources, while `dev/bevy/crates/bevy_winit/src/state.rs` translates concrete `WindowEvent::MouseWheel` line and pixel deltas.

Touch event capability is declared as `platform.touch_events`. Winit-backed desktop and mobile targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards `WindowEvent::Touch` into `ZrRuntimeEventV1::touch` and then into runtime `TouchPoint` values with stable ids, position, and `TouchPhase`. Browser targets report `unavailable:browser touch event host backend is not implemented yet` until a browser touch/DOM host path feeds the same runtime ABI, and headless/server targets report unavailable because they have no physical touch event host. This follows Bevy's split where `dev/bevy/crates/bevy_input/src/touch.rs` defines `TouchInput`, `TouchPhase`, and `Touches`, `dev/bevy/crates/bevy_winit/src/converters.rs` converts winit touch input, and `dev/bevy/crates/bevy_winit/src/state.rs` forwards `WindowEvent::Touch`.

Gesture event capability is declared as `platform.gesture_events`. Zircon currently reports `feature_disabled:input-gestures` by default because Bevy's gesture vocabulary is behind the `gestures` feature in `dev/bevy/crates/bevy_input/src/lib.rs`. When the feature is explicitly enabled, macOS and iOS winit targets still report `unavailable:winit gesture event host backend is not implemented yet`, while other winit targets report that gesture events are only declared for macOS and iOS. Browser targets report `unavailable:browser gesture event host backend is not implemented yet`, and headless/server targets report unavailable because they have no physical gesture host. This follows Bevy's split where `dev/bevy/crates/bevy_input/src/gestures.rs` defines `PinchGesture`, `RotationGesture`, `DoubleTapGesture`, and `PanGesture`, while `dev/bevy/crates/bevy_winit/src/state.rs` forwards `WindowEvent::PinchGesture`, `RotationGesture`, `DoubleTapGesture`, and `PanGesture`. Zircon deliberately stops at capability diagnostics in this slice because it has not yet added a runtime ABI or app-host forwarding path for gesture events.

Pointer position capability is declared as `platform.pointer_position`. Desktop and mobile winit targets report `supported:winit_window_events`, matching the runtime-preview host path that forwards local winit 0.31 beta `WindowEvent::PointerMoved` into `ZrRuntimeEventV1::pointer_moved` and then `InputFrameSnapshot::cursor_position`. Browser targets report `unavailable:browser pointer position host backend is not implemented yet` until a browser host maps pointer movement into the same runtime ABI. Headless/server targets report unavailable because they have no physical pointer-position host. This follows Bevy's `bevy_window::CursorMoved` event vocabulary in `dev/bevy/crates/bevy_window/src/event.rs` and the concrete `WindowEvent::CursorMoved` forwarding path in `dev/bevy/crates/bevy_winit/src/state.rs`.

Raw mouse motion capability is declared as `platform.raw_mouse_motion`. Desktop winit targets report `supported:winit_device_events`, matching the native runtime-preview host path that forwards raw mouse device deltas with winit `DeviceEvent::PointerMotion` -> `ZrRuntimeEventV1::mouse_motion` -> runtime input state. Mobile targets report `unavailable:mobile raw mouse motion host backend is not implemented yet`, browser targets report `unavailable:browser raw mouse motion host backend is not implemented yet`, and headless/server targets report unavailable because they have no physical pointing-device host. This follows Bevy's `bevy_winit` and `bevy_input::mouse::MouseMotion` split: the platform/backend layer owns OS event collection, while the runtime input layer owns neutral frame accumulation.

The host also forwards cursor position and boundary events with winit `WindowEvent::PointerMoved` / `PointerEntered` / `PointerLeft` -> ABI pointer and cursor boundary events -> runtime `InputFrameSnapshot::cursor_position` and `cursor_inside_window`. This follows Bevy's `bevy_window::CursorMoved` / `CursorEntered` / `CursorLeft` message path and keeps cursor presence separate from the last logical cursor position.

The host now forwards native file drag/drop events through the same descriptive platform/input boundary. Bevy's `bevy_winit` forwards file hover/drop/cancel host messages into `bevy_window::FileDragAndDrop`; Zircon's current winit 0.31 beta dependency exposes this as `DragEntered`, `DragDropped`, and `DragLeft`. The app layer maps those to `ZrRuntimeEventV1::file_hovered`, `file_dropped`, and `file_drag_cancelled`, and the runtime stores them as current-frame `FileDragDropEvent` values. The capability matrix reports desktop winit file drag/drop as `FileDragDropBackend::WinitWindowEvents`, while mobile, browser, and headless targets remain explicit unavailable declarations until their host backends exist.

Desktop winit also forwards Bevy-style window status messages through the ABI. Bevy defines `WindowMoved`, `WindowOccluded`, `WindowThemeChanged`, `WindowBackendScaleFactorChanged`, `WindowScaleFactorChanged`, `WindowCloseRequested`, and `WindowDestroyed` in `dev/bevy/crates/bevy_window/src/event.rs`, and `dev/bevy/crates/bevy_winit/src/state.rs` forwards the host events. Zircon maps local winit 0.31 beta `Moved`, `Occluded`, `ThemeChanged`, `ScaleFactorChanged`, `CloseRequested`, and `Destroyed` into `ZrRuntimeEventV1::window_moved`, `window_occluded`, `window_theme_changed`, `window_backend_scale_factor_changed`, `window_scale_factor_changed`, `window_close_requested`, and `window_destroyed`, then stores current-frame `WindowStatusEvent` values in the runtime input snapshot. Zircon currently emits both backend and logical scale-factor status events because it does not yet expose Bevy's window-resolution scale-factor override policy; viewport metrics and resize events still own the actual surface-size state.

The same host now forwards winit IME composition and delete-surrounding requests with `WindowEvent::Ime` -> `ZrRuntimeEventV1` IME events -> runtime `ImeEvent`. This follows Bevy's separation between `bevy_winit` collection and `bevy_window::Ime` messages while keeping winit's extra delete-surrounding signal available for Zircon text owners. The capability matrix remains descriptive; it does not imply mobile/web IME support until those backends expose a host path.

The runtime side now also declares native IME host requests through the same ABI family: enable, disable, cursor area, and surrounding text map to `ImeHostRequest` values in `InputFrameSnapshot::ime_host_requests`. `ZrRuntimeApiV1::drain_host_requests` exports those requests as a one-shot `ZrRuntimeHostRequestBatchV1`, and `zircon_app` applies them back to the native winit window with `Window::request_ime_update`. This follows Bevy's `Window::ime_enabled` / `Window::ime_position` configuration surface in `dev/bevy/crates/bevy_window/src/window.rs`, Bevy's winit IME event forwarding in `dev/bevy/crates/bevy_winit/src/state.rs`, and the richer cursor-area/surrounding-text shape exposed by the current winit 0.31 `ImeRequest` API. Native desktop preview now has the core host-application loop; mobile/web IME remains explicit future backend work.

Native desktop gamepad support now has a host path for the runtime preview app: `gamepad-gilrs` initializes and polls gilrs in `zircon_app`, then forwards connection, button, and axis events through `ZrRuntimeEventV1`. The runtime `default-platform` profile enables its own `gamepad-gilrs` capability feature so `zircon_runtime/target-client` and `zircon_runtime/target-editor-host` report the same desktop gilrs capability as the app host profile, without adding a gilrs dependency to `zircon_runtime`. This matches Bevy's crate split where `bevy_gilrs` is the platform backend and `bevy_input` owns neutral gamepad state. The capability matrix remains descriptive and continues to expose `GamepadBackend::Gilrs` only when the feature set enables it.

Gamepad event capability is declared separately as `platform.gamepad_events`. Desktop targets with `gamepad-gilrs` report `supported:gilrs_event_polling`, matching `zircon_app/src/entry/runtime_entry_app/gamepad/polling.rs` draining gilrs events and `events.rs` building runtime ABI connection/button/axis events. Browser targets report `supported:browser_gamepad_api_polling` only when `platform-web`, `input-gamepad`, and `gamepad-browser` are all enabled; otherwise they keep the same feature-disabled boundary as `platform.gamepad_input`. Mobile targets report unavailable until a mobile gamepad event host exists, and headless/server targets report unavailable because they have no physical event host. This follows Bevy's split where `dev/bevy/crates/bevy_input/src/gamepad.rs` defines `GamepadEvent`, `RawGamepadEvent`, connection, button, and axis event vocabulary, while `dev/bevy/crates/bevy_gilrs/src/gilrs_system.rs` drains gilrs events and writes neutral Bevy gamepad messages.

Gamepad rumble capability is declared separately as `platform.gamepad_rumble`. Desktop targets with `gamepad-gilrs` now report `supported:gilrs_force_feedback`: Zircon has the runtime request queue and host-request transport (`InputEvent::GamepadRumbleRequest` -> `ZrRuntimeHostRequestV1::GamepadRumble`), and `zircon_app` executes requests through gilrs force-feedback effects in the runtime preview host. The host tracks active effect lifetimes and clears them on explicit `Stop`, controller disconnect, and app shutdown. Browser targets report `feature_disabled:gamepad-browser` until browser gamepad input is enabled, then remain unavailable until a browser haptics host path exists. Mobile and headless/server targets are explicit unavailable or feature-disabled declarations. This follows Bevy's split where `dev/bevy/crates/bevy_input/src/gamepad.rs` defines `GamepadRumbleRequest`, while `dev/bevy/crates/bevy_gilrs/src/rumble.rs` consumes those requests through gilrs force-feedback.

Mobile targets use the mobile event-loop policy. Android requires a winit activity feature (`platform-android-game-activity` or `platform-android-native-activity`) before the matrix reports a supported winit window backend. Mobile gamepad is explicitly unavailable until a host backend is implemented.

Browser targets use `BrowserCanvas` when `platform-web` is enabled. Browser gamepad is now declared as its own `gamepad-browser` backend feature and reports `GamepadBackend::BrowserGamepadApi` only when `platform-web`, `input-gamepad`, and `gamepad-browser` are all enabled. This deliberately does not fall back to `gamepad-gilrs`: Bevy's checked-in `bevy_gilrs` plugin uses `GilrsBuilder` and has wasm-specific storage in `dev/bevy/crates/bevy_gilrs/src/lib.rs`, while Zircon's capability matrix names the browser host API separately so desktop gilrs polling, browser Gamepad API polling, and future browser haptics can evolve without becoming one ambiguous backend. Bevy's top-level `dev/bevy/Cargo.toml` also keeps `gamepad` as the neutral input feature and `bevy_gilrs` as a concrete backend feature; Zircon mirrors that split with `input-gamepad`, `gamepad-gilrs`, and `gamepad-browser`.

`EventLoopPolicy::Continuous` mirrors Bevy's `WinitSettings::continuous()` preset in `dev/bevy/crates/bevy_winit/src/winit_config.rs`: both focused and unfocused updates are continuous. Zircon exposes it as an explicit capability-report override rather than a default target policy. This lets runtime preview, profiling, or benchmark-style hosts request continuous polling without changing the normal client/editor/mobile/headless defaults.

Server runtime and explicit headless targets use `Headless` event-loop policy. They do not report physical gamepad support, and an explicit continuous event-loop request is normalized back to `Headless`.

## Tests

The platform, prelude, and app composition tests verify:

- the platform root remains structural;
- the platform test suite stays folder-backed under `zircon_runtime/src/platform/tests/`, with `mod.rs` as structural wiring and child files owning Cargo feature topology, feature-gate propagation, feature-selection policy fixtures, default desktop, event-loop policy, headless, Linux protocol, diagnostics, cross-target/mode matrix coverage, gesture, and gamepad capability assertions;
- platform target topology stays stable: `PlatformTarget::ALL`, target diagnostic tokens, desktop/mobile/browser/headless partitioning, and `PlatformTarget::current()` cfg mapping are covered explicitly;
- runtime target-mode policy stays stable: `client_runtime`, `server_runtime`, and `editor_host` diagnostic tokens, default event-loop policies, explicit headless-request fallback for windowed modes, and server/headless topology precedence are covered explicitly;
- backend and status diagnostic value tokens stay stable, including winit/browser/headless window backends, input-source backends, gilrs/browser gamepad backends, Linux protocol names, event-loop policy names, and the `supported:` / `feature_disabled:` / `unavailable:` status prefixes;
- `CapabilityStatus::is_supported()` treats only `Supported(...)` values as available, while feature-disabled and unavailable states remain negative even when they carry stable diagnostic reasons;
- capability-report diagnostic values stay in lockstep with the underlying `CapabilityStatus` variants across default desktop, headless/server, and browser gamepad gate transitions, so `platform.*` diagnostics cannot say `supported:*` when the report is feature-disabled or unavailable;
- metadata diagnostics stay separate from capability-status diagnostics: `platform.enabled`, `platform.target`, `platform.target_mode`, and `platform.event_loop_policy` remain plain booleans or policy tokens, not `supported:*`, `feature_disabled:*`, or `unavailable:*` values;
- feature-gate propagation stays explicit: disabling `platform-window` or `platform-winit` propagates to window-owned host capabilities, input gates take precedence over available host capability, and browser gamepad support cannot bypass `platform-web`;
- the full target/mode cross product keeps the same diagnostic surface for default, headless, and explicit continuous-policy reports, so adding a target or runtime mode cannot silently drop `platform.*` keys or leak Rust debug formatting into user-visible diagnostics;
- the runtime Cargo feature manifest keeps Bevy-style platform topology explicit: `default-platform` enables window/winit, X11, Wayland, and neutral input gates; `target-client` and `target-editor-host` stay windowed; `target-server` stays headless; winit-dependent platform features cascade through `platform-winit`; and concrete gamepad backends stay separate from neutral `input-gamepad`;
- `PlatformFeatureSelection` policy fixtures keep the Bevy-style split explicit: `none()` disables every declared gate, `bevy_default_platform()` enables window/winit, desktop Linux protocols, web/mobile declarations, neutral input, and desktop gilrs while leaving gestures/browser-gamepad opt-in, `headless()` enables only the headless topology gate, and `from_compiled_features()` mirrors the active Cargo feature set;
- client default platform reports winit, input, and gilrs on desktop;
- editor host uses the desktop-app event-loop policy;
- explicit continuous event-loop policy reports `platform.event_loop_policy=continuous` on windowed targets while headless topology remains `Headless`;
- server runtime remains headless;
- server/headless topology reports neutral mouse, keyboard, touch, and gesture input as `synthetic_only` only when the corresponding input feature gate is enabled, while physical window, pointer-event, file drag/drop, and gamepad host paths remain unavailable;
- Linux X11 and Wayland capability states are independent;
- capability reports and app module-selection diagnostics emit stable `platform.*` lines, including `platform.enabled`, target, event-loop policy, window backend, monitor inventory, window events, window lifecycle, window metrics, IME, keyboard events, cursor boundary, cursor options, mouse buttons, mouse wheel, touch events, gesture events, pointer position, raw mouse motion, input backends, gamepad backend, gamepad event backend, gamepad rumble state, file drag/drop state, and Linux protocol states;
- capability-report diagnostic keys stay ordered, unique by construction, and consistent across desktop, browser, mobile, and headless targets so platform capability additions cannot silently skip the user-visible diagnostics surface;
- monitor inventory diagnostics distinguish winit monitor handles from browser and headless unavailable host paths;
- window event diagnostics distinguish winit window events from browser and headless unavailable host paths;
- window lifecycle diagnostics distinguish winit lifecycle/status forwarding from browser and headless unavailable host paths;
- window metrics diagnostics distinguish winit resize/scale-factor metrics forwarding from browser and headless unavailable host paths;
- IME diagnostics distinguish native desktop winit IME from mobile, browser, and headless unavailable host paths;
- keyboard event diagnostics distinguish winit key event/focus cleanup support from browser and headless unavailable host paths;
- cursor boundary diagnostics distinguish winit pointer boundary events from browser and headless unavailable host paths;
- cursor options diagnostics distinguish winit-capable desktop hosts from the still-missing runtime host-request path for cursor visibility/grab/hit-test options;
- mouse button diagnostics distinguish winit button-state event support from browser and headless unavailable host paths;
- mouse wheel diagnostics distinguish winit line/pixel wheel event support from browser and headless unavailable host paths;
- touch event diagnostics distinguish winit touch phase/id event support from browser and headless unavailable host paths;
- gesture event diagnostics distinguish the `input-gestures` gate, future macOS/iOS winit gesture host paths, browser missing host support, and headless unavailable topology;
- pointer position diagnostics distinguish winit pointer/cursor moved events from browser and headless unavailable host paths;
- raw mouse motion diagnostics distinguish desktop winit device events from mobile, browser, and headless unavailable host paths;
- mobile and browser targets declare their current gamepad boundary explicitly, including the rule that browser gamepad support requires `gamepad-browser` and never aliases to desktop `gamepad-gilrs`;
- gamepad event diagnostics distinguish desktop gilrs event polling, browser Gamepad API polling, mobile missing host support, and headless unavailable topology;
- gamepad rumble diagnostics distinguish desktop gilrs force-feedback support from browser/mobile/headless unavailable states and feature-disabled browser gates;
- desktop, mobile, browser, and headless targets declare their current file drag/drop boundary explicitly;
- the runtime preview host preserves source-visible window status forwarding for move, occlusion, theme, close-request, and destroyed events;
- the runtime/input ABI exposes outgoing IME enable, cursor-area, and surrounding-text requests with validation before they become current-frame input state, drains them through an optional host-request API, and the native preview host applies them through winit `Window::request_ime_update`;
- the stable prelude can construct the default desktop capability report plus neutral window and input contracts through `crate::prelude::*`;
- app plugin groups keep platform/input in `DefaultPlugins`, `DevPlugins`, and `HeadlessPlugins`, while `MinimalPlugins` remains core-only.
- app bootstrap persists platform config for headless and minimal profile diagnostics.

M66 folder-split validation on 2026-05-21 checked the new `zircon_runtime/src/platform/capability/` subtree with scoped `rustfmt`, reran `cargo metadata --locked --no-deps --format-version 1`, confirmed the deleted flat `capability.rs` path has no live source/docs users, and ran scoped diff, trailing-whitespace, conflict-marker, and `boundary.rs` include-target checks. The focused runtime platform test command `cargo test -p zircon_runtime platform --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-capability-m66" --message-format short --color never -- --test-threads=1` passed 17 tests after the warm-target rerun; remaining warnings were unrelated existing ECS/query/UI test warnings.

M69 platform test folder-split validation on 2026-05-25 covers the hard cutover from flat `zircon_runtime/src/platform/tests.rs` to `zircon_runtime/src/platform/tests/`. The split is test organization only: production platform capability behavior, diagnostics, Cargo feature declarations, winit/gilrs host behavior, and app module-selection diagnostics are unchanged. Scoped `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, a 30-marker source contract, source stale-path scans, whitespace/conflict scans, and scoped `git diff --check` passed. `cargo test -p zircon_runtime ... --target-dir "F:\cargo-targets\zircon-platform-tests-m69"` repeatedly timed out during compile, but the produced test binary was then run directly with `platform::tests --test-threads=1` and passed 14 tests with 0 failed. Its Bevy anchors remain the same platform matrix references used by the tested behavior: `dev/bevy/Cargo.toml` for `default_platform`, `dev/bevy/docs/cargo_features.md` for `bevy_winit`/`bevy_gilrs`/`x11`/`wayland`/`web` feature vocabulary, `dev/bevy/crates/bevy_winit/src/winit_config.rs` for update-mode policy, and `dev/bevy/crates/bevy_gilrs/src/lib.rs` plus `gilrs_system.rs` for desktop gilrs event polling.

M70 platform feature-selection guard validation on 2026-05-25 covers the new `zircon_runtime/src/platform/tests/feature_selection.rs` child module. The guard is deliberately declarative: it keeps `none()`, `bevy_default_platform()`, `headless()`, and `from_compiled_features()` aligned with Bevy's `default_platform`/feature vocabulary without changing production platform behavior, Cargo feature declarations, diagnostics, or winit/gilrs host behavior. Scoped `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, and a 20-marker source contract passed. The focused Cargo command for `platform::tests::feature_selection` timed out during compile before a visible test summary, but its target processes finished and produced an updated test binary; direct execution of that binary with `platform::tests::feature_selection --test-threads=1` passed 4 tests with 0 failed and 1980 filtered out.

M71 platform feature-manifest topology guard covers the new `zircon_runtime/src/platform/tests/feature_manifest.rs` child module. It reads `zircon_runtime/Cargo.toml` directly and asserts the runtime feature graph that backs the declarative matrix: default desktop platform gates, client/editor/server profile defaults, winit backend cascades, independent input gates, default desktop gilrs capability declaration, and separated `gamepad-gilrs` / `gamepad-browser` backends. Its Bevy anchors are `dev/bevy/Cargo.toml` for `default_platform` and `bevy_gilrs = ["gamepad", ...]`, `dev/bevy/docs/cargo_features.md` for the platform feature vocabulary, and `dev/bevy/crates/bevy_gilrs/src/lib.rs` for gilrs as the concrete gamepad plugin. This is a guard against manifest drift; it now pins the M5 rule that runtime client/editor host defaults declare desktop gilrs capability while server remains headless and dependency-free. Validation on 2026-05-25 passed scoped `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, a 15-marker source contract, and the focused runtime command `cargo test -p zircon_runtime platform::tests::feature_manifest --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-tests-m69" --message-format short --color never -- --test-threads=1`, which ran 5 tests with 0 failed and 1986 filtered out after correcting the test helper to parse the manifest as `toml::Table`.

M72 platform diagnostic-key guard covers the new `zircon_runtime/src/platform/tests/diagnostic_keys.rs` child module. It asserts that `PlatformCapabilityReport::diagnostic_lines()` emits the complete ordered key surface and that `PlatformConfig::diagnostic_lines()` only prepends `platform.enabled` before the same report keys. It also checks desktop, browser, mobile, and headless reports share the same diagnostic key set. This follows Bevy's public diagnosability pattern without copying internals: Bevy documents platform feature selection in `dev/bevy/docs/cargo_features.md` and keeps winit/gilrs behavior feature-gated, while Zircon makes the selected platform capability matrix visible through stable `platform.*` diagnostics.

M72 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, and a 14-marker source contract. The focused runtime test command `cargo test -p zircon_runtime platform::tests::diagnostic_keys --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-tests-m69" --message-format short --color never -- --test-threads=1` was blocked before reaching the platform tests by unrelated active asset test compile errors in `zircon_runtime/src/asset/tests/assets/texture_importer.rs` where two call sites pass `String` values to an API currently expecting `&str`. That blocker belongs to the active asset-readiness session and is not changed by this platform diagnostic-key guard.

M73 platform backend-token guard covers the new `zircon_runtime/src/platform/tests/backend_tokens.rs` child module. It pins all public platform backend `as_str()` tokens and the status prefixes that appear in `PlatformCapabilityReport::diagnostic_lines()`: `supported:`, `feature_disabled:`, and `unavailable:`. This complements the M72 key guard by making diagnostics value tokens stable as the platform matrix evolves. Its Bevy anchors are the same feature-gated backend vocabulary used throughout this plan: `dev/bevy/docs/cargo_features.md` for `bevy_winit`, `bevy_gilrs`, `gamepad`, `x11`, `wayland`, and `web`, plus `dev/bevy/crates/bevy_winit/src/winit_config.rs` for event-loop policy vocabulary. This is a test/docs guard only and does not change backend enums, diagnostic values, or capability behavior.

M73 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, and a 14-marker source contract. The focused Cargo command `cargo test -p zircon_runtime platform::tests::backend_tokens --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-tests-m69" --message-format short --color never -- --test-threads=1` timed out while compiling, but the target processes finished and produced an updated test binary. Direct binary execution with `platform::tests::backend_tokens --test-threads=1` passed 4 tests with 0 failed and 1995 filtered out.

M74 platform target-topology guard covers the new `zircon_runtime/src/platform/tests/target_topology.rs` child module. It pins `PlatformTarget::ALL`, target `as_str()` diagnostic tokens, desktop/mobile/browser/headless helper partitioning, and `PlatformTarget::current()` cfg mapping. This gives the capability matrix a stable target vocabulary before more platform-specific backend work lands. Its Bevy anchors are the same public platform vocabulary used by Bevy's Cargo features: desktop X11/Wayland, Android activity features, wasm `web`, and winit-backed host routing in `dev/bevy/docs/cargo_features.md` plus the winit update-mode split in `dev/bevy/crates/bevy_winit/src/winit_config.rs`. This is a test/docs guard only and does not change target definitions or runtime target selection.

M74 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, and source-contract checks for the target-topology guard. The focused Cargo command `cargo test -p zircon_runtime platform::tests::target_topology --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-tests-m69" --message-format short --color never -- --test-threads=1` timed out while compiling, but the target processes finished and produced an updated test binary. Direct binary execution with `platform::tests::target_topology --test-threads=1` passed 4 tests with 0 failed and 1999 filtered out.

M75 runtime target-mode policy guard covers the new `zircon_runtime/src/platform/tests/target_modes.rs` child module. It pins the public `platform.target_mode` diagnostic tokens, default target-mode-to-event-loop policy mapping, the rule that an explicit `Headless` policy request falls back to the normal windowed default for client/editor modes, and the rule that server runtime remains headless across host targets even when `Continuous` is requested. Its Bevy anchors are `DefaultPlugins`/`MinimalPlugins` separating normal windowed app composition from minimal schedule-runner composition in `dev/bevy/crates/bevy_internal/src/default_plugins.rs`, plus `WinitSettings::{game, desktop_app, mobile, continuous}` in `dev/bevy/crates/bevy_winit/src/winit_config.rs`. This is a test/docs guard only and does not change `RuntimeTargetMode`, event-loop policy selection, window backend selection, diagnostics formatting, Cargo features, or winit/gilrs host behavior.

M75 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, a 12-marker source contract, and the focused runtime command `cargo test -p zircon_runtime platform::tests::target_modes --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-tests-m69" --message-format short --color never -- --test-threads=1`, which ran 4 tests with 0 failed and 2004 filtered out. The only emitted warnings were unrelated existing runtime UI/render/scene warnings.

M76 feature-gate propagation guard covers the new `zircon_runtime/src/platform/tests/feature_gate_propagation.rs` child module. It pins how disabled platform gates propagate through the already-declared matrix: `platform-window` gates window-owned capabilities, `platform-winit` gates desktop winit host capabilities, input feature gates take precedence over otherwise available host capability, and browser gamepad support requires `platform-web` before `gamepad-browser` can become meaningful. Its Bevy anchors are `dev/bevy/docs/cargo_features.md` for the public `default_platform` / `bevy_winit` / `bevy_gilrs` / `gamepad` / `web` feature vocabulary and `dev/bevy/Cargo.toml` for the split between neutral `gamepad` and concrete `bevy_gilrs`. This is a test/docs guard only and does not change `PlatformFeatureSelection`, `PlatformCapabilityMatrix`, diagnostics formatting, Cargo features, or winit/gilrs host behavior.

M76 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, and a 14-marker source contract. The focused Cargo command `cargo test -p zircon_runtime platform::tests::feature_gate_propagation --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-tests-m69" --message-format short --color never -- --test-threads=1` timed out during compile before a visible Cargo summary, but no target-dir cargo/rustc/test processes remained afterward and an updated test binary was produced. Direct binary execution with `platform::tests::feature_gate_propagation --test-threads=1` passed 4 tests with 0 failed and 2009 filtered out.

M77 matrix cross-product guard covers the new `zircon_runtime/src/platform/tests/matrix_cross_product.rs` child module. It runs the platform report diagnostic surface across every `PlatformTarget::ALL` member and every `RuntimeTargetMode` for default-platform, headless, and explicit continuous-policy reports. The guard pins complete ordered keys, duplicate-key absence, matching target/target-mode lines, event-loop policy presence, and absence of Rust debug enum formatting in public diagnostics. Its Bevy anchors are the same public surface this matrix mirrors: feature-gated `DefaultPlugins` and `MinimalPlugins` in `dev/bevy/crates/bevy_internal/src/default_plugins.rs`, Bevy's `default_platform` / `bevy_winit` / `bevy_gilrs` / `web` feature vocabulary in `dev/bevy/docs/cargo_features.md`, and `WinitSettings::{game, desktop_app, mobile, continuous}` in `dev/bevy/crates/bevy_winit/src/winit_config.rs`. This is a test/docs guard only and does not change report construction, diagnostic formatting, Cargo features, or winit/gilrs host behavior.

M77 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, and a 14-marker source contract. The focused Cargo command `cargo test -p zircon_runtime platform::tests::matrix_cross_product --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-tests-m69" --message-format short --color never -- --test-threads=1` timed out during compile before a visible Cargo summary, but no target-dir cargo/rustc/test processes remained afterward and an updated test binary was produced. Direct binary execution with `platform::tests::matrix_cross_product --test-threads=1` passed 3 tests with 0 failed and 2013 filtered out.

M78 headless synthetic-input policy guard covers the new `zircon_runtime/src/platform/tests/headless_synthetic_input.rs` child module. It pins the distinction between neutral input sources and physical host event sources for headless/server topology: enabled mouse, keyboard, touch, and gesture input gates can report `supported:synthetic_only`, but physical window events, pointer device events, file drag/drop, and gamepad host paths remain unavailable; the `headless()` fixture still disables all neutral input sources. Its Bevy anchors are `DefaultPlugins`/`MinimalPlugins` separating normal windowed app composition from minimal schedule-runner composition in `dev/bevy/crates/bevy_internal/src/default_plugins.rs`, Bevy's `default_platform`, `bevy_winit`, `bevy_gilrs`, and `gamepad` vocabulary in `dev/bevy/docs/cargo_features.md`, and `WinitSettings` update-mode policy in `dev/bevy/crates/bevy_winit/src/winit_config.rs`. This is a test/docs guard only and does not change `PlatformCapabilityMatrix`, `PlatformFeatureSelection`, diagnostics formatting, Cargo features, or winit/gilrs host behavior.

M78 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, and an 18-marker source contract. The focused Cargo command `cargo test -p zircon_runtime platform::tests::headless_synthetic_input --lib --no-default-features --features core-min --locked --jobs 1 --target-dir "F:\cargo-targets\zircon-platform-tests-m69" --message-format short --color never -- --test-threads=1` warmed the shared target but could not reach the platform tests because unrelated active ECS/query code failed lib-test compilation at `zircon_runtime/src/scene/ecs/system/query.rs:48` with `no method named cached_entities`. No target-dir cargo/rustc/test processes remained afterward and no updated `zircon_runtime` test binary was produced.

M79 capability-status semantics guard covers the new `zircon_runtime/src/platform/tests/status_semantics.rs` child module. It pins `CapabilityStatus::is_supported()` as a downstream-safe predicate: `Supported(...)` is true, while `FeatureDisabled { ... }` and `Unavailable { ... }` are false. The guard also checks representative desktop, headless, and synthetic-input reports so UI diagnostics, profile summaries, and future platform selection code cannot accidentally treat disabled features or unavailable physical host paths as supported. Its Bevy anchors remain the public feature/backend split mirrored by this matrix: feature-gated `DefaultPlugins`/`MinimalPlugins` in `dev/bevy/crates/bevy_internal/src/default_plugins.rs`, platform feature vocabulary in `dev/bevy/docs/cargo_features.md`, and winit update policy in `dev/bevy/crates/bevy_winit/src/winit_config.rs`. This is a test/docs guard only and does not change `CapabilityStatus`, report construction, diagnostics formatting, Cargo features, or winit/gilrs host behavior.

M79 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, and a 17-marker source contract. Focused Cargo test execution for `platform::tests::status_semantics` is deferred for this implementation slice because the shared checkout already had active Cargo/Rust compiler work from other lanes (`cargo=8`, `rustc=4`), including the active ECS CI expansion that owns `zircon_runtime/src/scene/ecs/system/query.rs`.

M80 diagnostic-status consistency guard covers the new `zircon_runtime/src/platform/tests/diagnostic_status_consistency.rs` child module. It compares every status-bearing `PlatformCapabilityReport` field with its emitted `platform.*` diagnostic value for representative default desktop, headless/server, and browser gamepad gate reports. Supported statuses must emit `supported:*`; feature-disabled statuses must emit `feature_disabled:<feature>`; unavailable statuses must emit `unavailable:<reason>`. This closes the gap between the M73 status-token guard and the M79 `is_supported()` predicate guard by preventing future diagnostics from drifting away from the actual capability status. Its Bevy anchors remain the public feature/backend split mirrored by this matrix: `DefaultPlugins` / `MinimalPlugins` are feature-gated in `dev/bevy/crates/bevy_internal/src/default_plugins.rs`, the cargo feature matrix exposes `default_platform`, `bevy_winit`, `bevy_gilrs`, `gamepad`, `web`, `x11`, and `wayland` in `dev/bevy/docs/cargo_features.md`, `dev/bevy/crates/bevy_winit/src/winit_config.rs` defines the windowed update-policy vocabulary, and `dev/bevy/crates/bevy_gilrs/src/lib.rs` plus `gilrs_system.rs` keep gilrs as the concrete desktop gamepad backend. This is a test/docs guard only and does not change report construction, diagnostic formatting, Cargo features, or winit/gilrs host behavior.

M80 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, a 20-marker source contract, scoped `git diff --check`, and scoped trailing-whitespace/conflict-marker scans. Focused Cargo test execution for `platform::tests::diagnostic_status_consistency` is deferred for this implementation slice because the shared checkout still has active Cargo/Rust compiler work from other lanes (`cargo=10`, `rustc=5`).

M81 diagnostic-metadata guard covers the new `zircon_runtime/src/platform/tests/diagnostic_metadata.rs` child module. It pins the non-capability metadata lines in `PlatformCapabilityReport` and `PlatformConfig`: `platform.target`, `platform.target_mode`, `platform.event_loop_policy`, and `platform.enabled` must stay plain tokens or booleans and must not use capability-status prefixes or status-style separators. This keeps policy metadata separate from the status-bearing capability fields guarded by M80. Its Bevy anchors are the same public boundaries this matrix mirrors: `DefaultPlugins` and `MinimalPlugins` separate windowed and minimal app composition in `dev/bevy/crates/bevy_internal/src/default_plugins.rs`, `dev/bevy/docs/cargo_features.md` documents platform feature selection, and `dev/bevy/crates/bevy_winit/src/winit_config.rs` defines the windowed update-policy token family that Zircon exposes as `platform.event_loop_policy`. This is a test/docs guard only and does not change report construction, diagnostic formatting, Cargo features, or winit/gilrs host behavior.

M81 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, a 14-marker source contract, scoped `git diff --check`, and scoped trailing-whitespace/conflict-marker scans. Focused Cargo test execution for `platform::tests::diagnostic_metadata` is deferred for this implementation slice because validation-time process checks still showed active Cargo/Rust compiler work from other lanes in the shared checkout.

M82 app feature-manifest guard covers the new `zircon_runtime/src/platform/tests/app_feature_manifest.rs` child module. It parses `zircon_app/Cargo.toml` as TOML and pins the app package feature surface that forwards platform/window/input defaults into `zircon_runtime`: `default-platform` must carry the runtime default platform plus window/winit/X11/Wayland/input/gamepad-gilrs host gates, `target-client` and `target-editor-host` must stay windowed, `target-server` must stay headless, winit protocol features must forward both runtime features and concrete `winit/*` backend features, gilrs and browser gamepad backends must remain separated, and `winit`/`softbuffer`/`gilrs` host dependencies must stay optional feature-owned dependencies. Its Bevy anchors are `DefaultPlugins` / `MinimalPlugins` feature-gated composition in `dev/bevy/crates/bevy_internal/src/default_plugins.rs` and the `default_platform`, `bevy_winit`, `bevy_gilrs`, `gamepad`, `web`, `x11`, and `wayland` vocabulary in `dev/bevy/docs/cargo_features.md`. This is a test/docs guard only and does not change Cargo features, runtime matrix behavior, diagnostics formatting, or winit/gilrs host behavior.

M82 scoped validation on 2026-05-25 passed `rustfmt --edition 2021 --check`, `cargo metadata --locked --no-deps --format-version 1`, a 12-marker source contract, scoped `git diff --check`, and scoped trailing-whitespace/conflict-marker scans. Focused Cargo test execution for `platform::tests::app_feature_manifest` is deferred for this implementation slice because validation-time process checks still showed active Cargo/Rust compiler work from other lanes in the shared checkout.

Workspace acceptance still requires CI-parity commands from `.github/workflows/ci.yml`, including workspace build/test and export platform contract checks.

M5 CI/export hardening now includes a `headless` export-platform contract lane beside `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, and `wasm`. `zircon_runtime::plugin::ExportTargetPlatform::Headless` is intentionally separate from `PlatformTarget::Headless`: the platform matrix continues to own runtime host capability diagnostics, while the export policy owns generated package shape. The built-in `server` export profile now resolves to the headless native export policy, emits a `target-server` source template with no mobile/browser `platform/*` shell files, and keeps CI from treating server acceptance as Linux desktop acceptance.

M6 documentation/example acceptance adds the mixed input event log harness `input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad`. It is deliberately runtime-side and hardware-free: one frame enters `DefaultInputManager` as normal `InputEvent` values for window, keyboard, mouse, touch, and gamepad families, then the test emits a deterministic log from `InputFrameSnapshot` plus contiguous `InputEventRecord` sequences. The focused harness command passed on 2026-05-28 with `cargo test -p zircon_runtime --lib input_manager_event_log_harness_covers_window_keyboard_mouse_touch_and_gamepad --locked --target-dir F:\cargo-targets\zircon-platform-m5-workspace --message-format short --color never -- --test-threads=1`, running 1 test with 0 failed and 2102 filtered out.

M6 final acceptance on 2026-05-28 reran the platform feature matrix checks and workspace gate. `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunExportPlatformContract -TargetDir F:\cargo-targets\zircon-platform-m5-export-headless -VerboseOutput` passed all eight export-platform lanes: `windows`, `linux`, `macos`, `android`, `ios`, `web_gpu`, `wasm`, and `headless`. `.\.codex\skills\zircon-dev\scripts\validate-matrix.ps1 -SkipBuild -SkipTest -RunProfileFeatureContract -TargetDir F:\cargo-targets\zircon-platform-m5-profile-runtime-server -VerboseOutput` passed all five profile-feature lanes: `zircon_app target-server`, `zircon_app target-client-platform`, `zircon_runtime target-client`, `zircon_runtime target-editor-host`, and `zircon_runtime target-server`.

The first final workspace validator attempt on `F:\cargo-targets\zircon-platform-m5-workspace` passed build and then exited the Cargo test phase with Windows access-violation code `-1073741819`. That failure did not reproduce: a direct `cargo test --workspace --locked --target-dir F:\cargo-targets\zircon-platform-m5-workspace --message-format short --color never` rerun exited `0` and completed all tests/doctests, and the final CI-parity validator on `D:\cargo-targets\zircon-platform-m6-workspace` passed both `cargo build --workspace --locked --verbose` and `cargo test --workspace --locked --verbose`. The accepted M6 workspace evidence is the passing D: validator run, chosen because D: had more than 50 GB free and avoided the F: cleanup/low-space variable.
