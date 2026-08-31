fn main() -> std::process::ExitCode {
    use zircon_runtime::diagnostic_log::{
        install_process_log_panic_flush, shutdown_process_log, write_log,
        DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT, DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT,
    };

    install_process_log_panic_flush(DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT);
    let result = zircon_app::EntryRunner::run_editor_with_args_exit_code(std::env::args().skip(1));
    write_log("editor_app", editor_process_teardown_diagnostic(&result));
    let process_log_shutdown_completed =
        shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT);
    editor_process_exit_code(result, process_log_shutdown_completed).into()
}

fn editor_process_exit_code<E: std::fmt::Display>(
    result: Result<u8, E>,
    process_log_shutdown_completed: bool,
) -> zircon_app::ProductProcessExitCode {
    if !process_log_shutdown_completed {
        eprintln!(
            "editor startup diagnostic: component=diagnostic_log requested=process-log-shutdown cause=log flush timed out or an output failed recovery=inspect the process log output and retry zircon_editor"
        );
    }

    match result {
        Ok(exit_code) if process_log_shutdown_completed => {
            zircon_app::ProductProcessExitCode::from_code(exit_code)
        }
        Ok(_) => zircon_app::ProductProcessExitCode::failure(),
        Err(error) => {
            eprintln!("{error}");
            zircon_app::ProductProcessExitCode::failure()
        }
    }
}

fn editor_process_teardown_diagnostic<E>(result: &Result<u8, E>) -> String {
    match result {
        Ok(exit_code) => {
            format!("editor_process_teardown_complete result=completed exit_code={exit_code}")
        }
        Err(_) => "editor_process_teardown_complete result=failed exit_code=1".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::{editor_process_exit_code, editor_process_teardown_diagnostic};

    #[test]
    fn completed_process_log_shutdown_preserves_the_editor_exit_code() {
        assert_eq!(
            editor_process_exit_code(Ok::<u8, std::io::Error>(7), true).code(),
            7
        );
    }

    #[test]
    fn process_log_shutdown_failure_overrides_a_successful_editor_exit() {
        assert_eq!(
            editor_process_exit_code(Ok::<u8, std::io::Error>(0), false).code(),
            1
        );
    }

    #[test]
    fn teardown_diagnostic_records_completed_exit_code() {
        assert_eq!(
            editor_process_teardown_diagnostic(&Ok::<u8, ()>(0)),
            "editor_process_teardown_complete result=completed exit_code=0"
        );
    }

    #[test]
    fn teardown_diagnostic_records_failed_exit_code() {
        assert_eq!(
            editor_process_teardown_diagnostic(&Err::<u8, ()>(())),
            "editor_process_teardown_complete result=failed exit_code=1"
        );
    }
}
