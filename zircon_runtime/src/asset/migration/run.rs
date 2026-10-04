use std::path::PathBuf;

use zircon_runtime_interface::project::RelPath;

use crate::asset::project::{lock_meta_document_paths, ProjectManifest, ProjectPaths};

use super::document::migrate_document;
use super::resolver::MigrationResolver;
use super::resolver_index::MigrationResolverIndex;
use super::scan::MigrationInventory;
use super::sidecar::preflight_sidecars;
use super::transaction::{
    apply_transaction, detect_pending_transactions, recover_pending_transactions, CommitFault,
};
use super::{
    AssetMigrationChange, AssetMigrationError, AssetMigrationIssue, AssetMigrationIssueKind,
    AssetMigrationMode, AssetMigrationOptions, AssetMigrationReport,
    AssetMigrationTransactionPhase,
};

pub fn migrate_project_assets(
    options: AssetMigrationOptions,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    migrate_project_assets_inner(options, CommitFault::Never)
}

fn migrate_project_assets_inner(
    options: AssetMigrationOptions,
    commit_fault: CommitFault,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    migrate_project_assets_with_commit_hook(options, commit_fault, || {})
}

fn migrate_project_assets_with_commit_hook(
    options: AssetMigrationOptions,
    commit_fault: CommitFault,
    before_commit: impl FnOnce(),
) -> Result<AssetMigrationReport, AssetMigrationError> {
    let paths = ProjectPaths::from_root(&options.project_root).map_err(|source| {
        AssetMigrationError::ProjectRoot {
            path: options.project_root.clone(),
            source,
        }
    })?;
    let manifest = ProjectManifest::load(paths.manifest_path()).map_err(|source| {
        AssetMigrationError::Manifest {
            path: paths.manifest_path().to_path_buf(),
            source,
        }
    })?;
    let roots = migration_roots(&paths, &manifest.asset_roots);
    let mut root_paths = Vec::with_capacity(roots.len());
    root_paths.extend(roots.iter().map(|(_, path)| path.clone()));
    let mut report = AssetMigrationReport::new(options.mode);
    let inventory =
        MigrationInventory::build(&roots).map_err(|source| AssetMigrationError::Scan {
            path: paths.root().to_path_buf(),
            source,
        })?;
    let recovery_targets = inventory.transaction_targets().to_vec();
    // Candidate bytes must be read under the same authority as their publication.
    // Locking only the final transaction would allow it to overwrite a newer CAS.
    let mut meta_write_guards = lock_meta_document_paths(&recovery_targets).map_err(|source| {
        AssetMigrationError::Transaction {
            phase: AssetMigrationTransactionPhase::Recovery,
            path: paths.root().to_path_buf(),
            source,
        }
    })?;
    let pending_recovery =
        detect_pending_transactions(paths.root(), &root_paths, &recovery_targets)?;
    if options.mode == AssetMigrationMode::DryRun {
        for journal in &pending_recovery {
            report.push_issue(AssetMigrationIssue::new(
                AssetMigrationIssueKind::PendingRecovery,
                Some(journal.clone()),
                "pending migration recovery requires apply mode",
            ));
        }
    } else if !pending_recovery.is_empty() {
        recover_pending_transactions(paths.root(), &root_paths, &recovery_targets)?;
    }
    let inventory = if pending_recovery.is_empty() || options.mode == AssetMigrationMode::DryRun {
        inventory
    } else {
        // Recovery removes transaction artifacts. Publish a fresh inventory so preflight and
        // every resolver lookup use one post-recovery filesystem generation.
        drop(meta_write_guards);
        let inventory =
            MigrationInventory::build(&roots).map_err(|source| AssetMigrationError::Scan {
                path: paths.root().to_path_buf(),
                source,
            })?;
        meta_write_guards =
            lock_meta_document_paths(inventory.transaction_targets()).map_err(|source| {
                AssetMigrationError::Transaction {
                    phase: AssetMigrationTransactionPhase::Stage,
                    path: paths.root().to_path_buf(),
                    source,
                }
            })?;
        inventory
    };
    report.metrics.entry_visits = inventory.entry_visits();
    report.metrics.directory_reads = inventory.directory_reads();
    report.metrics.directory_sorts = inventory.directory_sorts();
    let sidecars = match preflight_sidecars(&root_paths, &inventory) {
        Ok(sidecars) => sidecars,
        Err(issue) => {
            report.push_issue(issue);
            return Ok(report);
        }
    };
    let files = inventory.authoring_files();
    report.set_scanned_files(files.len());
    let resolver_index = match MigrationResolverIndex::build(
        inventory.resolver_projections(),
        sidecars.compound_bindings,
    ) {
        Ok(index) => index,
        Err(error) => {
            report.push_issue(AssetMigrationIssue::new(
                AssetMigrationIssueKind::InvalidDocument,
                None,
                error.to_string(),
            ));
            return Ok(report);
        }
    };
    let resolver = MigrationResolver::new(&sidecars.index, &resolver_index);
    let sidecar_pending = sidecars.pending;
    for document in &sidecar_pending {
        report.metrics.output_bytes += document.bytes.len();
        report.push_change(AssetMigrationChange::new(document.path.clone(), 0));
    }
    let mut pending = Vec::with_capacity(files.len() + sidecar_pending.len());
    // Publish sidecars before documents that can reference them across crash windows.
    pending.extend(sidecar_pending);
    for path in files {
        match migrate_document(path, &resolver, &mut report.metrics) {
            Ok(Some(document)) => {
                report.metrics.output_bytes += document.bytes.len();
                report.push_change(AssetMigrationChange::new(
                    document.path.clone(),
                    document.reference_count,
                ));
                pending.push(document);
            }
            Ok(None) => {}
            Err(issue) => report.push_issue(issue),
        }
    }
    report.metrics.resolver_index_lookups = resolver.resolver_index_lookups();
    if !report.succeeded() || options.mode == AssetMigrationMode::DryRun {
        return Ok(report);
    }
    before_commit();
    apply_transaction(paths.root(), pending, commit_fault)?;
    drop(meta_write_guards);
    report.mark_applied();
    Ok(report)
}

fn migration_roots(paths: &ProjectPaths, roots: &[RelPath]) -> Vec<(RelPath, PathBuf)> {
    let mut migration_roots = Vec::with_capacity(roots.len());
    migration_roots.extend(roots.iter().cloned().map(|root| {
        let path = paths.asset_root(&root);
        (root, path)
    }));
    migration_roots
}

#[cfg(test)]
pub(crate) fn migrate_project_assets_with_commit_fault(
    options: AssetMigrationOptions,
    commit_index: usize,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    migrate_project_assets_inner(options, CommitFault::At(commit_index))
}

#[cfg(test)]
pub(crate) fn migrate_project_assets_with_restore_fault(
    options: AssetMigrationOptions,
    commit_index: usize,
    restore_index: usize,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    migrate_project_assets_inner(
        options,
        CommitFault::AtWithRestoreFailure {
            commit_index,
            restore_index,
        },
    )
}

#[cfg(test)]
pub(crate) fn migrate_project_assets_with_process_interruption(
    options: AssetMigrationOptions,
    commit_index: usize,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    migrate_project_assets_inner(options, CommitFault::CrashAfter(commit_index))
}

#[cfg(test)]
pub(crate) fn migrate_project_assets_with_terminal_interruption(
    options: AssetMigrationOptions,
    after_cleanup_state: bool,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    migrate_project_assets_inner(
        options,
        if after_cleanup_state {
            CommitFault::CrashAfterCleanup
        } else {
            CommitFault::CrashAfterAllCommitted
        },
    )
}

#[cfg(test)]
pub(crate) fn migrate_project_assets_with_rollback_cleanup_fault(
    options: AssetMigrationOptions,
    commit_index: usize,
    fail_journal_delete: bool,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    migrate_project_assets_inner(
        options,
        if fail_journal_delete {
            CommitFault::FailRollbackJournalDelete { commit_index }
        } else {
            CommitFault::CrashAfterRollbackCompleted { commit_index }
        },
    )
}

#[cfg(test)]
pub(crate) fn migrate_project_assets_with_stage_fault(
    options: AssetMigrationOptions,
    document_index: usize,
    point: u8,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    let fault = match point {
        0 => CommitFault::FailStageWrite(document_index),
        1 => CommitFault::FailBackupCopy(document_index),
        2 => CommitFault::FailRetiredBackupSync(document_index),
        3 => CommitFault::CrashAfterStaging(document_index),
        _ => panic!("unknown stage fault point"),
    };
    migrate_project_assets_inner(options, fault)
}

#[cfg(test)]
#[path = "run/tests/optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "run/tests/meta_authority_tests.rs"]
mod meta_authority_tests;

#[cfg(test)]
pub(crate) fn migrate_project_assets_with_commit_window_fault(
    options: AssetMigrationOptions,
    document_index: usize,
    after_retired_delete: bool,
) -> Result<AssetMigrationReport, AssetMigrationError> {
    migrate_project_assets_inner(
        options,
        if after_retired_delete {
            CommitFault::CrashAfterRetiredDelete(document_index)
        } else {
            CommitFault::CrashAfterTargetReplace(document_index)
        },
    )
}
