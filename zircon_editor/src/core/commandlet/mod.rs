//! Headless editor commandlets projected from the canonical command registry.

mod runner;

pub use runner::{
    parse_commandlet_args, run_commandlet, run_commandlet_with_capabilities, CommandletExitCode,
    CommandletMigrationChange, CommandletMigrationIssue, CommandletMigrationReport,
    CommandletReport, CommandletRequest, CommandletStatus,
};

#[cfg(test)]
mod tests;
