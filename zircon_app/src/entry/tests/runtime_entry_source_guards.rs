use super::source_assertions::assert_source_order;

fn runtime_event_loop_policy_source() -> String {
    [
        include_str!("../runtime_entry_app/event_loop_policy/mod.rs"),
        include_str!("../runtime_entry_app/event_loop_policy/control_flow.rs"),
    ]
    .join("\n")
}

fn runtime_config_source() -> String {
    [
        include_str!("../runtime_entry_app/config/mod.rs"),
        include_str!("../runtime_entry_app/config/app_config.rs"),
    ]
    .join("\n")
}

fn runtime_application_handler_source() -> &'static str {
    include_str!("../runtime_entry_app/application_handler/hooks.rs")
}

fn runtime_window_events_source() -> String {
    [
        include_str!("../runtime_entry_app/window_events/mod.rs"),
        include_str!("../runtime_entry_app/window_events/dispatch.rs"),
    ]
    .join("\n")
}

#[test]
fn runtime_entry_application_handler_stays_folder_backed_hook_surface() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let application_handler_root_source =
        include_str!("../runtime_entry_app/application_handler/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry");

    assert!(
        runtime_app_source.contains("mod application_handler;"),
        "runtime entry app should keep the winit ApplicationHandler implementation in a child module"
    );
    assert!(
        application_handler_root_source.contains("mod hooks;"),
        "runtime application-handler root should stay structural and delegate trait hooks"
    );
    assert!(
        !root.join("runtime_entry_app/application_handler.rs")
            .exists(),
        "runtime application handler should stay folder-backed instead of returning to an umbrella application_handler.rs file"
    );
    assert_source_order(
        runtime_handler_source,
        &[
            "impl ApplicationHandler for RuntimeEntryApp",
            "fn can_create_surfaces",
            "self.create_primary_window_surface(event_loop);",
            "fn window_event",
            "self.handle_window_event(event_loop, event);",
            "fn about_to_wait",
            "self.pump_frame_loop(event_loop);",
            "fn device_event",
            "self.handle_device_event(event_loop, event);",
        ],
        "runtime ApplicationHandler hooks should remain a narrow profile-and-delegate surface",
    );
}

#[test]
fn runtime_entry_window_event_dispatch_stays_in_child_module() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_window_events_source = runtime_window_events_source();
    let runtime_window_events_root_source =
        include_str!("../runtime_entry_app/window_events/mod.rs");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry");

    assert!(
        runtime_app_source.contains("mod window_events;"),
        "runtime entry app should declare a child module for concrete winit window-event dispatch"
    );
    assert!(
        runtime_window_events_root_source.contains("mod dispatch;"),
        "runtime window-events root should stay structural and delegate dispatch"
    );
    assert!(
        !root.join("runtime_entry_app/window_events.rs").exists(),
        "runtime window events should stay folder-backed instead of returning to an umbrella window_events.rs file"
    );
    assert_source_order(
        runtime_handler_source,
        &[
            "fn window_event",
            "zircon_runtime::profile_scope!(\"app\", \"runtime_entry\", \"window_event\");",
            "self.handle_window_event(event_loop, event);",
        ],
        "ApplicationHandler::window_event should only profile and delegate concrete dispatch",
    );
    for forbidden in [
        "WindowEvent::CloseRequested",
        "WindowEvent::Destroyed",
        "WindowEvent::Moved",
        "WindowEvent::Occluded",
        "WindowEvent::ThemeChanged",
        "WindowEvent::ScaleFactorChanged",
        "WindowEvent::SurfaceResized",
        "WindowEvent::Focused",
        "WindowEvent::PointerEntered",
        "WindowEvent::PointerLeft",
        "WindowEvent::DragEntered",
        "WindowEvent::DragDropped",
        "WindowEvent::DragLeft",
        "WindowEvent::PointerMoved",
        "WindowEvent::PointerButton",
        "WindowEvent::KeyboardInput",
        "WindowEvent::Ime",
        "WindowEvent::MouseWheel",
        "WindowEvent::RedrawRequested",
    ] {
        assert!(
            !runtime_handler_source.contains(forbidden),
            "ApplicationHandler should not own concrete window-event arm `{forbidden}`"
        );
    }
    assert_source_order(
        runtime_window_events_source.as_str(),
        &[
            "fn handle_window_event",
            "WindowEvent::CloseRequested",
            "self.handle_window_close_requested(event_loop);",
            "WindowEvent::Destroyed",
            "self.handle_window_destroyed(event_loop);",
            "WindowEvent::Moved(position)",
            "self.handle_window_moved(event_loop, position);",
            "WindowEvent::Occluded(occluded)",
            "self.handle_window_occluded(event_loop, occluded);",
            "WindowEvent::ThemeChanged(theme)",
            "self.handle_window_theme_changed(event_loop, theme);",
            "WindowEvent::ScaleFactorChanged { scale_factor, .. }",
            "self.handle_window_scale_factor_changed(event_loop, scale_factor);",
            "WindowEvent::SurfaceResized(size)",
            "self.resize_surface_presenter(event_loop, size);",
            "WindowEvent::Focused(focused)",
            "self.handle_window_focus_changed(event_loop, focused);",
            "WindowEvent::PointerEntered",
            "self.handle_pointer_entered(event_loop);",
            "WindowEvent::PointerLeft { position, kind, .. }",
            "self.handle_pointer_left(event_loop, position, kind);",
            "WindowEvent::DragEntered { paths, .. }",
            "self.handle_files_hovered(event_loop, paths);",
            "WindowEvent::DragDropped { paths, .. }",
            "self.handle_files_dropped(event_loop, paths);",
            "WindowEvent::DragLeft { .. }",
            "self.handle_file_drag_cancelled(event_loop);",
            "WindowEvent::PointerMoved",
            "self.handle_pointer_moved(event_loop, position, source);",
            "WindowEvent::PointerButton",
            "self.handle_pointer_button(event_loop, state, button, position);",
            "WindowEvent::KeyboardInput { event, .. }",
            "self.handle_keyboard_input(event_loop, event);",
            "WindowEvent::Ime(ime)",
            "self.handle_ime_input(event_loop, ime);",
            "WindowEvent::MouseWheel { delta, .. }",
            "self.handle_mouse_wheel(event_loop, delta);",
            "WindowEvent::RedrawRequested",
            "self.present_redraw_frame(event_loop);",
        ],
        "runtime window-event dispatcher should preserve every concrete winit arm and delegate to focused child behavior modules",
    );
}

#[test]
fn runtime_entry_converter_helpers_stay_family_split() {
    let runtime_converter_root_source = include_str!("../runtime_entry_app/converters/mod.rs");
    let runtime_converter_abi_source = include_str!("../runtime_entry_app/converters/abi.rs");
    let runtime_converter_keyboard_source =
        include_str!("../runtime_entry_app/converters/keyboard.rs");
    let runtime_converter_pointer_source =
        include_str!("../runtime_entry_app/converters/pointer.rs");
    let runtime_converter_window_source = include_str!("../runtime_entry_app/converters/window.rs");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry")
        .join("runtime_entry_app");

    assert!(
        !root.join("converters.rs").exists(),
        "runtime entry converters should be folder-backed instead of returning to an umbrella converters.rs file"
    );
    for required in [
        "mod abi;",
        "mod keyboard;",
        "mod pointer;",
        "mod window;",
        "pub(super) use abi::{byte_slice, usize_to_u32};",
        "pub(super) use keyboard::{key_action, physical_key_code};",
        "pub(super) use pointer::{",
        "pub(super) use window::window_theme;",
    ] {
        assert!(
            runtime_converter_root_source.contains(required),
            "runtime converter root should keep structural re-export `{required}`"
        );
    }
    for required in [
        "fn byte_slice",
        "ZrByteSlice",
        "fn usize_to_u32",
        "u32::try_from(value).unwrap_or(u32::MAX - 1)",
    ] {
        assert!(
            runtime_converter_abi_source.contains(required),
            "runtime ABI converter module should preserve `{required}`"
        );
    }
    for required in [
        "fn key_action",
        "ZR_RUNTIME_KEY_ACTION_PRESSED_V1",
        "ZR_RUNTIME_KEY_ACTION_RELEASED_V1",
        "fn physical_key_code",
        "fn stable_key_code",
        "FNV_OFFSET",
    ] {
        assert!(
            runtime_converter_keyboard_source.contains(required),
            "runtime keyboard converter module should preserve `{required}`"
        );
    }
    for required in [
        "fn pointer_source_touch_id",
        "fn pointer_kind_touch_id",
        "fn touch_button_phase",
        "fn mouse_button",
        "fn button_state",
        "fn mouse_wheel_delta",
        "ZR_RUNTIME_MOUSE_WHEEL_UNIT_LINE_V1",
        "ZR_RUNTIME_MOUSE_WHEEL_UNIT_PIXEL_V1",
    ] {
        assert!(
            runtime_converter_pointer_source.contains(required),
            "runtime pointer converter module should preserve `{required}`"
        );
    }
    for required in [
        "fn window_theme",
        "ZR_RUNTIME_WINDOW_THEME_LIGHT_V1",
        "ZR_RUNTIME_WINDOW_THEME_DARK_V1",
    ] {
        assert!(
            runtime_converter_window_source.contains(required),
            "runtime window converter module should preserve `{required}`"
        );
    }
}

#[test]
fn runtime_entry_ticks_dynamic_runtime_time_before_redraw_request() {
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_frame_loop_source = include_str!("../runtime_entry_app/frame_loop.rs");
    let runtime_session_source = include_str!("../runtime_library/runtime_session.rs");
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");

    assert!(
        runtime_session_source.contains("pub(crate) fn tick_frame"),
        "runtime session should expose an optional dynamic runtime tick wrapper"
    );
    assert!(
        runtime_app_source.contains("event_loop_policy: EventLoopPolicy"),
        "runtime entry app should store the selected event-loop policy"
    );
    assert_source_order(
        runtime_handler_source,
        &["fn about_to_wait", "self.pump_frame_loop(event_loop);"],
        "runtime entry ApplicationHandler should delegate about-to-wait frame pumping",
    );
    assert!(
        runtime_app_source.contains("mod frame_loop;"),
        "runtime entry app should keep the frame pump in a child module"
    );
    assert_source_order(
        runtime_frame_loop_source,
        &[
            "fn pump_frame_loop",
            "self.apply_event_loop_policy(event_loop);",
            "self.session.tick_frame()",
            "self.apply_runtime_host_requests(event_loop)",
            "window.request_redraw();",
        ],
        "runtime entry should advance runtime time and apply host requests before requesting the next redraw",
    );
}

#[test]
fn runtime_entry_maps_platform_event_loop_policy_to_winit_control_flow() {
    let event_loop_policy_root_source =
        include_str!("../runtime_entry_app/event_loop_policy/mod.rs");
    let event_loop_policy_control_flow_source =
        include_str!("../runtime_entry_app/event_loop_policy/control_flow.rs");
    let event_loop_policy_source = runtime_event_loop_policy_source();
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry");

    for required in [
        "EventLoopPolicy::Game",
        "EventLoopPolicy::Continuous",
        "EventLoopPolicy::DesktopApp",
        "EventLoopPolicy::Mobile",
        "EventLoopPolicy::Headless",
        "ControlFlow::Poll",
        "ControlFlow::Wait",
        "event_loop.set_control_flow",
        "pub(in crate::entry::runtime_entry_app) fn apply_event_loop_policy",
    ] {
        assert!(
            event_loop_policy_source.contains(required),
            "runtime event-loop policy helper should preserve `{required}`"
        );
    }
    assert!(
        runtime_app_source.contains("mod event_loop_policy;"),
        "runtime entry app should keep event-loop policy mapping in a child module"
    );
    assert!(
        event_loop_policy_root_source.contains("mod control_flow;"),
        "runtime event-loop policy root should stay structural and delegate control-flow behavior"
    );
    assert!(
        !root.join("runtime_entry_app/event_loop_policy.rs").exists(),
        "runtime event-loop policy should stay folder-backed instead of returning to an umbrella event_loop_policy.rs file"
    );
    assert_source_order(
        event_loop_policy_control_flow_source,
        &[
            "fn apply_event_loop_policy",
            "event_loop.set_control_flow(winit_control_flow(self.event_loop_policy));",
            "fn winit_control_flow",
            "EventLoopPolicy::Game | EventLoopPolicy::Continuous",
            "ControlFlow::Poll",
            "EventLoopPolicy::DesktopApp | EventLoopPolicy::Mobile | EventLoopPolicy::Headless",
            "ControlFlow::Wait",
        ],
        "event-loop policy control-flow helper should keep the runtime profile to winit ControlFlow mapping source-visible",
    );
}

#[test]
fn runtime_runner_projects_session_profile_into_app_host_config() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_config_root_source = include_str!("../runtime_entry_app/config/mod.rs");
    let runtime_config_app_config_source =
        include_str!("../runtime_entry_app/config/app_config.rs");
    let runtime_config_source = runtime_config_source();
    let runtime_window_creation_source = include_str!("../runtime_entry_app/window_creation.rs");
    let runtime_construct_source = include_str!("../runtime_entry_app/construct.rs");
    let runtime_runner_source = include_str!("../entry_runner/runtime.rs");
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry");

    assert!(
        runtime_app_source.contains("mod config;")
            && runtime_app_source.contains("RuntimeEntryAppConfig"),
        "runtime entry app should keep host configuration in a child module"
    );
    assert!(
        runtime_config_root_source.contains("mod app_config;")
            && runtime_config_root_source
                .contains("pub(in crate::entry) use app_config::RuntimeEntryAppConfig;"),
        "runtime config root should stay structural and expose the host config type"
    );
    assert!(
        !root.join("runtime_entry_app/config.rs").exists(),
        "runtime config should stay folder-backed instead of returning to an umbrella config.rs file"
    );
    assert!(
        runtime_config_source.contains("WindowDescriptor")
            && runtime_config_source.contains("EventLoopPolicy")
            && runtime_config_source.contains("WindowLifecyclePolicy"),
        "runtime entry app config should carry the neutral window descriptor, event-loop policy, and lifecycle policy"
    );
    assert!(
        runtime_config_source.contains("with_window_lifecycle_policy")
            && runtime_config_source.contains("with_close_when_requested")
            && runtime_config_source.contains("window_lifecycle_policy(&self)"),
        "runtime entry app config should expose the Bevy-style close/exit host policy"
    );
    assert_source_order(
        runtime_config_app_config_source,
        &[
            "struct RuntimeEntryAppConfig",
            "window_descriptor: WindowDescriptor",
            "event_loop_policy: EventLoopPolicy",
            "window_lifecycle_policy: WindowLifecyclePolicy",
            "fn with_window_descriptor",
            "fn with_event_loop_policy",
            "fn with_window_lifecycle_policy",
            "impl Default for RuntimeEntryAppConfig",
            "EventLoopPolicy::Game",
        ],
        "runtime app-config implementation should keep host policy fields, builder methods, and defaults source-visible",
    );
    assert!(
        runtime_construct_source.contains("RuntimeEntryAppConfig")
            && runtime_construct_source.contains("config.window_descriptor")
            && runtime_construct_source.contains("config.event_loop_policy")
            && runtime_construct_source.contains("config.window_lifecycle_policy"),
        "runtime entry construction should seed host state from RuntimeEntryAppConfig"
    );
    assert!(
        runtime_app_source.contains("window_lifecycle_policy: WindowLifecyclePolicy"),
        "runtime entry construction should store close/exit policy from RuntimeEntryAppConfig"
    );
    assert!(
        runtime_window_creation_source.contains("self.window_descriptor.primary_window.is_none()"),
        "runtime entry should skip concrete winit window creation when the host config has no primary window"
    );
    assert_source_order(
        runtime_runner_source,
        &[
            "parse_runtime_session_startup_args",
            "RuntimeSession::create_with_profile(runtime, runtime_session_args.profile.as_bytes())",
            "runtime_entry_app_config_for_session_profile(runtime_session_args.profile)",
            "RuntimeEntryApp::new(session, host_config)",
        ],
        "runtime runner should derive the app host config from the already-parsed session profile before creating the app",
    );
    for required in [
        "RuntimeSessionProfile::Runtime => RuntimeEntryAppConfig::default()",
        "RuntimeSessionProfile::Editor | RuntimeSessionProfile::Dev",
        "EventLoopPolicy::DesktopApp",
        "RuntimeSessionProfile::Minimal | RuntimeSessionProfile::Headless",
        "WindowDescriptor::default().without_primary_window()",
        "EventLoopPolicy::Headless",
        "WindowExitCondition::DontExit",
    ] {
        assert!(
            runtime_runner_source.contains(required),
            "runtime session profile host mapping should preserve `{required}`"
        );
    }
}

#[test]
fn runtime_entry_window_attributes_use_monitor_aware_creation_policy() {
    let runtime_handler_source = runtime_application_handler_source();
    let runtime_window_creation_source = include_str!("../runtime_entry_app/window_creation.rs");
    let window_attributes_root_source =
        include_str!("../runtime_entry_app/window_attributes/mod.rs");
    let window_attributes_builder_source =
        include_str!("../runtime_entry_app/window_attributes/builder.rs");
    let window_attributes_fullscreen_source =
        include_str!("../runtime_entry_app/window_attributes/fullscreen.rs");
    let window_attributes_monitor_source =
        include_str!("../runtime_entry_app/window_attributes/monitor.rs");
    let window_attributes_position_source =
        include_str!("../runtime_entry_app/window_attributes/position.rs");
    let window_attributes_video_mode_source =
        include_str!("../runtime_entry_app/window_attributes/video_mode.rs");
    let window_attributes_source = [
        window_attributes_root_source,
        window_attributes_builder_source,
        window_attributes_fullscreen_source,
        window_attributes_monitor_source,
        window_attributes_position_source,
        window_attributes_video_mode_source,
    ]
    .join("\n");

    assert_source_order(
        runtime_handler_source,
        &[
            "fn can_create_surfaces",
            "self.create_primary_window_surface(event_loop);",
        ],
        "runtime entry ApplicationHandler should delegate primary window creation to the window-creation module",
    );
    assert_source_order(
        runtime_window_creation_source,
        &[
            "fn create_primary_window_surface",
            "runtime_window_attributes(&self.window_descriptor, event_loop)",
            "event_loop.create_window(window_attributes)",
        ],
        "runtime entry should provide active event-loop monitor context before creating the winit window",
    );
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "fn runtime_window_attributes(",
            "WindowMonitorContext::for_event_loop(event_loop)",
            "runtime_window_attributes_with_monitor_context(descriptor, &monitor_context)",
        ],
        "runtime window attributes should seed monitor context from the active event loop",
    );
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "fn runtime_window_attributes_with_monitor_context",
            "runtime_window_position",
            "centered_window_position",
        ],
        "window attributes should use primary monitor context for centered placement",
    );
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "fn for_event_loop(event_loop: &dyn ActiveEventLoop)",
            "event_loop.primary_monitor()",
            "event_loop.available_monitors().collect::<Vec<_>>()",
        ],
        "window monitor context should capture primary and available monitors from winit",
    );
    assert!(
        window_attributes_builder_source.contains("runtime_window_attributes_with_primary_monitor"),
        "window attribute tests should keep a primary-monitor helper for monitor-independent coverage"
    );
    for required in [
        "mod builder;",
        "mod fullscreen;",
        "mod monitor;",
        "mod position;",
        "mod video_mode;",
        "pub(super) use builder::runtime_window_attributes;",
    ] {
        assert!(
            window_attributes_root_source.contains(required),
            "window attributes root should preserve structural wiring `{required}`"
        );
    }
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "WindowPosition::Centered",
            "monitor.position()?",
            "monitor.current_video_mode()?.size()",
            "saturating_i64_to_i32",
        ],
        "centered placement should derive a safe physical position from the selected monitor",
    );
    assert_source_order(
        window_attributes_source.as_str(),
        &[
            "WindowMode::BorderlessFullscreen",
            "WindowMode::BorderlessFullscreenOn(monitor)",
            "borderless_fullscreen_for_selection",
            "WindowMode::Fullscreen",
            "WindowMode::FullscreenOn",
            "Fullscreen::Exclusive(monitor, video_mode)",
            "Fullscreen::Borderless(monitor)",
        ],
        "fullscreen policy should prefer exclusive current-video-mode fullscreen and fall back to borderless",
    );
    for required in [
        "WindowMonitorSelection::Current",
        "WindowMonitorSelection::Primary",
        "WindowMonitorSelection::Index(index)",
        "WindowVideoModeSelection::Current",
        "WindowVideoModeSelection::Specific(requested)",
        "video_mode_matches",
    ] {
        assert!(
            window_attributes_source.contains(required),
            "monitor-aware window creation should preserve `{required}`"
        );
    }
}

#[test]
fn runtime_runner_forwards_session_profile_to_dynamic_runtime() {
    let runtime_runner_source = include_str!("../entry_runner/runtime.rs");
    let runtime_session_args_source = include_str!("../entry_runner/runtime_session_args.rs");
    let runtime_session_source = include_str!("../runtime_library/runtime_session.rs");

    assert!(
        runtime_session_args_source.contains("--runtime-session-profile"),
        "runtime runner should expose an explicit dynamic session profile argument"
    );
    assert!(
        runtime_session_args_source.contains("\"dev\"")
            && runtime_session_args_source.contains("\"minimal\"")
            && runtime_session_args_source.contains("\"headless\""),
        "runtime session profile parser should accept the dynamic runtime's named profiles"
    );
    assert!(
        runtime_session_args_source.contains("RUNTIME_SESSION_STARTUP_HELP")
            && runtime_session_args_source.contains("ZIRCON_RUNTIME_LIBRARY")
            && runtime_session_args_source.contains("ZIRCON_LOG_FILTER")
            && runtime_session_args_source.contains("ZIRCON_LOG")
            && runtime_session_args_source.contains("RUST_LOG")
            && runtime_session_args_source.contains("ZIRCON_LOG_LEVEL"),
        "runtime session profile parser should expose startup help for profiles, logging, and runtime library override"
    );
    assert_source_order(
        runtime_runner_source,
        &[
            "parse_diagnostic_log_startup_args(args)?",
            "parse_runtime_session_startup_args",
            "if runtime_session_args.help_requested",
            "return Ok(());",
            "LoadedRuntime::load_default()",
            "RuntimeSession::create_with_profile(runtime, runtime_session_args.profile.as_bytes())",
        ],
        "runtime runner should parse logging first, allow help before dynamic loading, then pass the selected session profile to the dynamic runtime",
    );
    assert!(
        runtime_session_source.contains("profile: ZrByteSlice::from_static(profile)"),
        "runtime session creation should pass the selected profile bytes through ZrRuntimeSessionConfigV1"
    );
}

#[test]
fn runtime_viewport_interaction_is_owned_by_dynamic_runtime_session() {
    let runtime_app_source = include_str!("../runtime_entry_app/mod.rs");
    let runtime_construct_source = include_str!("../runtime_entry_app/construct.rs");
    let runtime_handler_source = runtime_application_handler_source();

    assert!(
        runtime_app_source.contains("RuntimeSession"),
        "runtime entry app should own a runtime session wrapper"
    );
    assert!(
        !runtime_app_source.contains("mod camera_controller;"),
        "runtime camera control should live in zircon_runtime dynamic session state"
    );
    assert!(
        runtime_construct_source.contains("ZrRuntimeEventV1::viewport_resized"),
        "runtime entry construction should forward viewport changes through ABI events"
    );
    assert!(
        !runtime_app_source.contains("zircon_graphics::ViewportController"),
        "runtime entry app should not depend on zircon_graphics::ViewportController"
    );
    assert!(
        !runtime_construct_source
            .contains("zircon_graphics::{ViewportController, ViewportInput, ViewportState}"),
        "runtime construction should not import graphics viewport interaction types"
    );
    assert!(
        !runtime_handler_source.contains("use zircon_graphics::ViewportInput;"),
        "runtime window event handling should not import graphics viewport input types"
    );
}

#[test]
fn entry_subsystem_is_split_into_builtin_modules_run_modes_and_runtime_app_tree() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("entry");

    for relative in [
        "builtin_modules.rs",
        "entry_runner/mod.rs",
        "entry_runner/editor.rs",
        "entry_runner/runtime.rs",
        "entry_runner/headless.rs",
        "runtime_entry_app/application_handler/mod.rs",
        "runtime_entry_app/application_handler/hooks.rs",
        "runtime_entry_app/config/mod.rs",
        "runtime_entry_app/config/app_config.rs",
        "runtime_entry_app/converters/mod.rs",
        "runtime_entry_app/converters/abi.rs",
        "runtime_entry_app/converters/keyboard.rs",
        "runtime_entry_app/converters/pointer.rs",
        "runtime_entry_app/converters/window.rs",
        "runtime_entry_app/device_events/mod.rs",
        "runtime_entry_app/device_events/dispatch.rs",
        "runtime_entry_app/event_loop_policy/mod.rs",
        "runtime_entry_app/event_loop_policy/control_flow.rs",
        "runtime_entry_app/file_drag_drop/mod.rs",
        "runtime_entry_app/file_drag_drop/cancelled.rs",
        "runtime_entry_app/file_drag_drop/dropped.rs",
        "runtime_entry_app/file_drag_drop/hovered.rs",
        "runtime_entry_app/frame_loop.rs",
        "runtime_entry_app/host_requests/mod.rs",
        "runtime_entry_app/host_requests/drain.rs",
        "runtime_entry_app/host_requests/routing.rs",
        "runtime_entry_app/host_requests/ime/mod.rs",
        "runtime_entry_app/host_requests/ime/enable.rs",
        "runtime_entry_app/host_requests/ime/geometry.rs",
        "runtime_entry_app/host_requests/ime/request.rs",
        "runtime_entry_app/host_requests/ime/surrounding_text.rs",
        "runtime_entry_app/ime_input/mod.rs",
        "runtime_entry_app/ime_input/composition.rs",
        "runtime_entry_app/ime_input/deletion.rs",
        "runtime_entry_app/ime_input/lifecycle.rs",
        "runtime_entry_app/ime_input/routing.rs",
        "runtime_entry_app/keyboard_input/mod.rs",
        "runtime_entry_app/keyboard_input/event.rs",
        "runtime_entry_app/keyboard_input/payload.rs",
        "runtime_entry_app/mod.rs",
        "runtime_entry_app/pointer_input/mod.rs",
        "runtime_entry_app/pointer_input/button.rs",
        "runtime_entry_app/pointer_input/cursor.rs",
        "runtime_entry_app/pointer_input/device.rs",
        "runtime_entry_app/pointer_input/motion.rs",
        "runtime_entry_app/pointer_input/wheel.rs",
        "runtime_entry_app/surface_present/mod.rs",
        "runtime_entry_app/surface_present/binding.rs",
        "runtime_entry_app/surface_present/fallback.rs",
        "runtime_entry_app/surface_present/lifecycle.rs",
        "runtime_entry_app/surface_present/redraw.rs",
        "runtime_entry_app/surface_present/resize.rs",
        "runtime_entry_app/window_attributes/mod.rs",
        "runtime_entry_app/window_attributes/builder.rs",
        "runtime_entry_app/window_attributes/fullscreen.rs",
        "runtime_entry_app/window_attributes/monitor.rs",
        "runtime_entry_app/window_attributes/position.rs",
        "runtime_entry_app/window_attributes/video_mode.rs",
        "runtime_entry_app/window_creation.rs",
        "runtime_entry_app/window_events/mod.rs",
        "runtime_entry_app/window_events/dispatch.rs",
        "runtime_entry_app/window_lifecycle/mod.rs",
        "runtime_entry_app/window_lifecycle/close.rs",
        "runtime_entry_app/window_lifecycle/focus.rs",
        "runtime_entry_app/window_lifecycle/scale_factor.rs",
        "runtime_entry_app/window_lifecycle/status.rs",
        "runtime_entry_app/window_surface/mod.rs",
        "runtime_entry_app/window_surface/native_target.rs",
        "runtime_library/mod.rs",
        "runtime_library/loaded_runtime.rs",
        "runtime_library/runtime_session.rs",
        "tests/mod.rs",
        "tests/runtime_entry_device_guards.rs",
        "tests/runtime_entry_input_guards.rs",
        "tests/runtime_entry_source_guards.rs",
        "tests/runtime_entry_window_lifecycle_guards.rs",
        "tests/source_assertions.rs",
        "tests/profile_bootstrap.rs",
        "tests/builtin_engine_entry.rs",
    ] {
        assert!(
            root.join(relative).exists(),
            "expected entry module {relative} under {:?}",
            root
        );
    }
    assert!(
        !root.join("runtime_entry_app/application_handler.rs")
            .exists(),
        "runtime application handler should stay folder-backed instead of returning to an umbrella application_handler.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/window_attributes.rs").exists(),
        "runtime window attributes should stay folder-backed instead of returning to an umbrella window_attributes.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/pointer_input.rs").exists(),
        "runtime pointer input should stay folder-backed instead of returning to an umbrella pointer_input.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/ime_input.rs").exists(),
        "runtime IME input should stay folder-backed instead of returning to an umbrella ime_input.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/keyboard_input.rs").exists(),
        "runtime keyboard input should stay folder-backed instead of returning to an umbrella keyboard_input.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/host_requests.rs").exists(),
        "runtime host requests should stay folder-backed instead of returning to an umbrella host_requests.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/config.rs").exists(),
        "runtime config should stay folder-backed instead of returning to an umbrella config.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/surface_present.rs").exists(),
        "runtime surface present should stay folder-backed instead of returning to an umbrella surface_present.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/window_surface.rs").exists(),
        "runtime window surface should stay folder-backed instead of returning to an umbrella window_surface.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/event_loop_policy.rs").exists(),
        "runtime event-loop policy should stay folder-backed instead of returning to an umbrella event_loop_policy.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/file_drag_drop.rs").exists(),
        "runtime file drag/drop should stay folder-backed instead of returning to an umbrella file_drag_drop.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/window_lifecycle.rs").exists(),
        "runtime window lifecycle should stay folder-backed instead of returning to an umbrella window_lifecycle.rs file"
    );
    assert!(
        !root.join("runtime_entry_app/window_events.rs").exists(),
        "runtime window events should stay folder-backed instead of returning to an umbrella window_events.rs file"
    );
}

#[test]
fn entry_uses_runtime_owned_builtin_module_list_without_manual_graphics_insertion() {
    let builtin_modules_source = include_str!("../builtin_modules.rs");

    assert!(
        builtin_modules_source.contains("runtime_modules_for_target"),
        "entry bootstrap should source runtime modules through target-aware runtime loader"
    );
    for forbidden in [
        "use zircon_runtime::graphics::GraphicsModule;",
        "modules.insert(4, Arc::new(GraphicsModule));",
    ] {
        assert!(
            !builtin_modules_source.contains(forbidden),
            "entry builtin module bootstrap should stop keeping runtime-owned graphics registration detail `{forbidden}`"
        );
    }
}
