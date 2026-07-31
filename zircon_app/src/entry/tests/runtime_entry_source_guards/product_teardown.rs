use super::super::source_assertions::assert_source_order;

#[test]
fn product_binaries_log_teardown_completion_before_process_log_shutdown() {
    let editor = include_str!("../../../bin/editor.rs");
    let runtime = include_str!("../../../bin/runtime_preview.rs");

    assert_source_order(
        editor,
        &[
            "EntryRunner::run_editor_with_args_exit_code",
            "editor_process_teardown_complete",
            "let _ = shutdown_process_log",
        ],
        "editor binary must report teardown only after its entry runner returns and before log shutdown",
    );
    assert_source_order(
        runtime,
        &[
            "EntryRunner::run_runtime_with_args",
            "runtime_process_teardown_complete",
            "let _ = shutdown_process_log",
        ],
        "runtime binary must report teardown only after its entry runner returns and before log shutdown",
    );
}
