//! One-shot project authoring-asset migration and crash-state inspection.
//!
//! Migration journals and their sibling artifacts are untrusted filesystem input. Recovery never
//! writes backup or staging bytes into a live asset and never deletes a live target based on a
//! journal. After strict path and evidence validation it may only discard reserved transaction
//! artifacts; the normal preflight and idempotent migration pass then converges authoritative
//! project files. Unprovable journals fail as [`AssetMigrationError::InvalidJournal`] before writes.

mod document;
mod error;
mod mode;
mod options;
mod report;
mod resolver;
mod resolver_index;
mod run;
mod scan;
mod sidecar;
mod transaction;

pub use error::{AssetMigrationError, AssetMigrationTransactionPhase};
pub use mode::AssetMigrationMode;
pub use options::AssetMigrationOptions;
pub use report::{
    AssetMigrationChange, AssetMigrationIssue, AssetMigrationIssueKind, AssetMigrationReport,
};
pub(crate) use resolver_index::{
    MigrationCompoundBinding, MigrationResolverIndex, MigrationSourceProjection,
};
pub use run::migrate_project_assets;
#[cfg(test)]
pub(crate) use run::migrate_project_assets_with_commit_fault;
#[cfg(test)]
pub(crate) use run::migrate_project_assets_with_commit_window_fault;
#[cfg(test)]
pub(crate) use run::migrate_project_assets_with_process_interruption;
#[cfg(test)]
pub(crate) use run::migrate_project_assets_with_restore_fault;
#[cfg(test)]
pub(crate) use run::migrate_project_assets_with_rollback_cleanup_fault;
#[cfg(test)]
pub(crate) use run::migrate_project_assets_with_stage_fault;
#[cfg(test)]
pub(crate) use run::migrate_project_assets_with_terminal_interruption;
#[cfg(test)]
pub(crate) use scan::scan_migration_inventory_for_test;
