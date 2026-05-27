use super::super::*;

const EXPECTED_REPORT_KEYS: &[&str] = &[
    "platform.target",
    "platform.target_mode",
    "platform.window_backend",
    "platform.monitor_inventory",
    "platform.window_events",
    "platform.window_lifecycle",
    "platform.window_metrics",
    "platform.ime",
    "platform.keyboard_events",
    "platform.cursor_boundary",
    "platform.cursor_options",
    "platform.mouse_buttons",
    "platform.mouse_wheel",
    "platform.touch_events",
    "platform.gesture_events",
    "platform.pointer_position",
    "platform.raw_mouse_motion",
    "platform.event_loop_policy",
    "platform.mouse_input",
    "platform.keyboard_input",
    "platform.touch_input",
    "platform.gesture_input",
    "platform.gamepad_input",
    "platform.gamepad_events",
    "platform.gamepad_rumble",
    "platform.file_drag_drop",
    "platform.linux_x11",
    "platform.linux_wayland",
];

fn diagnostic_keys(lines: &[String]) -> Vec<&str> {
    lines
        .iter()
        .map(|line| {
            line.split_once('=')
                .unwrap_or_else(|| panic!("diagnostic line should contain `=`: {line}"))
                .0
        })
        .collect()
}

#[test]
fn capability_report_diagnostic_keys_stay_ordered_and_complete() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Linux,
            crate::RuntimeTargetMode::ClientRuntime,
        );
    let lines = report.diagnostic_lines();

    assert_eq!(diagnostic_keys(&lines), EXPECTED_REPORT_KEYS);
    assert_eq!(
        lines.len(),
        EXPECTED_REPORT_KEYS.len(),
        "each platform report key should produce exactly one diagnostic line"
    );
}

#[test]
fn platform_config_diagnostics_prepend_enabled_before_capability_keys() {
    let config = PlatformConfig {
        enabled: true,
        target: PlatformTarget::Windows,
        target_mode: crate::RuntimeTargetMode::ClientRuntime,
        features: PlatformFeatureSelection::bevy_default_platform(),
    };
    let lines = config.diagnostic_lines();
    let mut expected = vec!["platform.enabled"];
    expected.extend_from_slice(EXPECTED_REPORT_KEYS);

    assert_eq!(diagnostic_keys(&lines), expected);
}

#[test]
fn diagnostic_key_set_is_consistent_across_windowed_browser_mobile_and_headless_targets() {
    let mut browser_features = PlatformFeatureSelection::bevy_default_platform();
    browser_features.gamepad_browser = true;

    let cases = [
        (
            PlatformFeatureSelection::bevy_default_platform(),
            PlatformTarget::Windows,
            crate::RuntimeTargetMode::ClientRuntime,
        ),
        (
            browser_features,
            PlatformTarget::WebGpu,
            crate::RuntimeTargetMode::ClientRuntime,
        ),
        (
            PlatformFeatureSelection::bevy_default_platform(),
            PlatformTarget::Android,
            crate::RuntimeTargetMode::ClientRuntime,
        ),
        (
            PlatformFeatureSelection::headless(),
            PlatformTarget::Headless,
            crate::RuntimeTargetMode::ServerRuntime,
        ),
    ];

    for (features, target, target_mode) in cases {
        let report = PlatformCapabilityMatrix::new(features).report(target, target_mode);

        assert_eq!(
            diagnostic_keys(&report.diagnostic_lines()),
            EXPECTED_REPORT_KEYS,
            "diagnostic keys should not vary by target {:?}",
            target
        );
    }
}
