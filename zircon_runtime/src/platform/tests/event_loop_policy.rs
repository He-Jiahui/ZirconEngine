use super::super::*;

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
