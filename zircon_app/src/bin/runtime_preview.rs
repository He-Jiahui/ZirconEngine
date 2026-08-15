fn runtime_process_exit_code(
    result: Result<(), Box<dyn std::error::Error>>,
) -> std::process::ExitCode {
    match result {
        Ok(()) => std::process::ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
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
    let _ = shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT);
    exit_code
}

fn runtime_process_failure_teardown_diagnostic<E>(result: &Result<(), E>) -> Option<&'static str> {
    result
        .is_err()
        .then_some("runtime_process_teardown_complete result=failed exit_code=1")
}

#[cfg(test)]
mod tests {
    use super::{runtime_process_exit_code, runtime_process_failure_teardown_diagnostic};

    #[test]
    fn successful_runtime_process_returns_success() {
        let exit_code = runtime_process_exit_code(Ok(()));

        assert_eq!(exit_code, std::process::ExitCode::SUCCESS);
    }

    #[test]
    fn failed_runtime_process_returns_failure() {
        let exit_code = runtime_process_exit_code(Err(std::io::Error::other(
            "expected runtime startup failure",
        )
        .into()));

        assert_eq!(exit_code, std::process::ExitCode::FAILURE);
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
