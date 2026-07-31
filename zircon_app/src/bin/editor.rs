fn main() -> std::process::ExitCode {
    use zircon_runtime::diagnostic_log::{
        install_process_log_panic_flush, shutdown_process_log, write_log,
        DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT, DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT,
    };

    install_process_log_panic_flush(DEFAULT_DIAGNOSTIC_LOG_CRASH_FLUSH_TIMEOUT);
    let result = zircon_app::EntryRunner::run_editor_with_args_exit_code(std::env::args().skip(1));
    write_log("editor_app", editor_process_teardown_diagnostic(&result));
    let _ = shutdown_process_log(DEFAULT_DIAGNOSTIC_LOG_SHUTDOWN_TIMEOUT);
    match result {
        Ok(exit_code) => std::process::ExitCode::from(exit_code),
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
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
    use super::editor_process_teardown_diagnostic;

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
