fn runtime_process_exit_code(
    result: Result<(), Box<dyn std::error::Error>>,
) -> zircon_app::ProductTerminalOutcome {
    match result {
        Ok(()) => zircon_app::ProductTerminalOutcome::host(
            zircon_app::ProductExitClass::Success,
            "runtime_completed",
        ),
        Err(error) => {
            eprintln!("{error}");
            zircon_app::ProductTerminalOutcome::host(
                zircon_app::ProductExitClass::UnclassifiedFailure,
                "runtime_runner_failed",
            )
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
    runtime_process_exit_code_after_log_shutdown(exit_code, process_log_shutdown_completed)
        .exit_code()
        .into()
}

fn runtime_process_exit_code_after_log_shutdown(
    mut exit_code: zircon_app::ProductTerminalOutcome,
    process_log_shutdown_completed: bool,
) -> zircon_app::ProductTerminalOutcome {
    if !process_log_shutdown_completed {
        eprintln!(
            "runtime startup diagnostic: component=diagnostic_log requested=process-log-shutdown cause=log flush timed out or an output failed recovery=inspect the process log output and retry zircon_runtime"
        );
    }
    exit_code.observe_diagnostic_log_shutdown(process_log_shutdown_completed);
    exit_code
}

fn runtime_process_failure_teardown_diagnostic<E>(result: &Result<(), E>) -> Option<&'static str> {
    result
        .is_err()
        .then_some("runtime_process_teardown_complete result=failed exit_code=1")
}

#[cfg(test)]
#[path = "tests/runtime_preview.rs"]
mod tests;
