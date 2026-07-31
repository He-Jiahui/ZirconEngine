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
    // `EntryRunner` emits `runtime_process_teardown_complete` only after the
    // event loop returns without a recorded runtime failure.
    let exit_code = runtime_process_exit_code(result);
    let _ = shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT);
    exit_code
}

#[cfg(test)]
mod tests {
    use super::runtime_process_exit_code;

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
}
