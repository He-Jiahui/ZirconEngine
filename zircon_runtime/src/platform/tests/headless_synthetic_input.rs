use super::super::*;

fn assert_diagnostic_line(report: &PlatformCapabilityReport, expected: &str) {
    let lines = report.diagnostic_lines();

    assert!(
        lines.iter().any(|line| line == expected),
        "expected diagnostic line `{expected}` in {lines:?}"
    );
}

#[test]
fn headless_target_with_input_features_reports_synthetic_input_only() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
        .report(
            PlatformTarget::Headless,
            crate::RuntimeTargetMode::ClientRuntime,
        );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::FeatureDisabled {
            feature: "platform-headless"
        }
    );
    assert_eq!(report.event_loop_policy, EventLoopPolicy::Headless);
    assert_eq!(
        report.mouse_input,
        CapabilityStatus::Supported(InputBackend::SyntheticOnly)
    );
    assert_eq!(
        report.keyboard_input,
        CapabilityStatus::Supported(InputBackend::SyntheticOnly)
    );
    assert_eq!(
        report.touch_input,
        CapabilityStatus::Supported(InputBackend::SyntheticOnly)
    );
    assert_eq!(
        report.gesture_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        report.mouse_buttons,
        CapabilityStatus::Unavailable {
            reason: "headless target has no mouse button host backend"
        }
    );
    assert_eq!(
        report.raw_mouse_motion,
        CapabilityStatus::Unavailable {
            reason: "headless target has no raw mouse motion host backend"
        }
    );
    assert_eq!(
        report.gamepad_input,
        CapabilityStatus::Unavailable {
            reason: "headless target has no physical gamepad backend"
        }
    );
    assert_diagnostic_line(&report, "platform.mouse_input=supported:synthetic_only");
    assert_diagnostic_line(&report, "platform.keyboard_input=supported:synthetic_only");
    assert_diagnostic_line(&report, "platform.touch_input=supported:synthetic_only");
    assert_diagnostic_line(
        &report,
        "platform.gamepad_input=unavailable:headless target has no physical gamepad backend",
    );
}

#[test]
fn server_runtime_with_input_features_keeps_headless_window_and_synthetic_input() {
    let mut features = PlatformFeatureSelection::bevy_default_platform();
    features.platform_headless = true;
    features.input_gestures = true;

    let report = PlatformCapabilityMatrix::new(features).report(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ServerRuntime,
    );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Headless)
    );
    assert_eq!(report.event_loop_policy, EventLoopPolicy::Headless);
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
        report.mouse_input,
        CapabilityStatus::Supported(InputBackend::SyntheticOnly)
    );
    assert_eq!(
        report.keyboard_input,
        CapabilityStatus::Supported(InputBackend::SyntheticOnly)
    );
    assert_eq!(
        report.touch_input,
        CapabilityStatus::Supported(InputBackend::SyntheticOnly)
    );
    assert_eq!(
        report.gesture_input,
        CapabilityStatus::Supported(InputBackend::SyntheticOnly)
    );
    assert_eq!(
        report.gamepad_events,
        CapabilityStatus::Unavailable {
            reason: "headless target has no physical gamepad event backend"
        }
    );
    assert_eq!(
        report.file_drag_drop,
        CapabilityStatus::Unavailable {
            reason: "headless target has no file drag/drop host backend"
        }
    );
    assert_diagnostic_line(&report, "platform.window_backend=supported:headless");
    assert_diagnostic_line(&report, "platform.gesture_input=supported:synthetic_only");
    assert_diagnostic_line(&report, "platform.event_loop_policy=headless");
}

#[test]
fn headless_fixture_disables_synthetic_input_sources() {
    let report = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless()).report(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::ServerRuntime,
    );

    assert_eq!(
        report.window_backend,
        CapabilityStatus::Supported(WindowBackend::Headless)
    );
    assert_eq!(
        report.mouse_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-mouse"
        }
    );
    assert_eq!(
        report.keyboard_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-keyboard"
        }
    );
    assert_eq!(
        report.touch_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-touch"
        }
    );
    assert_eq!(
        report.gesture_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gestures"
        }
    );
    assert_eq!(
        report.gamepad_input,
        CapabilityStatus::FeatureDisabled {
            feature: "input-gamepad"
        }
    );
    assert_diagnostic_line(&report, "platform.mouse_input=feature_disabled:input-mouse");
    assert_diagnostic_line(
        &report,
        "platform.keyboard_input=feature_disabled:input-keyboard",
    );
    assert_diagnostic_line(&report, "platform.touch_input=feature_disabled:input-touch");
    assert_diagnostic_line(
        &report,
        "platform.gesture_input=feature_disabled:input-gestures",
    );
}
