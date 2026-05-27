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
            "LoadedRuntime::load_default()",
            "RuntimeSession::create_with_profile(runtime, runtime_session_args.profile.as_bytes())",
        ],
        "runtime runner should parse logging first, allow help before dynamic loading, then pass the selected session profile to the dynamic runtime",
    );
    assert!(
        runtime_session_source.contains("profile: ZrByteSlice::from_static(profile)"),
        "runtime session creation should pass the selected profile bytes through ZrRuntimeSessionConfigV1"
    );
}
