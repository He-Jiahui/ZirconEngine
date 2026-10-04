fn main() -> std::process::ExitCode {
    use zircon_runtime::diagnostic_log::{
        install_process_log_panic_flush, shutdown_process_log, write_log,
        DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT, DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT,
    };

    install_process_log_panic_flush(DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT);
    let result = zircon_app::EntryRunner::run_editor_with_args_terminal(std::env::args().skip(1));
    write_log("editor_app", editor_process_teardown_diagnostic(&result));
    let process_log_shutdown_completed =
        shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT);
    editor_process_exit_code(result, process_log_shutdown_completed)
        .exit_code()
        .into()
}

fn editor_process_exit_code<E: std::fmt::Display>(
    result: Result<zircon_app::ProductTerminalOutcome, E>,
    process_log_shutdown_completed: bool,
) -> zircon_app::ProductTerminalOutcome {
    if !process_log_shutdown_completed {
        eprintln!(
            "editor startup diagnostic: component=diagnostic_log requested=process-log-shutdown cause=log flush timed out or an output failed recovery=inspect the process log output and retry zircon_editor"
        );
    }

    let mut outcome = match result {
        Ok(outcome) => outcome,
        Err(error) => {
            eprintln!("{error}");
            zircon_app::ProductTerminalOutcome::host(
                zircon_app::ProductExitClass::UnclassifiedFailure,
                "editor_runner_failed",
            )
        }
    };
    outcome.observe_diagnostic_log_shutdown(process_log_shutdown_completed);
    outcome
}

fn editor_process_teardown_diagnostic<E>(
    result: &Result<zircon_app::ProductTerminalOutcome, E>,
) -> String {
    match result {
        Ok(outcome) => format!(
            "editor_process_teardown_complete result=completed exit_code={}",
            outcome.exit_code().code()
        ),
        Err(_) => "editor_process_teardown_complete result=failed exit_code=1".to_string(),
    }
}

#[cfg(test)]
#[path = "tests/editor.rs"]
mod tests;
