fn main() -> std::process::ExitCode {
    match zircon_app::EntryRunner::run_editor_with_args_exit_code(std::env::args().skip(1)) {
        Ok(exit_code) => std::process::ExitCode::from(exit_code),
        Err(error) => {
            eprintln!("{error}");
            std::process::ExitCode::FAILURE
        }
    }
}
