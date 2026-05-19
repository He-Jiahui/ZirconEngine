---
related_code:
  - zircon_runtime/src/platform/mod.rs
  - zircon_runtime/src/platform/config.rs
  - zircon_runtime/src/platform/capability.rs
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
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad.rs
  - zircon_runtime/src/core/framework/input/window_status.rs
  - zircon_runtime/Cargo.toml
  - zircon_app/Cargo.toml
implementation_files:
  - zircon_runtime/src/platform/capability.rs
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
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs
  - zircon_app/src/entry/runtime_entry_app/gamepad.rs
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
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
  - dev/bevy/docs/cargo_features.md
  - dev/bevy/crates/bevy_input/src/mouse.rs
  - dev/bevy/crates/bevy_gilrs/src/lib.rs
  - dev/bevy/crates/bevy_gilrs/src/gilrs_system.rs
tests:
  - zircon_runtime/src/platform/tests.rs
  - zircon_runtime/src/dynamic_api/tests.rs
  - zircon_runtime_interface/src/tests/contracts.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/entry/runtime_library/tests.rs
  - zircon_app/src/tests/prelude.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/plugins/tests.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
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
- event-loop policy: `Game`, `DesktopApp`, `Mobile`, `Continuous`, or `Headless`;
- mouse, keyboard, touch, and gesture input backend declarations;
- gamepad backend declaration: desktop `Gilrs`, browser `BrowserGamepadApi`, feature-disabled, or unavailable;
- file drag/drop backend declaration: desktop `WinitWindowEvents`, future `BrowserDragEvents`, or unavailable;
- Linux protocol declarations for X11 and Wayland.

`report(...)` derives the default event-loop policy from topology: client desktop uses `Game`, editor host uses `DesktopApp`, mobile uses `Mobile`, and server/headless uses `Headless`. `report_with_event_loop_policy(...)` is the explicit opt-in path for Bevy-style update-policy selection, including `EventLoopPolicy::Continuous`. Server runtime and explicit headless targets still report `Headless` even when an explicit continuous policy is requested, because the topology has no active window event loop to poll.

`PlatformConfig` keeps the existing `enabled` flag and adds target, runtime mode, and feature snapshot fields. `PLATFORM_CONFIG_KEY` is the runtime config-store key used by `zircon_app` bootstrap. `PlatformManager::capability_report()` is a thin access surface over that config. The primary window descriptor is stored separately under `PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY` by the app entry layer, so capability diagnostics can say which backend is available while window diagnostics say which primary-window policy was selected.

`BuiltinEngineEntry` stores a serialized `PlatformConfig` before module activation. Runtime/editor entries use the current host target and compiled feature snapshot; headless entries use `PlatformTarget::Headless` plus `PlatformFeatureSelection::headless()`. `RuntimeProfileId::Minimal` still stores the config for diagnostics, but marks it disabled because `MinimalPlugins` does not install `PlatformModule`.

`PlatformCapabilityReport::diagnostic_lines()` and `PlatformConfig::diagnostic_lines()` expose the same matrix through stable `platform.*` lines. The format is intentionally simple: supported entries use `supported:<backend>`, disabled feature gates use `feature_disabled:<feature>`, and not-yet-implemented host paths use `unavailable:<reason>`. `EntryModuleSelectionReport::diagnostic_lines()` now appends those lines after the entry target mode, so `EntryRunner::module_selection_diagnostics(...)` reports both the selected plugin/module group and the platform capability surface selected for that profile.

This mirrors Bevy's public diagnostic need without copying its internals: Bevy's `DefaultPlugins` are feature-gated and documented as user-controllable by disabling default features in `dev/bevy/crates/bevy_internal/src/default_plugins.rs`; `dev/bevy/docs/cargo_features.md` documents `default_platform`, `bevy_winit`, `bevy_gilrs`, `gamepad`, `x11`, `wayland`, and `web`; and `dev/bevy/crates/bevy_winit/src/winit_config.rs` separates game, desktop-app, mobile, and continuous update policies. Zircon keeps those decisions visible as capability diagnostics instead of relying on implicit Cargo feature knowledge.

The runtime-preview winit host maps this runtime-facing policy to winit `ControlFlow` in `zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs`: `Game` and `Continuous` use `ControlFlow::Poll`, while `DesktopApp`, `Mobile`, and `Headless` use `ControlFlow::Wait`. That mapping is app-host behavior and does not move concrete event-loop ownership into `zircon_runtime`.

## Target Policy

Desktop targets use `Winit` for windowing when `platform-window` and `platform-winit` are enabled. Linux separately declares `platform-x11` and `platform-wayland` so the engine can later validate X11/Wayland support independently instead of treating Linux as one opaque desktop path.

The platform matrix reports backend availability, while `zircon_runtime::core::framework::window::WindowDescriptor` now owns the neutral primary-window defaults that a backend can consume: primary-window handle, title, present mode, window mode, position, resolution, resize constraints, resizable/decorated/visible/focused flags, and physical/logical scale-factor conversion. This mirrors Bevy's `bevy_window::Window` and `WindowPlugin::primary_window` split from `bevy_winit` host creation. The descriptor is additive, is persisted through app bootstrap config, and is consumed by the runtime-preview app host for initial winit window attributes.

The native winit preview host now forwards raw mouse device deltas with winit `DeviceEvent::PointerMotion` -> `ZrRuntimeEventV1::mouse_motion` -> runtime input state. This follows Bevy's `bevy_winit` and `bevy_input::mouse::MouseMotion` split: the platform/backend layer owns OS event collection, while the runtime input layer owns neutral frame accumulation.

The host also forwards cursor boundary events with winit `WindowEvent::PointerEntered` / `PointerLeft` -> ABI cursor boundary events -> runtime `InputFrameSnapshot::cursor_inside_window`. This follows Bevy's `bevy_window::CursorEntered` / `CursorLeft` message path and keeps cursor presence separate from the last logical cursor position.

The host now forwards native file drag/drop events through the same descriptive platform/input boundary. Bevy's `bevy_winit` forwards file hover/drop/cancel host messages into `bevy_window::FileDragAndDrop`; Zircon's current winit 0.31 beta dependency exposes this as `DragEntered`, `DragDropped`, and `DragLeft`. The app layer maps those to `ZrRuntimeEventV1::file_hovered`, `file_dropped`, and `file_drag_cancelled`, and the runtime stores them as current-frame `FileDragDropEvent` values. The capability matrix reports desktop winit file drag/drop as `FileDragDropBackend::WinitWindowEvents`, while mobile, browser, and headless targets remain explicit unavailable declarations until their host backends exist.

Desktop winit also forwards Bevy-style window status messages through the ABI. Bevy defines `WindowMoved`, `WindowOccluded`, `WindowThemeChanged`, `WindowBackendScaleFactorChanged`, `WindowScaleFactorChanged`, `WindowCloseRequested`, and `WindowDestroyed` in `dev/bevy/crates/bevy_window/src/event.rs`, and `dev/bevy/crates/bevy_winit/src/state.rs` forwards the host events. Zircon maps local winit 0.31 beta `Moved`, `Occluded`, `ThemeChanged`, `ScaleFactorChanged`, `CloseRequested`, and `Destroyed` into `ZrRuntimeEventV1::window_moved`, `window_occluded`, `window_theme_changed`, `window_backend_scale_factor_changed`, `window_scale_factor_changed`, `window_close_requested`, and `window_destroyed`, then stores current-frame `WindowStatusEvent` values in the runtime input snapshot. Zircon currently emits both backend and logical scale-factor status events because it does not yet expose Bevy's window-resolution scale-factor override policy; viewport metrics and resize events still own the actual surface-size state.

The same host now forwards winit IME composition and delete-surrounding requests with `WindowEvent::Ime` -> `ZrRuntimeEventV1` IME events -> runtime `ImeEvent`. This follows Bevy's separation between `bevy_winit` collection and `bevy_window::Ime` messages while keeping winit's extra delete-surrounding signal available for Zircon text owners. The capability matrix remains descriptive; it does not imply mobile/web IME support until those backends expose a host path.

The runtime side now also declares native IME host requests through the same ABI family: enable, disable, cursor area, and surrounding text map to `ImeHostRequest` values in `InputFrameSnapshot::ime_host_requests`. `ZrRuntimeApiV1::drain_host_requests` exports those requests as a one-shot `ZrRuntimeHostRequestBatchV1`, and `zircon_app` applies them back to the native winit window with `Window::request_ime_update`. This follows Bevy's `Window::ime_enabled` / `Window::ime_position` configuration surface in `dev/bevy/crates/bevy_window/src/window.rs`, Bevy's winit IME event forwarding in `dev/bevy/crates/bevy_winit/src/state.rs`, and the richer cursor-area/surrounding-text shape exposed by the current winit 0.31 `ImeRequest` API. Native desktop preview now has the core host-application loop; mobile/web IME remains explicit future backend work.

Native desktop gamepad support now has a host path for the runtime preview app: `gamepad-gilrs` initializes and polls gilrs in `zircon_app`, then forwards connection, button, and axis events through `ZrRuntimeEventV1`. This matches Bevy's crate split where `bevy_gilrs` is the platform backend and `bevy_input` owns neutral gamepad state. The capability matrix remains descriptive and continues to expose `GamepadBackend::Gilrs` only when the feature set enables it.

Mobile targets use the mobile event-loop policy. Android requires a winit activity feature (`platform-android-game-activity` or `platform-android-native-activity`) before the matrix reports a supported winit window backend. Mobile gamepad is explicitly unavailable until a host backend is implemented.

Browser targets use `BrowserCanvas` when `platform-web` is enabled. Browser gamepad is now declared as its own `gamepad-browser` backend feature and reports `GamepadBackend::BrowserGamepadApi` only when `platform-web`, `input-gamepad`, and `gamepad-browser` are all enabled. This deliberately does not fall back to `gamepad-gilrs`: Bevy's checked-in `bevy_gilrs` plugin uses `GilrsBuilder` and has wasm-specific storage in `dev/bevy/crates/bevy_gilrs/src/lib.rs`, while Zircon's capability matrix names the browser host API separately so desktop gilrs polling and browser Gamepad API polling can evolve without becoming one ambiguous backend. Bevy's top-level `dev/bevy/Cargo.toml` also keeps `gamepad` as the neutral input feature and `bevy_gilrs` as a concrete backend feature; Zircon mirrors that split with `input-gamepad`, `gamepad-gilrs`, and `gamepad-browser`.

`EventLoopPolicy::Continuous` mirrors Bevy's `WinitSettings::continuous()` preset in `dev/bevy/crates/bevy_winit/src/winit_config.rs`: both focused and unfocused updates are continuous. Zircon exposes it as an explicit capability-report override rather than a default target policy. This lets runtime preview, profiling, or benchmark-style hosts request continuous polling without changing the normal client/editor/mobile/headless defaults.

Server runtime and explicit headless targets use `Headless` event-loop policy. They do not report physical gamepad support, and an explicit continuous event-loop request is normalized back to `Headless`.

## Tests

The platform, prelude, and app composition tests verify:

- the platform root remains structural;
- client default platform reports winit, input, and gilrs on desktop;
- editor host uses the desktop-app event-loop policy;
- explicit continuous event-loop policy reports `platform.event_loop_policy=continuous` on windowed targets while headless topology remains `Headless`;
- server runtime remains headless;
- Linux X11 and Wayland capability states are independent;
- capability reports and app module-selection diagnostics emit stable `platform.*` lines, including `platform.enabled`, target, event-loop policy, window backend, input backends, gamepad backend, file drag/drop state, and Linux protocol states;
- mobile and browser targets declare their current gamepad boundary explicitly, including the rule that browser gamepad support requires `gamepad-browser` and never aliases to desktop `gamepad-gilrs`;
- desktop, mobile, browser, and headless targets declare their current file drag/drop boundary explicitly;
- the runtime preview host preserves source-visible window status forwarding for move, occlusion, theme, close-request, and destroyed events;
- the runtime/input ABI exposes outgoing IME enable, cursor-area, and surrounding-text requests with validation before they become current-frame input state, drains them through an optional host-request API, and the native preview host applies them through winit `Window::request_ime_update`;
- the stable prelude can construct the default desktop capability report plus neutral window and input contracts through `crate::prelude::*`;
- app plugin groups keep platform/input in `DefaultPlugins`, `DevPlugins`, and `HeadlessPlugins`, while `MinimalPlugins` remains core-only.
- app bootstrap persists platform config for headless and minimal profile diagnostics.

Workspace acceptance still requires CI-parity commands from `.github/workflows/ci.yml`, including workspace build/test and export platform contract checks.
