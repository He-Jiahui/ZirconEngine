use super::super::source_assertions::assert_source_order;
use super::sources::{entry_root, runtime_config_source};

#[test]
fn runtime_runner_projects_session_profile_into_app_host_config() {
    let runtime_app_source = include_str!("../../runtime_entry_app/mod.rs");
    let runtime_config_root_source = include_str!("../../runtime_entry_app/config/mod.rs");
    let runtime_config_app_config_source =
        include_str!("../../runtime_entry_app/config/app_config.rs");
    let runtime_config_source = runtime_config_source();
    let runtime_window_creation_source = include_str!("../../runtime_entry_app/window_creation.rs");
    let runtime_construct_source = include_str!("../../runtime_entry_app/construct.rs");
    let runtime_runner_source = include_str!("../../entry_runner/runtime.rs");
    let root = entry_root();

    assert!(
        runtime_app_source.contains("mod config;")
            && runtime_app_source.contains("RuntimeEntryAppConfig"),
        "runtime entry app should keep host configuration in a child module"
    );
    assert!(
        runtime_config_root_source.contains("mod app_config;")
            && runtime_config_root_source
                .contains("pub(in crate::entry) use app_config::RuntimeEntryAppConfig;"),
        "runtime config root should stay structural and expose the host config type"
    );
    assert!(
        !root.join("runtime_entry_app/config.rs").exists(),
        "runtime config should stay folder-backed instead of returning to an umbrella config.rs file"
    );
    assert!(
        runtime_config_source.contains("WindowDescriptor")
            && runtime_config_source.contains("EventLoopPolicy")
            && runtime_config_source.contains("WindowLifecyclePolicy"),
        "runtime entry app config should carry the neutral window descriptor, event-loop policy, and lifecycle policy"
    );
    assert!(
        runtime_config_source.contains("with_window_lifecycle_policy")
            && runtime_config_source.contains("with_close_when_requested")
            && runtime_config_source.contains("window_lifecycle_policy(&self)"),
        "runtime entry app config should expose the Bevy-style close/exit host policy"
    );
    assert_source_order(
        runtime_config_app_config_source,
        &[
            "struct RuntimeEntryAppConfig",
            "window_descriptor: WindowDescriptor",
            "event_loop_policy: EventLoopPolicy",
            "window_lifecycle_policy: WindowLifecyclePolicy",
            "fn with_window_descriptor",
            "fn with_event_loop_policy",
            "fn with_window_lifecycle_policy",
            "impl Default for RuntimeEntryAppConfig",
            "EventLoopPolicy::Game",
        ],
        "runtime app-config implementation should keep host policy fields, builder methods, and defaults source-visible",
    );
    assert!(
        runtime_construct_source.contains("RuntimeEntryAppConfig")
            && runtime_construct_source.contains("config.window_descriptor")
            && runtime_construct_source.contains("config.event_loop_policy")
            && runtime_construct_source.contains("config.window_lifecycle_policy"),
        "runtime entry construction should seed host state from RuntimeEntryAppConfig"
    );
    assert!(
        runtime_app_source.contains("window_lifecycle_policy: WindowLifecyclePolicy"),
        "runtime entry construction should store close/exit policy from RuntimeEntryAppConfig"
    );
    assert!(
        runtime_window_creation_source.contains("self.window_descriptor.primary_window.is_none()"),
        "runtime entry should skip concrete winit window creation when the host config has no primary window"
    );
    assert_source_order(
        runtime_runner_source,
        &[
            "parse_runtime_session_startup_args",
            "RuntimeSession::create_with_profile(runtime, runtime_session_args.profile.as_bytes())",
            "runtime_entry_app_config_for_session_profile(runtime_session_args.profile)",
            "RuntimeEntryApp::new(session, host_config)",
        ],
        "runtime runner should derive the app host config from the already-parsed session profile before creating the app",
    );
    for required in [
        "RuntimeSessionProfile::Runtime => RuntimeEntryAppConfig::default()",
        "RuntimeSessionProfile::Editor | RuntimeSessionProfile::Dev",
        "EventLoopPolicy::DesktopApp",
        "RuntimeSessionProfile::Minimal | RuntimeSessionProfile::Headless",
        "WindowDescriptor::default().without_primary_window()",
        "EventLoopPolicy::Headless",
        "WindowExitCondition::DontExit",
    ] {
        assert!(
            runtime_runner_source.contains(required),
            "runtime session profile host mapping should preserve `{required}`"
        );
    }
}
