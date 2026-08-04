//! Headless editor commandlets projected from the canonical command registry.

mod runner;

pub use runner::{
    CommandletExitCode, CommandletMigrationChange, CommandletMigrationIssue,
    CommandletMigrationReport, CommandletReport, CommandletRequest, CommandletStatus,
    parse_commandlet_args, run_commandlet, run_commandlet_with_capabilities,
};

#[cfg(test)]
mod tests;
