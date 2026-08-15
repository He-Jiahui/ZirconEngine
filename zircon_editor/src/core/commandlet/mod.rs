//! Headless editor commandlets projected from the canonical command registry.

mod runner;

pub use runner::{
    AuthoringAutomationCommandletRequest, CommandletExitCode, CommandletHost,
    CommandletMigrationChange, CommandletMigrationIssue, CommandletMigrationReport,
    CommandletReport, CommandletRequest, CommandletStatus, parse_commandlet_args, run_commandlet,
    run_commandlet_with_capabilities, run_commandlet_with_host,
};

#[cfg(test)]
mod tests;
