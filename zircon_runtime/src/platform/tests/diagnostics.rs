use super::super::*;

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
            "platform.gamepad_rumble=supported:gilrs_force_feedback",
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
