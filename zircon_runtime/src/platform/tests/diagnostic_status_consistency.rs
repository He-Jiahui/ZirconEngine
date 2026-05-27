use super::super::*;

fn diagnostic_value(report: &PlatformCapabilityReport, key: &str) -> String {
    let prefix = format!("{key}=");
    report
        .diagnostic_lines()
        .into_iter()
        .find_map(|line| line.strip_prefix(&prefix).map(str::to_owned))
        .unwrap_or_else(|| panic!("missing diagnostic key `{key}`"))
}

fn assert_status_prefix<T>(
    report: &PlatformCapabilityReport,
    key: &str,
    status: CapabilityStatus<T>,
) {
    let value = diagnostic_value(report, key);

    match status {
        CapabilityStatus::Supported(_) => assert!(
            value.starts_with("supported:"),
            "expected `{key}` diagnostic value `{value}` to use the supported prefix"
        ),
        CapabilityStatus::FeatureDisabled { feature } => assert_eq!(
            value,
            format!("feature_disabled:{feature}"),
            "expected `{key}` diagnostic value to match its disabled feature"
        ),
        CapabilityStatus::Unavailable { reason } => assert_eq!(
            value,
            format!("unavailable:{reason}"),
            "expected `{key}` diagnostic value to match its unavailable reason"
        ),
    }
}

fn assert_capability_diagnostic_status_prefixes(report: &PlatformCapabilityReport) {
    assert_status_prefix(report, "platform.window_backend", report.window_backend);
    assert_status_prefix(
        report,
        "platform.monitor_inventory",
        report.monitor_inventory,
    );
    assert_status_prefix(report, "platform.window_events", report.window_events);
    assert_status_prefix(report, "platform.window_lifecycle", report.window_lifecycle);
    assert_status_prefix(report, "platform.window_metrics", report.window_metrics);
    assert_status_prefix(report, "platform.ime", report.ime);
    assert_status_prefix(report, "platform.keyboard_events", report.keyboard_events);
    assert_status_prefix(report, "platform.cursor_boundary", report.cursor_boundary);
    assert_status_prefix(report, "platform.cursor_options", report.cursor_options);
    assert_status_prefix(report, "platform.mouse_buttons", report.mouse_buttons);
    assert_status_prefix(report, "platform.mouse_wheel", report.mouse_wheel);
    assert_status_prefix(report, "platform.touch_events", report.touch_events);
    assert_status_prefix(report, "platform.gesture_events", report.gesture_events);
    assert_status_prefix(report, "platform.pointer_position", report.pointer_position);
    assert_status_prefix(report, "platform.raw_mouse_motion", report.raw_mouse_motion);
    assert_status_prefix(report, "platform.mouse_input", report.mouse_input);
    assert_status_prefix(report, "platform.keyboard_input", report.keyboard_input);
    assert_status_prefix(report, "platform.touch_input", report.touch_input);
    assert_status_prefix(report, "platform.gesture_input", report.gesture_input);
    assert_status_prefix(report, "platform.gamepad_input", report.gamepad_input);
    assert_status_prefix(report, "platform.gamepad_events", report.gamepad_events);
    assert_status_prefix(report, "platform.gamepad_rumble", report.gamepad_rumble);
    assert_status_prefix(report, "platform.file_drag_drop", report.file_drag_drop);
    assert_status_prefix(report, "platform.linux_x11", report.linux_x11);
    assert_status_prefix(report, "platform.linux_wayland", report.linux_wayland);
}

#[test]
fn default_desktop_diagnostics_match_report_statuses() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Windows,
            crate::RuntimeTargetMode::ClientRuntime,
        );

    assert_capability_diagnostic_status_prefixes(&report);
    assert_eq!(
        diagnostic_value(&report, "platform.window_backend"),
        "supported:winit"
    );
    assert_eq!(
        diagnostic_value(&report, "platform.gesture_events"),
        "feature_disabled:input-gestures"
    );
    assert_eq!(
        diagnostic_value(&report, "platform.gamepad_rumble"),
        "supported:gilrs_force_feedback"
    );
    assert_eq!(
        diagnostic_value(&report, "platform.linux_x11"),
        "unavailable:protocol is linux-specific"
    );
}

#[test]
fn headless_fixture_diagnostics_match_disabled_and_unavailable_statuses() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless()).report(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::ServerRuntime,
    );

    assert_capability_diagnostic_status_prefixes(&report);
    assert_eq!(
        diagnostic_value(&report, "platform.window_backend"),
        "supported:headless"
    );
    assert_eq!(
        diagnostic_value(&report, "platform.monitor_inventory"),
        "unavailable:headless target has no monitor inventory backend"
    );
    assert_eq!(
        diagnostic_value(&report, "platform.mouse_input"),
        "feature_disabled:input-mouse"
    );
    assert_eq!(
        diagnostic_value(&report, "platform.gamepad_input"),
        "feature_disabled:input-gamepad"
    );
    assert_eq!(
        diagnostic_value(&report, "platform.linux_x11"),
        "feature_disabled:platform-x11"
    );
}

#[test]
fn browser_gamepad_gate_diagnostics_follow_status_transitions() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.gamepad_browser = false;

    let disabled = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::WebGpu,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_capability_diagnostic_status_prefixes(&disabled);
    assert_eq!(
        diagnostic_value(&disabled, "platform.window_backend"),
        "supported:browser_canvas"
    );
    assert_eq!(
        diagnostic_value(&disabled, "platform.mouse_input"),
        "supported:browser_events"
    );
    assert_eq!(
        diagnostic_value(&disabled, "platform.gamepad_input"),
        "feature_disabled:gamepad-browser"
    );
    assert_eq!(
        diagnostic_value(&disabled, "platform.gamepad_events"),
        "feature_disabled:gamepad-browser"
    );
    assert_eq!(
        diagnostic_value(&disabled, "platform.gamepad_rumble"),
        "feature_disabled:gamepad-browser"
    );

    features.gamepad_browser = true;
    let enabled = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::WebGpu,
        crate::RuntimeTargetMode::ClientRuntime,
    );

    assert_capability_diagnostic_status_prefixes(&enabled);
    assert_eq!(
        diagnostic_value(&enabled, "platform.gamepad_input"),
        "supported:browser_gamepad_api"
    );
    assert_eq!(
        diagnostic_value(&enabled, "platform.gamepad_events"),
        "supported:browser_gamepad_api_polling"
    );
    assert_eq!(
        diagnostic_value(&enabled, "platform.gamepad_rumble"),
        "unavailable:browser gamepad rumble host backend is not implemented yet"
    );
}
