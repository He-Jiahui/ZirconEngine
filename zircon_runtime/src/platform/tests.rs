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
            "platform.event_loop_policy=desktop_app",
            "platform.mouse_input=supported:winit_window_events",
            "platform.keyboard_input=supported:winit_window_events",
            "platform.touch_input=supported:winit_window_events",
            "platform.gesture_input=feature_disabled:input-gestures",
            "platform.gamepad_input=supported:gilrs",
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
        mobile.gamepad_input,
        CapabilityStatus::Unavailable {
            reason: "mobile gamepad host backend is not implemented yet"
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
        browser.gamepad_input,
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

    features.gamepad_browser = true;
    let with_browser_backend = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::WebGpu,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_eq!(
        with_browser_backend.gamepad_input,
        CapabilityStatus::Supported(GamepadBackend::BrowserGamepadApi)
    );
}
