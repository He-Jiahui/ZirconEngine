fn runtime_process_exit_code(
    result: Result<(), Box<dyn std::error::Error>>,
) -> zircon_app::ProductProcessExitCode {
    match result {
        Ok(()) => zircon_app::ProductProcessExitCode::Success,
        Err(error) => {
            eprintln!("{error}");
            zircon_app::ProductProcessExitCode::failure()
        }
    }
}

fn main() -> std::process::ExitCode {
    use zircon_runtime::diagnostic_log::{
        install_process_log_panic_flush, shutdown_process_log,
        DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT, DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT,
    };

    install_process_log_panic_flush(DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT);
    let result = zircon_app::EntryRunner::run_runtime_with_args(std::env::args().skip(1));
    let failure_teardown_diagnostic = runtime_process_failure_teardown_diagnostic(&result);
    let exit_code = runtime_process_exit_code(result);
    if let Some(diagnostic) = failure_teardown_diagnostic {
        eprintln!("{diagnostic}");
    }
    let process_log_shutdown_completed =
        shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT);
    runtime_process_exit_code_after_log_shutdown(exit_code, process_log_shutdown_completed).into()
}

fn runtime_process_exit_code_after_log_shutdown(
    exit_code: zircon_app::ProductProcessExitCode,
    process_log_shutdown_completed: bool,
) -> zircon_app::ProductProcessExitCode {
    if process_log_shutdown_completed {
        return exit_code;
    }

    eprintln!(
        "runtime startup diagnostic: component=diagnostic_log requested=process-log-shutdown cause=log flush timed out or an output failed recovery=inspect the process log output and retry zircon_runtime"
    );
    zircon_app::ProductProcessExitCode::failure()
}

fn runtime_process_failure_teardown_diagnostic<E>(result: &Result<(), E>) -> Option<&'static str> {
    result
        .is_err()
        .then_some("runtime_process_teardown_complete result=failed exit_code=1")
}

#[cfg(test)]
mod tests {
    use super::{
        runtime_process_exit_code, runtime_process_exit_code_after_log_shutdown,
        runtime_process_failure_teardown_diagnostic,
    };

    #[test]
    fn completed_process_log_shutdown_preserves_the_runtime_exit_code() {
        assert_eq!(
            runtime_process_exit_code_after_log_shutdown(
                zircon_app::ProductProcessExitCode::Success,
                true
            ),
            zircon_app::ProductProcessExitCode::Success
        );
    }

    #[test]
    fn process_log_shutdown_failure_overrides_a_successful_runtime_exit() {
        assert_eq!(
            runtime_process_exit_code_after_log_shutdown(
                zircon_app::ProductProcessExitCode::Success,
                false
            )
            .code(),
            1
        );
    }

    #[test]
    fn successful_runtime_process_returns_success() {
        let exit_code = runtime_process_exit_code(Ok(()));

        assert_eq!(exit_code, zircon_app::ProductProcessExitCode::Success);
    }

    #[test]
    fn failed_runtime_process_returns_failure() {
        let exit_code = runtime_process_exit_code(Err(std::io::Error::other(
            "expected runtime startup failure",
        )
        .into()));

        assert_eq!(exit_code.code(), 1);
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
}
