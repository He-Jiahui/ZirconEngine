#[path = "create_project.rs"]
mod create_project;
#[path = "open_project.rs"]
mod open_project;
#[path = "preflight.rs"]
mod preflight;
#[path = "project_authority.rs"]
mod project_authority;
#[path = "recent_validation.rs"]
mod recent_validation;
#[path = "transaction.rs"]
mod transaction;

pub use project_authority::ProjectAuthority;
pub(super) use transaction::{
    cleanup_failed_transaction_staging, commit_staged_directory, finalize_published_project,
    ProjectCreationLease,
};

#[cfg(test)]
#[path = "tests/cases.rs"]
mod tests;
