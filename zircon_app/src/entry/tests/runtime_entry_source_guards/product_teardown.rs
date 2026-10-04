use super::super::source_assertions::assert_source_order;

#[test]
fn product_binaries_log_teardown_completion_before_process_log_shutdown() {
    let editor = include_str!("../../../bin/editor.rs");
    let runtime = include_str!("../../../bin/runtime_preview.rs");
    let server = include_str!("../../../bin/server.rs");

    assert_source_order(
        editor,
        &[
            "EntryRunner::run_editor_with_args_terminal",
            "write_log(\"editor_app\", editor_process_teardown_diagnostic(&result));",
            "let process_log_shutdown_completed =",
            "shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT)",
            "editor_process_exit_code(result, process_log_shutdown_completed)",
        ],
        "editor binary must report teardown only after its entry runner returns and before log shutdown",
    );
    assert_source_order(
        runtime,
        &[
            "EntryRunner::run_runtime_with_args",
            "let failure_teardown_diagnostic = runtime_process_failure_teardown_diagnostic(&result)",
            "let exit_code = runtime_process_exit_code(result)",
            "eprintln!(\"{diagnostic}\")",
            "let process_log_shutdown_completed =",
            "shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT)",
            "runtime_process_exit_code_after_log_shutdown(",
        ],
        "runtime binary must report the startup error and top-level failure teardown before log shutdown",
    );
    assert_source_order(
        server,
        &[
            "let shutdown_deadline = controller.begin_shutdown();",
            "controller.finish_runtime_until(shutdown_deadline)",
            "shutdown_process_log(",
            "shutdown_deadline.saturating_duration_since(std::time::Instant::now())",
            "controller.mark_shutdown_complete();",
        ],
        "headless server runtime teardown and diagnostic log shutdown must share one absolute deadline",
    );
}
