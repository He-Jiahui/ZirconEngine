mod create_project;
mod open_project;
mod preflight;
mod project_authority;
mod recent_validation;
mod transaction;

pub use project_authority::ProjectAuthority;
pub(super) use transaction::{
    cleanup_failed_transaction_staging, commit_staged_directory, finalize_published_project,
    ProjectCreationLease,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
