use super::{
    runtime_process_exit_code, runtime_process_exit_code_after_log_shutdown,
    runtime_process_failure_teardown_diagnostic,
};

#[test]
fn completed_process_log_shutdown_preserves_the_runtime_exit_code() {
    assert_eq!(
        runtime_process_exit_code_after_log_shutdown(
            zircon_app::ProductTerminalOutcome::host(
                zircon_app::ProductExitClass::Success,
                "runtime_completed",
            ),
            true
        )
        .exit_code()
        .code(),
        0
    );
}

#[test]
fn process_log_shutdown_failure_is_secondary_and_fails_a_successful_runtime_exit() {
    assert_eq!(
        runtime_process_exit_code_after_log_shutdown(
            zircon_app::ProductTerminalOutcome::host(
                zircon_app::ProductExitClass::Success,
                "runtime_completed",
            ),
            false
        )
        .exit_code()
        .code(),
        7
    );
}

#[test]
fn successful_runtime_process_returns_success() {
    let exit_code = runtime_process_exit_code(Ok(()));

    assert_eq!(exit_code.exit_code().code(), 0);
}

#[test]
fn failed_runtime_process_returns_failure() {
    let exit_code = runtime_process_exit_code(Err(std::io::Error::other(
        "expected runtime startup failure",
    )
    .into()));

    assert_eq!(exit_code.exit_code().code(), 1);
}

#[test]
fn failed_runtime_result_retains_primary_when_log_shutdown_fails() {
    let primary = runtime_process_exit_code(Err(std::io::Error::other("run failed").into()));
    let outcome = runtime_process_exit_code_after_log_shutdown(primary, false);
    assert_eq!(outcome.primary().exit_code().code(), 1);
    assert_eq!(outcome.exit_code().code(), 1);
    assert_eq!(outcome.secondary().len(), 1);
}

#[test]
fn failed_runtime_process_reports_completed_top_level_teardown() {
    assert_eq!(
        runtime_process_failure_teardown_diagnostic(&Err::<(), ()>(())),
        Some("runtime_process_teardown_complete result=failed exit_code=1")
    );
    assert_eq!(
        runtime_process_failure_teardown_diagnostic(&Ok::<(), ()>(())),
        None,
        "successful teardown is already reported by EntryRunner"
    );
}
