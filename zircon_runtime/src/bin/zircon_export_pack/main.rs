#[path = "args.rs"]
mod args;
#[path = "error.rs"]
mod error;
#[path = "manifest.rs"]
mod manifest;
#[path = "pack.rs"]
mod pack;
#[path = "run.rs"]
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
