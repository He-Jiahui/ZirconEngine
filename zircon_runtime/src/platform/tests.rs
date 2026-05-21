use super::*;

#[test]
fn platform_root_stays_structural_after_module_split() {
    let source = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("src")
            .join("platform")
            .join("mod.rs"),
    )
    .expect("platform mod source");

    for forbidden in [
        "pub struct PlatformConfig",
        "pub struct PlatformModule",
        "pub struct PlatformDriver",
        "pub struct PlatformManager",
        "pub fn module_descriptor(",
        "impl EngineModule for PlatformModule",
    ] {
        assert!(
            !source.contains(forbidden),
            "expected platform/mod.rs to stay structural after split, found `{forbidden}`"
        );
    }
}

#[test]
fn client_desktop_default_platform_declares_window_input_and_gilrs() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Windows,
            crate::RuntimeTargetMode::ClientRuntime,
        );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Winit)
    );
    assert_eq!(
        report.monitor_inventory,
        CapabilityStatus::Supported(MonitorBackend::WinitMonitorHandles)
    );
    assert_eq!(
        report.window_events,
        CapabilityStatus::Supported(WindowEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.window_lifecycle,
        CapabilityStatus::Supported(WindowLifecycleBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.window_metrics,
        CapabilityStatus::Supported(WindowMetricsBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.ime,
        CapabilityStatus::Supported(ImeBackend::WinitIme)
    );
    assert_eq!(
        report.keyboard_events,
        CapabilityStatus::Supported(KeyboardEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.cursor_boundary,
        CapabilityStatus::Supported(CursorBoundaryBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "desktop cursor options host-request backend is not implemented yet"
        }
    );
    assert_eq!(
        report.mouse_buttons,
        CapabilityStatus::Supported(MouseButtonBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.mouse_wheel,
        CapabilityStatus::Supported(MouseWheelBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.touch_events,
        CapabilityStatus::Supported(TouchEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        report.pointer_position,
        CapabilityStatus::Supported(PointerPositionBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.raw_mouse_motion,
        CapabilityStatus::Supported(RawMouseMotionBackend::WinitDeviceEvents)
    );
    assert_eq!(report.event_loop_policy, EventLoopPolicy::Game);
    assert_eq!(
        report.mouse_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.keyboard_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.touch_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        report.gamepad_input,
        CapabilityStatus::Supported(GamepadBackend::Gilrs)
    );
    assert_eq!(
        report.gamepad_events,
        CapabilityStatus::Supported(GamepadEventBackend::GilrsEventPolling)
    );
    assert_eq!(
        report.gamepad_rumble,
        CapabilityStatus::Unavailable {
            reason: "desktop gamepad rumble host backend is not implemented yet"
        }
    );
    assert_eq!(
        report.file_drag_drop,
        CapabilityStatus::Supported(FileDragDropBackend::WinitWindowEvents)
    );
}

#[test]
fn editor_host_uses_desktop_app_event_policy() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(PlatformTarget::Macos, crate::RuntimeTargetMode::EditorHost);

    assert_eq!(report.event_loop_policy, EventLoopPolicy::DesktopApp);
    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Winit)
    );
}

#[test]
fn explicit_continuous_event_policy_is_reported_for_windowed_targets() {
    let matrix = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform());

    let report = matrix.report_with_event_loop_policy(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ClientRuntime,
        EventLoopPolicy::Continuous,
    );

    assert_eq!(report.event_loop_policy, EventLoopPolicy::Continuous);
    assert!(report
        .diagnostic_lines()
        .contains(&"platform.event_loop_policy=continuous".to_string()));
}

#[test]
fn explicit_event_policy_does_not_override_headless_topology() {
    let matrix = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless());

    let report = matrix.report_with_event_loop_policy(
        PlatformTarget::Headless,
        crate::RuntimeTargetMode::ServerRuntime,
        EventLoopPolicy::Continuous,
    );

    assert_eq!(report.event_loop_policy, EventLoopPolicy::Headless);
    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Headless)
    );
}

#[test]
fn server_runtime_stays_headless() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless()).report(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::ServerRuntime,
    );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Headless)
    );
    assert_eq!(
        report.monitor_inventory,
        CapabilityStatus::Unavailable {
            reason: "headless target has no monitor inventory backend"
        }
    );
    assert_eq!(
        report.window_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no window event host backend"
        }
    );
    assert_eq!(
        report.window_lifecycle,
        CapabilityStatus::Unavailable {
            reason: "headless target has no window lifecycle host backend"
        }
    );
    assert_eq!(
        report.window_metrics,
        CapabilityStatus::Unavailable {
            reason: "headless target has no window metrics host backend"
        }
    );
    assert_eq!(
        report.ime,
        CapabilityStatus::Unavailable {
            reason: "headless target has no ime host backend"
        }
    );
    assert_eq!(
        report.keyboard_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no keyboard event host backend"
        }
    );
    assert_eq!(
        report.cursor_boundary,
        CapabilityStatus::Unavailable {
            reason: "headless target has no cursor boundary host backend"
        }
    );
    assert_eq!(
        report.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "headless target has no cursor options host backend"
        }
    );
    assert_eq!(
        report.mouse_buttons,
        CapabilityStatus::Unavailable {
            reason: "headless target has no mouse button host backend"
        }
    );
    assert_eq!(
        report.mouse_wheel,
        CapabilityStatus::Unavailable {
            reason: "headless target has no mouse wheel host backend"
        }
    );
    assert_eq!(
        report.touch_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no touch event host backend"
        }
    );
    assert_eq!(
        report.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no gesture event host backend"
        }
    );
    assert_eq!(
        report.pointer_position,
        CapabilityStatus::Unavailable {
            reason: "headless target has no pointer position host backend"
        }
    );
    assert_eq!(
        report.raw_mouse_motion,
        CapabilityStatus::Unavailable {
            reason: "headless target has no raw mouse motion host backend"
        }
    );
    assert_eq!(report.event_loop_policy, EventLoopPolicy::Headless);
    assert_eq!(
        report.mouse_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-mouse"
        }
    );
    assert_eq!(
        report.gamepad_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gamepad"
        }
    );
    assert_eq!(
        report.gamepad_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gamepad"
        }
    );
    assert_eq!(
        report.gamepad_rumble,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gamepad"
        }
    );
    assert_eq!(
        report.file_drag_drop,
        CapabilityStatus::Unavailable {
            reason: "headless target has no file drag/drop host backend"
        }
    );
}

#[test]
fn linux_protocols_are_declared_independently() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.platform_wayland = false;

    let report = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        report.linux_x11,
        CapabilityStatus::Supported(LinuxWindowProtocol::X11)
    );
    assert_eq!(
        report.linux_wayland,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-wayland"
        }
    );
}

#[test]
fn capability_reports_format_stable_diagnostic_lines() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.platform_wayland = false;

    let report = PlatformCapabilityMatrix::new(features)
        .report(PlatformTarget::Linux, crate::RuntimeTargetMode::EditorHost);

    assert_eq!(
        report.diagnostic_lines(),
        vec![
            "platform.target=linux",
            "platform.target_mode=editor_host",
            "platform.window_backend=supported:winit",
            "platform.monitor_inventory=supported:winit_monitor_handles",
            "platform.window_events=supported:winit_window_events",
            "platform.window_lifecycle=supported:winit_window_events",
            "platform.window_metrics=supported:winit_window_events",
            "platform.ime=supported:winit_ime",
            "platform.keyboard_events=supported:winit_window_events",
            "platform.cursor_boundary=supported:winit_window_events",
            "platform.cursor_options=unavailable:desktop cursor options host-request backend is not implemented yet",
            "platform.mouse_buttons=supported:winit_window_events",
            "platform.mouse_wheel=supported:winit_window_events",
            "platform.touch_events=supported:winit_window_events",
            "platform.gesture_events=feature_disabled:input-gestures",
            "platform.pointer_position=supported:winit_window_events",
            "platform.raw_mouse_motion=supported:winit_device_events",
            "platform.event_loop_policy=desktop_app",
            "platform.mouse_input=supported:winit_window_events",
            "platform.keyboard_input=supported:winit_window_events",
            "platform.touch_input=supported:winit_window_events",
            "platform.gesture_input=feature_disabled:input-gestures",
            "platform.gamepad_input=supported:gilrs",
            "platform.gamepad_events=supported:gilrs_event_polling",
            "platform.gamepad_rumble=unavailable:desktop gamepad rumble host backend is not implemented yet",
            "platform.file_drag_drop=supported:winit_window_events",
            "platform.linux_x11=supported:x11",
            "platform.linux_wayland=feature_disabled:platform-wayland",
        ]
    );
    assert!(report
        .format_diagnostics()
        .contains("platform.event_loop_policy=desktop_app"));

    assert_eq!(EventLoopPolicy::Continuous.as_str(), "continuous");
}

#[test]
fn platform_config_diagnostics_include_enabled_policy() {
    let config = PlatformConfig {
        enabled: false,
        target: PlatformTarget::Headless,
        target_mode: crate::RuntimeTargetMode::ServerRuntime,
        features: PlatformFeatureSelection::headless(),
    };

    let diagnostics = config.format_diagnostics();

    assert!(diagnostics.contains("platform.enabled=false"));
    assert!(diagnostics.contains("platform.target=headless"));
    assert!(diagnostics.contains("platform.window_backend=supported:headless"));
}

#[test]
fn mobile_and_browser_capabilities_are_explicit() {
    let mobile = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Android,
            crate::RuntimeTargetMode::ClientRuntime,
        );

    assert_eq!(mobile.event_loop_policy, EventLoopPolicy::Mobile);
    assert_eq!(
        mobile.touch_input,
        CapabilityStatus::Supported(InputBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.monitor_inventory,
        CapabilityStatus::Supported(MonitorBackend::WinitMonitorHandles)
    );
    assert_eq!(
        mobile.window_events,
        CapabilityStatus::Supported(WindowEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.window_lifecycle,
        CapabilityStatus::Supported(WindowLifecycleBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.window_metrics,
        CapabilityStatus::Supported(WindowMetricsBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.ime,
        CapabilityStatus::Unavailable {
            reason: "mobile ime host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.keyboard_events,
        CapabilityStatus::Supported(KeyboardEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.cursor_boundary,
        CapabilityStatus::Supported(CursorBoundaryBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "mobile cursor options host-request backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.mouse_buttons,
        CapabilityStatus::Supported(MouseButtonBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.mouse_wheel,
        CapabilityStatus::Supported(MouseWheelBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.touch_events,
        CapabilityStatus::Supported(TouchEventBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        mobile.pointer_position,
        CapabilityStatus::Supported(PointerPositionBackend::WinitWindowEvents)
    );
    assert_eq!(
        mobile.raw_mouse_motion,
        CapabilityStatus::Unavailable {
            reason: "mobile raw mouse motion host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.gamepad_input,
        CapabilityStatus::Unavailable {
            reason: "mobile gamepad host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.gamepad_events,
        CapabilityStatus::Unavailable {
            reason: "mobile gamepad event host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.gamepad_rumble,
        CapabilityStatus::Unavailable {
            reason: "mobile gamepad rumble host backend is not implemented yet"
        }
    );
    assert_eq!(
        mobile.file_drag_drop,
        CapabilityStatus::Unavailable {
            reason: "mobile file drag/drop host backend is not implemented yet"
        }
    );

    let browser = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Wasm,
            crate::RuntimeTargetMode::ClientRuntime,
        );

    assert_eq!(
        browser.window_backend,
        CapabilityStatus::Supported(WindowBackend::BrowserCanvas)
    );
    assert_eq!(
        browser.monitor_inventory,
        CapabilityStatus::Unavailable {
            reason: "browser monitor inventory host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.window_events,
        CapabilityStatus::Unavailable {
            reason: "browser window event host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.window_lifecycle,
        CapabilityStatus::Unavailable {
            reason: "browser window lifecycle host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.window_metrics,
        CapabilityStatus::Unavailable {
            reason: "browser window metrics host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.ime,
        CapabilityStatus::Unavailable {
            reason: "browser ime host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.keyboard_events,
        CapabilityStatus::Unavailable {
            reason: "browser keyboard event host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.cursor_boundary,
        CapabilityStatus::Unavailable {
            reason: "browser cursor boundary host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.cursor_options,
        CapabilityStatus::Unavailable {
            reason: "browser cursor options host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.mouse_buttons,
        CapabilityStatus::Unavailable {
            reason: "browser mouse button host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.mouse_wheel,
        CapabilityStatus::Unavailable {
            reason: "browser mouse wheel host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.touch_events,
        CapabilityStatus::Unavailable {
            reason: "browser touch event host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        browser.pointer_position,
        CapabilityStatus::Unavailable {
            reason: "browser pointer position host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.raw_mouse_motion,
        CapabilityStatus::Unavailable {
            reason: "browser raw mouse motion host backend is not implemented yet"
        }
    );
    assert_eq!(
        browser.gamepad_input,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        browser.gamepad_events,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        browser.gamepad_rumble,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        browser.file_drag_drop,
        CapabilityStatus::Unavailable {
            reason: "browser file drag/drop host backend is not implemented yet"
        }
    );
}

#[test]
fn gesture_event_capabilities_declare_feature_gate_and_missing_host_paths() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();

    let default_desktop = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Macos,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        default_desktop.gesture_events,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );

    features.input_gestures = true;

    let macos = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Macos,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        macos.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "winit gesture event host backend is not implemented yet"
        }
    );

    let ios = PlatformCapabilityMatrix::new(features)
        .report(PlatformTarget::Ios, crate::RuntimeTargetMode::ClientRuntime);
    assert_eq!(
        ios.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "winit gesture event host backend is not implemented yet"
        }
    );

    let windows = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        windows.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "winit gesture events are only declared for macOS and iOS targets"
        }
    );

    let browser = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Wasm,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(
        browser.gesture_events,
        CapabilityStatus::Unavailable {
            reason: "browser gesture event host backend is not implemented yet"
        }
    );
}

#[test]
fn browser_gamepad_backend_is_separate_from_gilrs() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.gamepad_gilrs = true;
    features.gamepad_browser = false;

    let without_browser_backend = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::WebGpu,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        without_browser_backend.gamepad_input,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        without_browser_backend.gamepad_events,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );
    assert_eq!(
        without_browser_backend.gamepad_rumble,
        CapabilityStatus::FeatureDisabled {
            feature: "gamepad-browser"
        }
    );

    features.gamepad_browser = true;
    let with_browser_backend = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::WebGpu,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        with_browser_backend.gamepad_input,
        CapabilityStatus::Supported(GamepadBackend::BrowserGamepadApi)
    );
    assert_eq!(
        with_browser_backend.gamepad_events,
        CapabilityStatus::Supported(GamepadEventBackend::BrowserGamepadApiPolling)
    );
    assert_eq!(
        with_browser_backend.gamepad_rumble,
        CapabilityStatus::Unavailable {
            reason: "browser gamepad rumble host backend is not implemented yet"
        }
    );
}
