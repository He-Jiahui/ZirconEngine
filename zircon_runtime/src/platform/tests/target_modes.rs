use super::super::*;

#[test]
fn runtime_target_mode_diagnostic_tokens_stay_stable() {
    let cases = [
        (
            crate::RuntimeTargetMode::ClientRuntime,
            PlatformTarget::Windows,
            PlatformFeatureSelection::bevy_default_platform(),
            "platform.target_mode=client_runtime",
        ),
        (
            crate::RuntimeTargetMode::ServerRuntime,
            PlatformTarget::Linux,
            PlatformFeatureSelection::headless(),
            "platform.target_mode=server_runtime",
        ),
        (
            crate::RuntimeTargetMode::EditorHost,
            PlatformTarget::Linux,
            PlatformFeatureSelection::bevy_default_platform(),
            "platform.target_mode=editor_host",
        ),
    ];

    for (target_mode, target, features, expected_line) in cases {
        let report = PlatformCapabilityMatrix::new(features).report(target, target_mode);

        assert!(
            report
                .diagnostic_lines()
                .contains(&expected_line.to_string()),
            "expected stable target-mode diagnostic token `{expected_line}` for {target_mode:?}"
        );
    }
}

#[test]
fn runtime_target_modes_select_default_event_loop_policy() {
    let default_platform =
        PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform());

    let client = default_platform.report(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ClientRuntime,
    );
    assert_eq!(client.event_loop_policy, EventLoopPolicy::Game);
    assert_eq!(
        client.window_backend,
        CapabilityStatus::Supported(WindowBackend::Winit)
    );

    let editor =
        default_platform.report(PlatformTarget::Macos, crate::RuntimeTargetMode::EditorHost);
    assert_eq!(editor.event_loop_policy, EventLoopPolicy::DesktopApp);
    assert_eq!(
        editor.window_backend,
        CapabilityStatus::Supported(WindowBackend::Winit)
    );

    let server = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless()).report(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::ServerRuntime,
    );
    assert_eq!(server.event_loop_policy, EventLoopPolicy::Headless);
    assert_eq!(
        server.window_backend,
        CapabilityStatus::Supported(WindowBackend::Headless)
    );
}

#[test]
fn explicit_headless_policy_request_falls_back_to_windowed_defaults() {
    let matrix = PlatformCapabilityMatrix::new(PlatformFeatureSelection::bevy_default_platform());

    let client = matrix.report_with_event_loop_policy(
        PlatformTarget::Windows,
        crate::RuntimeTargetMode::ClientRuntime,
        EventLoopPolicy::Headless,
    );
    assert_eq!(client.event_loop_policy, EventLoopPolicy::Game);
    assert_eq!(
        client.window_backend,
        CapabilityStatus::Supported(WindowBackend::Winit)
    );

    let editor = matrix.report_with_event_loop_policy(
        PlatformTarget::Linux,
        crate::RuntimeTargetMode::EditorHost,
        EventLoopPolicy::Headless,
    );
    assert_eq!(editor.event_loop_policy, EventLoopPolicy::DesktopApp);
    assert_eq!(
        editor.window_backend,
        CapabilityStatus::Supported(WindowBackend::Winit)
    );
}

#[test]
fn server_runtime_forces_headless_policy_across_host_targets() {
    let matrix = PlatformCapabilityMatrix::new(PlatformFeatureSelection::headless());

    for target in [
        PlatformTarget::Windows,
        PlatformTarget::Linux,
        PlatformTarget::Macos,
        PlatformTarget::Android,
        PlatformTarget::Ios,
        PlatformTarget::WebGpu,
        PlatformTarget::Wasm,
        PlatformTarget::Headless,
    ] {
        let report = matrix.report_with_event_loop_policy(
            target,
            crate::RuntimeTargetMode::ServerRuntime,
            EventLoopPolicy::Continuous,
        );

        assert_eq!(
            report.event_loop_policy,
            EventLoopPolicy::Headless,
            "server runtime should stay headless for target {target:?}"
        );
        assert_eq!(
            report.window_backend,
            CapabilityStatus::Supported(WindowBackend::Headless),
            "server runtime should not report a physical window backend for target {target:?}"
        );
    }
}
