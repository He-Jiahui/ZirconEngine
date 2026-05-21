---
related_code:
  - zircon_runtime/src/core/framework/window/mod.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/window/lifecycle_policy.rs
  - zircon_runtime/src/core/framework/window/mode.rs
  - zircon_runtime/src/core/framework/window/monitor_selection.rs
  - zircon_runtime/src/core/framework/window/position.rs
  - zircon_runtime/src/core/framework/window/primary_window_handle.rs
  - zircon_runtime/src/core/framework/window/present_mode.rs
  - zircon_runtime/src/core/framework/window/resize_constraints.rs
  - zircon_runtime/src/core/framework/window/resolution.rs
  - zircon_runtime/src/core/framework/window/validation.rs
  - zircon_runtime/src/core/framework/window/video_mode_selection.rs
  - zircon_runtime/src/core/framework/window/tests.rs
  - zircon_runtime/src/core/framework/mod.rs
  - zircon_runtime/src/prelude.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs
  - zircon_app/src/entry/runtime_entry_app/config.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/builder.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/fullscreen.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/monitor.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/position.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/video_mode.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
implementation_files:
  - zircon_runtime/src/core/framework/window/mod.rs
  - zircon_runtime/src/core/framework/window/constants.rs
  - zircon_runtime/src/core/framework/window/descriptor.rs
  - zircon_runtime/src/core/framework/window/lifecycle_policy.rs
  - zircon_runtime/src/core/framework/window/mode.rs
  - zircon_runtime/src/core/framework/window/monitor_selection.rs
  - zircon_runtime/src/core/framework/window/position.rs
  - zircon_runtime/src/core/framework/window/primary_window_handle.rs
  - zircon_runtime/src/core/framework/window/present_mode.rs
  - zircon_runtime/src/core/framework/window/resize_constraints.rs
  - zircon_runtime/src/core/framework/window/resolution.rs
  - zircon_runtime/src/core/framework/window/validation.rs
  - zircon_runtime/src/core/framework/window/video_mode_selection.rs
  - zircon_app/src/entry/entry_config.rs
  - zircon_app/src/entry/engine_entry.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_entry_app/config.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/builder.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/fullscreen.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/monitor.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/position.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/video_mode.rs
plan_sources:
  - .codex/plans/ZirconEngine Bevy 式 Platform Window Input Gilrs 完成度计划.md
  - .codex/plans/ZirconEngine Bevy 完成度两层路线图.md
  - dev/bevy/crates/bevy_window/src/lib.rs
  - dev/bevy/crates/bevy_window/src/system.rs
  - dev/bevy/crates/bevy_window/src/window.rs
  - dev/bevy/crates/bevy_window/src/event.rs
  - dev/bevy/crates/bevy_winit/src/winit_windows.rs
  - dev/bevy/crates/bevy_winit/src/winit_config.rs
tests:
  - zircon_runtime/src/core/framework/window/tests.rs
  - zircon_runtime/src/tests/prelude.rs
  - zircon_app/src/entry/tests/builtin_engine_entry.rs
  - zircon_app/src/entry/tests/profile_bootstrap.rs
  - zircon_app/src/entry/tests/mod.rs
  - zircon_app/src/entry/entry_runner/runtime.rs
  - zircon_app/src/entry/runtime_entry_app/config.rs
  - zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs
  - zircon_app/src/entry/runtime_entry_app/application_handler.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/mod.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/builder.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/fullscreen.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/monitor.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/position.rs
  - zircon_app/src/entry/runtime_entry_app/window_attributes/video_mode.rs
  - cargo test -p zircon_runtime core::framework::window --lib --no-default-features --features core-min --locked
  - cargo test -p zircon_runtime runtime_prelude_exports_platform_window_and_input_contracts --lib --no-default-features --features core-min --locked
  - cargo test -p zircon_app module_selection_report_formats_diagnostic_summary --lib --no-default-features --features platform-x11,platform-wayland,input-mouse,input-keyboard,input-touch --locked
  - cargo test -p zircon_app runtime_bootstrap_stores_primary_window_descriptor --lib --no-default-features --features platform-x11,platform-wayland,input-mouse,input-keyboard,input-touch --locked
  - cargo test -p zircon_app runtime_entry_keeps_window_lifecycle_policy_source_visible --lib --no-default-features --features platform-x11,platform-wayland,input-mouse,input-keyboard,input-touch --locked
  - cargo test -p zircon_app runtime_entry_maps_platform_event_loop_policy_to_winit_control_flow --lib --no-default-features --features platform-x11,platform-wayland,input-mouse,input-keyboard,input-touch --locked
doc_type: module-detail
---

# Runtime Framework Window Contracts

## Purpose

`zircon_runtime::core::framework::window` owns Zircon's neutral runtime window descriptor vocabulary. It is the M3 foundation for Bevy-style window defaults without moving host-specific `winit` ownership out of `zircon_app`.

The module is descriptive. It does not create OS windows, hold raw handles, run an event loop, or present swapchains. Those remain owned by `zircon_app` host code and runtime graphics surfaces. Runtime and editor-facing code can still agree on a primary-window shape through `WindowDescriptor`.

## Reference Evidence

Bevy is the primary reference for this slice:

- `dev/bevy/crates/bevy_window/src/lib.rs` defines `WindowPlugin::primary_window: Option<Window>` and marks the spawned entity as `PrimaryWindow`.
- `dev/bevy/crates/bevy_window/src/lib.rs` also defines `WindowPlugin::exit_condition` and `close_when_requested`; `dev/bevy/crates/bevy_window/src/system.rs` contains the close-on-request and exit-on-window-closed systems.
- `dev/bevy/crates/bevy_window/src/window.rs` defines `Window`, `WindowResolution`, `WindowResizeConstraints`, `WindowPosition`, `WindowMode`, and `PresentMode`.
- `dev/bevy/crates/bevy_winit/src/winit_windows.rs` resolves centered placement, borderless fullscreen, and exclusive fullscreen at the concrete winit creation boundary with monitor and video-mode data.
- `dev/bevy/crates/bevy_winit/src/system.rs` keeps runtime mode and position updates monitor-aware after creation.
- `dev/bevy/crates/bevy_winit/src/winit_config.rs` keeps event-loop policy separate from the platform-neutral window description.

Zircon deliberately keeps this first window model as framework DTOs instead of Bevy ECS components. The ECS/window-entity path can be added later once scene/runtime scheduling consumes window state directly.

## Data Model

The folder-backed module keeps declaration families separated:

- `PrimaryWindowHandle` is the neutral primary-window identity used before a concrete host handle exists.
- `WindowDescriptor` carries the primary-window handle, title, present mode, mode, position, resolution, resize constraints, and basic host-visible booleans.
- `WindowResolution` tracks physical size, backend scale factor, optional scale-factor override, and logical-size conversion.
- `WindowResizeConstraints` records logical min/max bounds and validates invalid or inverted bounds into a safe window range while preserving unbounded defaults.
- `WindowMonitorSelection` is the neutral Bevy-style monitor selector for current, primary, or indexed monitor choices.
- `WindowVideoMode` / `WindowVideoModeSelection` are the neutral Bevy-style video-mode selectors for the current monitor mode or a specific size/refresh/bit-depth request. Specific selections always match physical size; bit depth and refresh rate are optional constraints and stay wildcarded unless explicitly set.
- `WindowPosition` covers automatic placement, primary-monitor centered placement, selected-monitor centered placement, and explicit physical pixel coordinates.
- `WindowMode` covers windowed, primary-monitor borderless/fullscreen defaults, and selected-monitor borderless/exclusive fullscreen intent.
- `WindowPresentMode` mirrors Bevy's `AutoVsync`, `AutoNoVsync`, `Fifo`, `FifoRelaxed`, `Immediate`, and `Mailbox` vocabulary without importing `wgpu` or `winit` types.
- `WindowLifecyclePolicy` carries the Bevy-style close/exit policy as neutral data: `WindowExitCondition::{OnPrimaryClosed, OnAllClosed, DontExit}` plus `close_when_requested`.

`DEFAULT_WINDOW_TITLE` is `Zircon Runtime`. The default descriptor matches the current runtime preview host policy: primary window handle `0`, 1280x720, FIFO present mode, windowed, automatic position, resizable, decorated, visible, and focused.

`WindowLifecyclePolicy::default()` mirrors Bevy `WindowPlugin::default()`: close requests are honored and the app exits once all windows are closed. Zircon's current runtime-preview host owns only one primary window, so `OnPrimaryClosed` and `OnAllClosed` both mean "exit after the primary close request is applied". `DontExit` is explicit for no-window or service-style host profiles that should keep the runtime session alive after the primary window is gone.

`PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY` is `runtime.window.primary_descriptor`. `zircon_app::entry::EntryConfig` owns the selected `WindowDescriptor` at bootstrap time, and `BuiltinEngineEntry` stores it in `CoreRuntime` config before and after module activation, matching the existing app-owned platform/render config policy. Runtime/editor entries default to the primary descriptor above. Headless entries and runtime profiles that intentionally skip window ownership record `WindowDescriptor::without_primary_window()`, which keeps the descriptor serializable and diagnostic while setting `primary_window = None`, `visible = false`, and `focused = false`.

`WindowResizeConstraints` serializes unbounded max width/height as `null` and deserializes `null` back to `f32::INFINITY`. This is intentional because the runtime config store uses JSON, and JSON has no native representation for non-finite floats. The conversion keeps the public Rust DTO ergonomic while allowing default window descriptors to round-trip through `CoreHandle::store_config(...)` / `load_config(...)`.

## Ownership Boundary

The owner is `zircon_runtime::core::framework` because the values are neutral DTOs shared by runtime modules, the app host, and later editor host integration. `zircon_app` remains responsible for translating these DTOs into `winit::window::WindowAttributes` and event-loop behavior. `zircon_runtime::platform` remains responsible for declaring whether a window backend is available for the current target.

The first app-host conversion lives in `zircon_app/src/entry/runtime_entry_app/window_attributes/`. It maps the neutral descriptor to winit title, physical surface size, logical resize constraints, visibility, decoration, resizable and focus flags, explicit physical position, centered monitor placement, borderless fullscreen, and exclusive fullscreen intent. The folder root is structural; `builder.rs` assembles `WindowAttributes`, `monitor.rs` captures and selects winit monitors from the active event loop, `position.rs` owns centered physical placement, `fullscreen.rs` owns borderless/exclusive fullscreen fallback policy, and `video_mode.rs` owns current/specific video-mode matching. Legacy `WindowPosition::Centered`, `WindowMode::BorderlessFullscreen`, and `WindowMode::Fullscreen` retain their primary-monitor policy. The explicit variants `WindowPosition::CenteredOn`, `WindowMode::BorderlessFullscreenOn`, and `WindowMode::FullscreenOn` carry Bevy-style monitor/video-mode selection without storing concrete `winit::monitor::MonitorHandle` in runtime DTOs.

The runtime-preview host now consumes the descriptor through `RuntimeEntryAppConfig` instead of constructing only `WindowDescriptor::default()` internally. `zircon_app/src/entry/entry_runner/runtime.rs` derives this config from the already-parsed runtime session profile: `runtime` keeps the default primary window, `EventLoopPolicy::Game`, and `WindowLifecyclePolicy::default()`; `editor` and `dev` keep the primary window and use `EventLoopPolicy::DesktopApp`; `minimal` and `headless` use `WindowDescriptor::without_primary_window()`, `EventLoopPolicy::Headless`, and `WindowExitCondition::DontExit`. `ApplicationHandler::can_create_surfaces()` checks `primary_window` before calling winit window creation, so profile-level no-window policy reaches the concrete host path. `RuntimeEntryAppConfig` carries Bevy-style close/exit policy through `WindowLifecyclePolicy`, whose default keeps `close_when_requested = true` like `WindowPlugin::close_when_requested`.

Event-loop update mode remains a separate platform policy rather than a window descriptor field. `zircon_runtime::platform::EventLoopPolicy` names the runtime-facing policy, while `zircon_app/src/entry/runtime_entry_app/event_loop_policy.rs` owns the current winit `ControlFlow` mapping for the runtime-preview host.

Window lifecycle forwarding is host behavior layered on top of the neutral descriptor and lifecycle policy. `zircon_app/src/entry/runtime_entry_app/application_handler.rs` keeps close, destroyed, moved, occluded, theme, scale-factor, and focus changes in the winit host surface, then emits neutral `ZrRuntimeEventV1` records. The close path forwards `window_close_requested` before applying `WindowLifecyclePolicy::should_close_on_request()` and `should_exit_after_primary_close()`; focus maps to foreground/background lifecycle states; scale-factor changes keep backend and logical notifications separate. This mirrors Bevy's separation between `bevy_window` event vocabulary/policy and `bevy_winit` event translation in `dev/bevy/crates/bevy_window/src/event.rs`, `dev/bevy/crates/bevy_window/src/lib.rs`, `dev/bevy/crates/bevy_window/src/system.rs`, and `dev/bevy/crates/bevy_winit/src/state.rs`.

The app bootstrap path now exposes the same descriptor through `EntryModuleSelectionReport::diagnostic_lines()` before `CoreRuntime` activation. The stable lines include `window.primary_window`, `window.title`, `window.present_mode`, `window.mode`, `window.position`, `window.physical_size`, `window.logical_size`, `window.scale_factor`, `window.scale_factor_override`, `window.resize_constraints`, and the host-visible booleans. These lines are diagnostic text, not a second runtime API.

This split keeps Bevy-style `bevy_window` / `bevy_winit` separation: the runtime framework owns the window vocabulary, while the app host owns concrete platform creation. The primary-window absence behavior follows Bevy `WindowPlugin::primary_window: Option<Window>` in `dev/bevy/crates/bevy_window/src/lib.rs`; the app-host update policy follows Bevy `WinitSettings::{game, desktop_app, continuous}` and `UpdateMode` in `dev/bevy/crates/bevy_winit/src/winit_config.rs`; monitor-aware window creation follows Bevy's `WindowPosition`, `WindowMode`, and winit-window creation split in `dev/bevy/crates/bevy_window/src/window.rs` and `dev/bevy/crates/bevy_winit/src/winit_windows.rs`.

## Test Coverage

`zircon_runtime/src/core/framework/window/tests.rs` verifies:

- default primary-window policy,
- default close/exit lifecycle policy and explicit no-exit/ignore-close variants,
- physical/logical/scale-factor conversion,
- invalid resize-constraint clamping and unbounded defaults,
- JSON round-trip behavior for unbounded resize constraints,
- builder methods for host-neutral settings,
- absent-primary-window diagnostics for headless/minimal profile policy,
- structural root-module exports so implementation stays out of `window/mod.rs`.
- app-host config defaults and profile mapping from runtime session profiles to primary-window/event-loop policy choices.
- source-visible app-host lifecycle policy for no-primary-window creation skip, close-before-policy-exit ordering, window status forwarding, focus lifecycle mapping, configurable `WindowLifecyclePolicy`, and backend-before-logical scale-factor forwarding.
- app-host conversion from `WindowDescriptor` to winit `WindowAttributes` for default descriptors, custom descriptors, centered placement fallback before monitor context is available, selected-monitor centered physical-position math, borderless fullscreen, and exclusive-fullscreen fallback.
- app entry diagnostics and bootstrap config persistence through `PRIMARY_WINDOW_DESCRIPTOR_CONFIG_KEY`.

`zircon_runtime/src/tests/prelude.rs` verifies the stable runtime prelude exports the window descriptor vocabulary. Use the fully qualified `core::framework::window` test filter for this module's focused Cargo validation; the broader `window` substring also matches unrelated dynamic/render tests.

Milestone-level Cargo acceptance remains the platform/window/input testing stage from the active Bevy parity plan. The neutral descriptor foundation is now consumed by the runtime preview app for initial winit window attributes, Bevy-style monitor/video-mode selection DTOs, monitor-aware placement/fullscreen selection, and profile-driven primary-window plus close/exit policy. Later M3 slices can add monitor inventory diagnostics around unavailable or falling-back monitor selections.
