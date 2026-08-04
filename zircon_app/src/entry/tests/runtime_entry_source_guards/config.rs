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
    let runtime_failure_source = include_str!("../../runtime_entry_app/failure.rs");
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
            && runtime_config_source.contains("WindowLifecyclePolicy")
            && runtime_config_source.contains("exit_after_first_presented_frame")
            && runtime_config_source.contains("require_persisted_scene_diagnostics"),
        "runtime entry app config should carry the neutral window descriptor, event-loop policy, lifecycle policy, first-frame exit validation policy, and persisted-scene diagnostic policy"
    );
    assert!(
        runtime_config_source.contains("with_window_lifecycle_policy")
            && runtime_config_source.contains("with_close_when_requested")
            && runtime_config_source.contains("window_lifecycle_policy(&self)")
            && runtime_config_source.contains("with_exit_after_first_presented_frame")
            && runtime_config_source.contains("exit_after_first_presented_frame(&self)")
            && runtime_config_source.contains("with_persisted_scene_diagnostics")
            && runtime_config_source.contains("require_persisted_scene_diagnostics(&self)"),
        "runtime entry app config should expose the Bevy-style close/exit host policy, explicit startup-smoke first-frame exit policy, and project-scoped F2 diagnostics policy"
    );
    assert_source_order(
        runtime_config_app_config_source,
        &[
            "struct RuntimeEntryAppConfig",
            "window_descriptor: WindowDescriptor",
            "event_loop_policy: EventLoopPolicy",
            "window_lifecycle_policy: WindowLifecyclePolicy",
            "exit_after_first_presented_frame: bool",
            "require_persisted_scene_diagnostics: bool",
            "fn with_window_descriptor",
            "fn with_event_loop_policy",
            "fn with_window_lifecycle_policy",
            "fn with_exit_after_first_presented_frame",
            "fn with_persisted_scene_diagnostics",
            "impl Default for RuntimeEntryAppConfig",
            "EventLoopPolicy::Game",
            "exit_after_first_presented_frame: false",
            "require_persisted_scene_diagnostics: false",
        ],
        "runtime app-config implementation should keep host policy fields, builder methods, and defaults source-visible",
    );
    assert!(
        runtime_construct_source.contains("RuntimeEntryAppConfig")
            && runtime_construct_source.contains("config.window_descriptor")
            && runtime_construct_source.contains("config.event_loop_policy")
            && runtime_construct_source.contains("config.window_lifecycle_policy")
            && runtime_construct_source.contains("config.exit_after_first_presented_frame")
            && runtime_construct_source.contains("config.require_persisted_scene_diagnostics"),
        "runtime entry construction should seed host state and persisted-scene diagnostic policy from RuntimeEntryAppConfig"
    );
    assert!(
        runtime_app_source.contains("window_lifecycle_policy: WindowLifecyclePolicy")
            && runtime_app_source.contains("exit_after_first_presented_frame: bool")
            && runtime_app_source.contains("require_persisted_scene_diagnostics: bool")
            && runtime_app_source.contains("failure_state: RuntimeEntryAppFailureState"),
        "runtime entry construction should retain close/exit, first-frame-exit, persisted-scene diagnostic, and terminal callback-failure policies"
    );
    assert!(
        runtime_failure_source.contains("runtime startup diagnostic: component={}")
            && runtime_failure_source.contains("RuntimeEntryAppFailureState")
            && runtime_failure_source.contains("if recorded_failure.is_none()"),
        "runtime entry failures should remain actionable and retain the first terminal callback failure"
    );
    let session_create_start = runtime_runner_source
        .find("let session = RuntimeSession::create_with_profile_and_project(")
        .expect("runtime runner should create a dynamic runtime session");
    let session_create_done = runtime_runner_source[session_create_start..]
        .find("\"runtime_session_create_done\"")
        .map(|offset| session_create_start + offset)
        .expect("runtime runner should log completed dynamic runtime session creation");
    let session_create = &runtime_runner_source[session_create_start..session_create_done];
    assert_source_order(
        session_create,
        &[
            "RuntimeSession::create_with_profile_and_project(",
            "runtime_session_args.profile.as_bytes(),",
            "project_root.as_deref(),",
            ".map_err(|error|",
            "\"runtime_session\"",
            "runtime_session_startup_request(",
            "runtime_session_args.profile,",
            "project_root.as_deref(),",
            "format!(\"runtime session creation failed: {error}\")",
        ],
        "runtime session creation should retain the selected profile and resolved project identity through the typed product diagnostic boundary",
    );
    assert!(
        runtime_window_creation_source.contains("self.window_descriptor.primary_window.is_none()"),
        "runtime entry should skip concrete winit window creation when the host config has no primary window"
    );
    assert_source_order(
        runtime_runner_source,
        &[
            "parse_runtime_session_startup_args",
            "let project_root =",
            "resolve_runtime_project_root(runtime_session_args.project_root.as_deref())?",
            "RuntimeSession::create_with_profile_and_project",
            "project_root.as_deref()",
            "runtime_entry_app_config_for_session_profile_with_first_frame_exit",
            "runtime_exit_after_first_frame_enabled()",
            ".with_persisted_scene_diagnostics(project_root.is_some())",
            "RuntimeEntryAppFailureState::default()",
            "RuntimeEntryApp::new(session, host_config, failure_state.clone())",
            "event_loop.run_app(app)",
            "let event_loop_failure = result.err().map",
            "let runtime_app_failure = failure_state",
            ".take()",
            "let runtime_session_failure = session_teardown_failure.take().map",
            "finish_runtime_process(",
            ")?;",
            "runtime_process_teardown_complete_diagnostic()",
        ],
        "runtime runner should derive the app host config from the parsed session profile and aggregate terminal failures after the event loop ends",
    );
    assert!(
        runtime_runner_source.contains("ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME"),
        "runtime startup smoke should keep the first-frame exit validation hook source-visible"
    );
    for required in [
        "RuntimeSessionProfile::Runtime | RuntimeSessionProfile::RuntimePipelined",
        "RuntimeEntryAppConfig::default()",
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
