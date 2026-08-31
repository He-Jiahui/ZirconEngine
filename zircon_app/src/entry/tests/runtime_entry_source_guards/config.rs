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
            && runtime_config_source.contains("exit_after_presented_frames")
            && runtime_config_source.contains("exit_after_first_presented_frame")
            && runtime_config_source.contains("require_persisted_scene_diagnostics")
            && runtime_config_source.contains("reference_cpu_presenter"),
        "runtime entry app config should carry the neutral window descriptor, event-loop policy, lifecycle policy, first-frame exit validation policy, persisted-scene diagnostic policy, and explicit degraded presenter policy"
    );
    assert!(
        runtime_config_source.contains("with_window_lifecycle_policy")
            && runtime_config_source.contains("with_close_when_requested")
            && runtime_config_source.contains("window_lifecycle_policy(&self)")
            && runtime_config_source.contains("with_exit_after_presented_frames")
            && runtime_config_source.contains("exit_after_presented_frames(&self)")
            && runtime_config_source.contains("with_exit_after_first_presented_frame")
            && runtime_config_source.contains("exit_after_first_presented_frame(&self)")
            && runtime_config_source.contains("with_persisted_scene_diagnostics")
            && runtime_config_source.contains("require_persisted_scene_diagnostics(&self)")
            && runtime_config_source.contains("with_reference_cpu_presenter")
            && runtime_config_source.contains("reference_cpu_presenter(&self)"),
        "runtime entry app config should expose the Bevy-style close/exit host policy, explicit startup-smoke first-frame exit policy, project-scoped F2 diagnostics policy, and opt-in degraded presenter policy"
    );
    assert_source_order(
        runtime_config_app_config_source,
        &[
            "struct RuntimeEntryAppConfig",
            "window_descriptor: WindowDescriptor",
            "event_loop_policy: EventLoopPolicy",
            "window_lifecycle_policy: WindowLifecyclePolicy",
            "exit_after_presented_frames: Option<NonZeroU64>",
            "require_persisted_scene_diagnostics: bool",
            "reference_cpu_presenter: bool",
            "fn with_window_descriptor",
            "fn with_event_loop_policy",
            "fn with_window_lifecycle_policy",
            "fn with_exit_after_first_presented_frame",
            "fn with_exit_after_presented_frames",
            "fn with_persisted_scene_diagnostics",
            "fn with_reference_cpu_presenter",
            "impl Default for RuntimeEntryAppConfig",
            "EventLoopPolicy::Game",
            "exit_after_presented_frames: None",
            "require_persisted_scene_diagnostics: false",
            "reference_cpu_presenter: false",
        ],
        "runtime app-config implementation should keep host policy fields, builder methods, and defaults source-visible",
    );
    assert!(
        runtime_construct_source.contains("RuntimeEntryAppConfig")
            && runtime_construct_source.contains("config.window_descriptor")
            && runtime_construct_source.contains("config.event_loop_policy")
            && runtime_construct_source.contains("config.window_lifecycle_policy")
            && runtime_construct_source.contains("config.exit_after_presented_frames")
            && runtime_construct_source.contains("config.require_persisted_scene_diagnostics")
            && runtime_construct_source.contains("config.reference_cpu_presenter"),
        "runtime entry construction should seed host state, persisted-scene diagnostic policy, and explicit degraded presenter policy from RuntimeEntryAppConfig"
    );
    assert!(
        runtime_app_source.contains("window_lifecycle_policy: WindowLifecyclePolicy")
            && runtime_app_source.contains("exit_after_presented_frames: Option<NonZeroU64>")
            && runtime_app_source.contains("presented_frame_count: u64")
            && runtime_app_source.contains("require_persisted_scene_diagnostics: bool")
            && runtime_app_source.contains("reference_cpu_presenter_enabled: bool")
            && runtime_app_source.contains("failure_state: RuntimeEntryAppFailureState"),
        "runtime entry construction should retain close/exit, first-frame-exit, persisted-scene diagnostic, and terminal callback-failure policies"
    );
    assert!(
        runtime_failure_source.contains("runtime startup diagnostic: component={}")
            && runtime_failure_source.contains("RuntimeEntryAppFailureState")
            && runtime_failure_source.contains("AtomicBool")
            && runtime_failure_source.contains("self.failures.record(")
            && !runtime_failure_source.contains("Mutex<Option<RuntimeEntryAppFailure>"),
        "runtime entry failures should use an atomic hot-path stop gate and retain every failure in the cold-path ledger"
    );
    assert_source_order(
        runtime_failure_source,
        &[
            "self.failures.record(",
            "self.recorded.store(true, Ordering::Release);",
        ],
        "runtime callback failures must enter the ledger before the stop gate publishes them to Acquire readers",
    );
    let session_create_start = runtime_runner_source
        .find("let session = match RuntimeSession::create_with_profile_and_project(")
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
            "project_root.as_ref().map(ResolvedProjectPath::operation_path),",
            "runtime_session_args.play_scene.as_ref(),",
            "runtime_session_args.play_report_pipe.as_deref(),",
            ".map_err(|error|",
            "\"runtime_session\"",
            "runtime_session_startup_request(",
            "runtime_session_args.profile,",
            "project_root.as_ref(),",
            "runtime_project_diagnostic_cause(project_root.as_ref(), error)",
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
            "resolve_runtime_project_root(runtime_session_args.project_root.as_deref())",
            "runtime_presented_frame_exit_limit_from_env()",
            "RuntimeSession::create_with_profile_and_project",
            "project_root.as_ref().map(ResolvedProjectPath::operation_path)",
            "runtime_entry_app_config_for_session_profile_with_presented_frame_exit_limit",
            "presented_frame_exit_limit",
            ".with_persisted_scene_diagnostics(project_root.is_some())",
            ".with_reference_cpu_presenter(runtime_session_args.reference_cpu_presenter)",
            "RuntimeEntryAppFailureState::with_failure_ledger(product_failure_ledger.clone())",
            "RuntimeEntryApp::new(session, host_config, failure_state)",
            "event_loop.run_app(app)",
            "product_failure_ledger.record(",
            "let terminal_status = if product_failure_ledger.is_empty()",
            "let terminal_report_result = report_play_startup(",
            "record_runtime_terminal_report_failure(",
            "let failure_report = product_failure_ledger.snapshot();",
            "let terminal_result = finish_runtime_process(",
            "terminal_result?;",
            "runtime_process_teardown_complete_diagnostic()",
        ],
        "runtime runner should derive the app host config from the parsed session profile and aggregate terminal failures after the event loop ends",
    );
    assert!(
        runtime_runner_source.contains("ZIRCON_RUNTIME_EXIT_AFTER_FIRST_FRAME"),
        "runtime startup smoke should keep the first-frame exit validation hook source-visible"
    );
    assert!(
        runtime_runner_source.contains("ZIRCON_RUNTIME_EXIT_AFTER_PRESENTED_FRAMES"),
        "runtime profiling should keep the explicit presented-frame exit limit source-visible"
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
