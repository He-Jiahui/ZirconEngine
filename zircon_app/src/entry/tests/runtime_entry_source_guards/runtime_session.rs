use super::super::source_assertions::assert_source_order;

#[test]
fn runtime_runner_forwards_session_profile_to_dynamic_runtime() {
    let runtime_runner_source = include_str!("../../entry_runner/runtime.rs");
    let runtime_session_args_source = include_str!("../../entry_runner/runtime_session_args.rs");
    let runtime_session_source = include_str!("../../runtime_library/runtime_session.rs");

    assert!(
        runtime_session_args_source.contains("--runtime-session-profile"),
        "runtime runner should expose an explicit dynamic session profile argument"
    );
    assert!(
        runtime_session_args_source.contains("--project")
            && runtime_session_args_source.contains("project_root: Option<PathBuf>"),
        "runtime runner should expose a project-root argument for standalone game projects"
    );
    assert!(
        runtime_session_args_source.contains("--play-scene")
            && runtime_session_args_source.contains("play_scene: Option<RelPath>")
            && runtime_session_args_source.contains("--play-report-pipe")
            && runtime_session_args_source.contains("play_report_pipe: Option<String>"),
        "runtime runner should type Play scene/report startup values instead of leaving them as unknown arguments"
    );
    assert!(
        runtime_session_args_source.contains("\"dev\"")
            && runtime_session_args_source.contains("\"minimal\"")
            && runtime_session_args_source.contains("\"headless\""),
        "runtime session profile parser should accept the dynamic runtime's named profiles"
    );
    assert!(
        runtime_session_args_source.contains("RUNTIME_SESSION_STARTUP_HELP")
            && runtime_session_args_source.contains("ZIRCON_RUNTIME_LIBRARY")
            && runtime_session_args_source.contains("ZIRCON_LOG_FILTER")
            && runtime_session_args_source.contains("ZIRCON_LOG")
            && runtime_session_args_source.contains("RUST_LOG")
            && runtime_session_args_source.contains("ZIRCON_LOG_LEVEL"),
        "runtime session profile parser should expose startup help for profiles, logging, and runtime library override"
    );
    assert_source_order(
        runtime_runner_source,
        &[
            "parse_diagnostic_log_startup_args(args)?",
            "parse_runtime_session_startup_args",
            "if runtime_session_args.help_requested",
            "return Ok(());",
            "let project_root =",
            "resolve_runtime_project_root(runtime_session_args.project_root.as_deref())?",
            "LoadedRuntime::load_default()",
            "EventLoop::new().map_err",
            "event_loop.create_proxy()",
            "RuntimeSession::create_with_profile_and_project",
            "runtime_session_args.profile.as_bytes()",
            "project_root.as_ref().map(ResolvedProjectPath::operation_path)",
            "let session_teardown_failure = session.teardown_failure_state();",
        ],
        "runtime runner should parse logging first, resolve the selected project once before dynamic loading, create the event loop wake proxy, then pass the selected session profile and physical project root to the dynamic runtime",
    );
    let runtime_load_start = runtime_runner_source
        .find("let runtime = LoadedRuntime::load_default()")
        .expect("runtime runner should load the staged dynamic runtime");
    let event_loop_start = runtime_runner_source[runtime_load_start..]
        .find("let event_loop = EventLoop::new()")
        .map(|offset| runtime_load_start + offset)
        .expect("runtime runner should create an event loop after dynamic runtime loading");
    let runtime_load = &runtime_runner_source[runtime_load_start..event_loop_start];
    assert!(
        runtime_load.contains(".map_err(|error|"),
        "runtime library loading failures should enter the product diagnostic boundary"
    );
    let runtime_load_lines = runtime_load.lines().map(str::trim).collect::<Vec<_>>();
    assert!(
        runtime_load_lines.windows(5).any(|lines| {
            lines
                == [
                    "runtime_library_startup_error(",
                    "runtime_session_args.profile,",
                    "project_root.as_ref(),",
                    "error,",
                    ")",
                ]
        }),
        "runtime library diagnostics should receive the selected profile/project and original loading error"
    );
    assert!(
        runtime_runner_source.contains("fn runtime_library_startup_error(")
            && runtime_runner_source.contains("fn resolve_runtime_project_root(")
            && runtime_runner_source.contains("ProjectPaths::resolve_existing(requested_root)")
            && runtime_runner_source.contains("\"runtime_library\"")
            && runtime_runner_source
                .contains("runtime_session_startup_request(profile, project_root)"),
        "runtime startup should resolve one physical project root before dynamic loading and retain the selected request in a stable diagnostic component"
    );
    assert!(
        runtime_session_source.contains("profile: ZrByteSlice::from_static(profile)"),
        "runtime session creation should pass the selected profile bytes through ZrRuntimeSessionConfigV3"
    );
    assert!(
        runtime_session_source.contains("project_root,")
            && runtime_session_source.contains("play_scene,")
            && runtime_session_source.contains("play_report_pipe,"),
        "runtime session creation should pass the root and relative Play startup values through ZrRuntimeSessionConfigV3"
    );
    for phase in ["Starting", "Ready", "StartFailed", "Terminal"] {
        assert!(
            runtime_runner_source.contains(&format!("PlayStartupReportPhase::{phase}")),
            "runtime Play report outlet should emit the typed {phase} phase"
        );
    }
    assert!(
        runtime_session_source.contains("wake_sink:")
            && runtime_runner_source.contains("RuntimeWakeRegistration::register")
            && runtime_runner_source.contains("event_loop.create_proxy()"),
        "standalone runtime creation should bind the V3 session to a real host wake proxy"
    );
    assert_source_order(
        runtime_runner_source,
        &[
            "let session_teardown_failure = session.teardown_failure_state();",
            "let product_failure_ledger = session_teardown_failure.failure_ledger();",
            "let result = event_loop.run_app(app);",
            "product_failure_ledger.record(",
            "let failure_report = product_failure_ledger.snapshot();",
            "finish_runtime_process(",
            "failure_report,",
            "PlayStartupReportPhase::Terminal",
            "terminal_result?;",
            "runtime_process_teardown_complete_diagnostic()",
        ],
        "runtime runner must collect event-loop, callback, and session teardown failures before reporting successful product teardown",
    );
}
