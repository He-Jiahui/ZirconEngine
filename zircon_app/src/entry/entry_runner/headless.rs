use std::error::Error;

use super::EntryRunner;

mod args;
mod controller;
mod managed;
mod schedule;
mod session;

pub use controller::HeadlessController;
pub use session::{HeadlessHostError, HeadlessRunReport, HeadlessStopReason};

impl EntryRunner {
    pub fn run_headless() -> Result<(), Box<dyn Error>> {
        Self::run_headless_with_args(std::env::args().skip(1), HeadlessController::default())
            .map(|_| ())
    }

    pub fn run_headless_with_args(
        args: impl IntoIterator<Item = impl Into<String>>,
        controller: HeadlessController,
    ) -> Result<HeadlessRunReport, Box<dyn Error>> {
        let args = args::parse(args)?;
        if args.help {
            println!("{}", args::HELP);
            return Ok(HeadlessRunReport::help());
        }
        zircon_runtime::diagnostic_log::initialize_unity_process_log_with_config(
            "server",
            args.log_filter,
        );
        session::run(args.startup, args.tick_limit, controller)
            .map_err(|error| Box::new(error) as Box<dyn Error>)
    }
}

#[cfg(test)]
#[path = "headless/tests/cases.rs"]
mod tests;
