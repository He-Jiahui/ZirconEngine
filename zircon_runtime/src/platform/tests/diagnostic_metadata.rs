use super::super::*;

const TARGET_MODES: &[crate::RuntimeTargetMode] = &[
    crate::RuntimeTargetMode::ClientRuntime,
    crate::RuntimeTargetMode::ServerRuntime,
    crate::RuntimeTargetMode::EditorHost,
];

const EVENT_LOOP_POLICY_TOKENS: &[&str] =
    &["game", "desktop_app", "mobile", "continuous", "headless"];

fn target_mode_as_str(mode: crate::RuntimeTargetMode) -> &'static str {
    match mode {
        crate::RuntimeTargetMode::ClientRuntime => "client_runtime",
        crate::RuntimeTargetMode::ServerRuntime => "server_runtime",
        crate::RuntimeTargetMode::EditorHost => "editor_host",
    }
}

fn diagnostic_value<'a>(lines: &'a [String], key: &str) -> &'a str {
    let prefix = format!("{key}=");
    lines
        .iter()
        .find_map(|line| line.strip_prefix(&prefix))
        .unwrap_or_else(|| panic!("missing diagnostic key `{key}`"))
}

fn assert_plain_metadata_value(lines: &[String], key: &str, expected: &str) {
    let value = diagnostic_value(lines, key);

    assert_eq!(
        value, expected,
        "expected `{key}` to remain a plain metadata value"
    );
    assert!(
        !value.starts_with("supported:")
            && !value.starts_with("feature_disabled:")
            && !value.starts_with("unavailable:"),
        "`{key}` metadata value `{value}` should not use capability-status prefixes"
    );
    assert!(
        !value.contains(':'),
        "`{key}` metadata value `{value}` should not contain status-style separators"
    );
}

fn assert_report_metadata_lines_are_plain(report: PlatformCapabilityReport) {
    let lines = report.diagnostic_lines();

    assert_plain_metadata_value(&lines, "platform.target", report.target.as_str());
    assert_plain_metadata_value(
        &lines,
        "platform.target_mode",
        target_mode_as_str(report.target_mode),
    );

    let event_loop_policy = diagnostic_value(&lines, "platform.event_loop_policy");
    assert!(
        EVENT_LOOP_POLICY_TOKENS.contains(&event_loop_policy),
        "unexpected event-loop policy diagnostic token `{event_loop_policy}`"
    );
    assert!(
        !event_loop_policy.starts_with("supported:")
            && !event_loop_policy.starts_with("feature_disabled:")
            && !event_loop_policy.starts_with("unavailable:")
            && !event_loop_policy.contains(':'),
        "event-loop policy `{event_loop_policy}` should stay metadata, not capability status"
    );
}

#[test]
fn report_metadata_diagnostics_stay_plain_across_targets_and_modes() {
    for target in PlatformTarget::ALL {
        for target_mode in TARGET_MODES {
            assert_report_metadata_lines_are_plain(
                PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform())
                    .report(target, *target_mode),
            );
            assert_report_metadata_lines_are_plain(
                PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless())
                    .report(target, *target_mode),
            );
        }
    }
}

#[test]
fn explicit_event_loop_policy_diagnostic_stays_plain_metadata() {
    let matrix = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform());
    let report = matrix.report_with_event_loop_policy(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ClientRuntime,
        EventLoopPolicy::Continuous,
    );
    let lines = report.diagnostic_lines();

    assert_plain_metadata_value(&lines, "platform.target", "windows");
    assert_plain_metadata_value(&lines, "platform.target_mode", "client_runtime");
    assert_plain_metadata_value(&lines, "platform.event_loop_policy", "continuous");
}

#[test]
fn platform_config_enabled_diagnostic_stays_plain_boolean_metadata() {
    for enabled in [true, false] {
        let config = PlatformConfig {
            enabled,
            target: PlatformTarget::Headless,
            target_mode: crate::RuntimeTargetMode::ServerRuntime,
            features: PlatformFeatureSelection::headless(),
        };
        let lines = config.diagnostic_lines();
        let expected_enabled = if enabled { "true" } else { "false" };

        assert_eq!(lines[0], format!("platform.enabled={expected_enabled}"));
        assert_plain_metadata_value(&lines, "platform.enabled", expected_enabled);
        assert_plain_metadata_value(&lines, "platform.target", "headless");
        assert_plain_metadata_value(&lines, "platform.target_mode", "server_runtime");
        assert_plain_metadata_value(&lines, "platform.event_loop_policy", "headless");
    }
}
