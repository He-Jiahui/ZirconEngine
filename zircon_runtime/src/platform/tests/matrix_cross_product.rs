use std::collections::HashSet;

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

const TARGET_MODES: &[crate::RuntimeTargetMode] = &[
    crate::RuntimeTargetMode::ClientRuntime,
    crate::RuntimeTargetMode::ServerRuntime,
    crate::RuntimeTargetMode::EditorHost,
];

fn target_mode_as_str(mode: crate::RuntimeTargetMode) -> &'static str {
    match mode {
        crate::RuntimeTargetMode::ClientRuntime => "client_runtime",
        crate::RuntimeTargetMode::ServerRuntime => "server_runtime",
        crate::RuntimeTargetMode::EditorHost => "editor_host",
    }
}

fn diagnostic_key(line: &str) -> &str {
    line.split_once('=')
        .unwrap_or_else(|| panic!("diagnostic line should contain `=`: {line}"))
        .0
}

fn assert_report_diagnostic_surface(report: PlatformCapabilityReport) {
    let lines = report.diagnostic_lines();
    let keys: Vec<&str> = lines.iter().map(|line| diagnostic_key(line)).collect();
    let unique_keys: HashSet<&str> = keys.iter().copied().collect();

    assert_eq!(keys, EXPECTED_REPORT_KEYS);
    assert_eq!(
        keys.len(),
        unique_keys.len(),
        "platform diagnostics should not emit duplicate keys for {:?} / {:?}",
        report.target,
        report.target_mode
    );
    assert!(lines.iter().all(|line| line.starts_with("platform.")));
    assert!(lines.contains(&format!("platform.target={}", report.target.as_str())));
    assert!(lines.contains(&format!(
        "platform.target_mode={}",
        target_mode_as_str(report.target_mode)
    )));
    assert!(lines
        .iter()
        .any(|line| line.starts_with("platform.event_loop_policy=")));
    assert!(!lines.iter().any(|line| line.contains("Supported(")));
    assert!(!lines.iter().any(|line| line.contains("FeatureDisabled")));
    assert!(!lines.iter().any(|line| line.contains("Unavailable")));
}

#[test]
fn default_platform_reports_have_complete_diagnostics_for_all_targets_and_modes() {
    let matrix = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform());

    for target in PlatformTarget::ALL {
        for target_mode in TARGET_MODES {
            assert_report_diagnostic_surface(matrix.report(target, *target_mode));
        }
    }
}

#[test]
fn headless_reports_have_complete_diagnostics_for_all_targets_and_modes() {
    let matrix = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless());

    for target in PlatformTarget::ALL {
        for target_mode in TARGET_MODES {
            assert_report_diagnostic_surface(matrix.report(target, *target_mode));
        }
    }
}

#[test]
fn explicit_continuous_policy_reports_keep_the_same_diagnostic_surface() {
    let matrix = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform());

    for target in PlatformTarget::ALL {
        for target_mode in TARGET_MODES {
            assert_report_diagnostic_surface(matrix.report_with_event_loop_policy(
                target,
                *target_mode,
                EventLoopPolicy::Continuous,
            ));
        }
    }
}
