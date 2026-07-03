mod args;
mod error;
mod manifest;
mod run;

use std::process::ExitCode;

fn main() -> ExitCode {
    match run::run(std::env::args_os().skip(1)) {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::from(1)
        }
    }
}
