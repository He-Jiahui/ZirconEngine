//! Live-target commit, in-process rollback, and artifact cleanup.

use std::fs;
use std::io;
use std::path::Path;

use super::schema::{CommitFault, JournalPhase};
use super::stage::StagedDocument;
use super::{remove_if_exists, sync_journal, transaction_error};
use crate::asset::migration::{AssetMigrationError, AssetMigrationTransactionPhase};
use crate::asset::project::meta_io::atomic_write;

pub(super) fn commit_document(
    document: &mut StagedDocument,
    fault: CommitFault,
    document_index: usize,
) -> io::Result<()> {
    #[cfg(not(test))]
    let _ = (fault, document_index);
    let bytes = fs::read(&document.staging)?;
    atomic_write(&document.target, &bytes)?;
    document.committed = true;
    #[cfg(test)]
    if matches!(fault, CommitFault::CrashAfterTargetReplace(index) if index == document_index) {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "injected interruption after target replacement",
        ));
    }
    if let Some(retired) = &document.retired_path {
        fs::remove_file(retired)?;
        #[cfg(test)]
        if matches!(fault, CommitFault::CrashAfterRetiredDelete(index) if index == document_index) {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "injected interruption after retired sidecar deletion",
            ));
        }
    }
    Ok(())
}

pub(super) fn rollback_and_cleanup(
    journal: &Path,
    staged: &mut [StagedDocument],
    committed_end: usize,
    fault: CommitFault,
) -> Result<(), AssetMigrationError> {
    let mut first_error = None;
    for (index, document) in staged[..committed_end].iter_mut().enumerate().rev() {
        if !document.committed {
            continue;
        }
        let mut restored = true;
        if should_fail_restore(fault, index) {
            first_error.get_or_insert_with(|| {
                io::Error::new(io::ErrorKind::Other, "injected migration restore failure")
            });
            continue;
        }
        if let Some(backup) = document.backup.as_ref() {
            match fs::read(&backup).and_then(|bytes| atomic_write(&document.target, &bytes)) {
                Ok(()) => {}
                Err(error) => {
                    first_error.get_or_insert(error);
                    restored = false;
                }
            }
        } else if let Err(error) = fs::remove_file(&document.target) {
            if error.kind() != io::ErrorKind::NotFound {
                first_error.get_or_insert(error);
                restored = false;
            }
        }
        if let (Some(retired), Some(backup)) = (
            document.retired_path.as_ref(),
            document.retired_backup.as_ref(),
        ) {
            match fs::read(&backup).and_then(|bytes| atomic_write(retired, &bytes)) {
                Ok(()) => {}
                Err(error) => {
                    first_error.get_or_insert(error);
                    restored = false;
                }
            }
        }
        if restored {
            document.committed = false;
        }
    }
    if let Some(error) = first_error {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Rollback,
            journal.to_path_buf(),
            error,
        ));
    }
    sync_journal(journal, staged, JournalPhase::RollbackCompleted)?;
    #[cfg(test)]
    if matches!(fault, CommitFault::CrashAfterRollbackCompleted { .. }) {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Rollback,
            journal.to_path_buf(),
            io::Error::new(
                io::ErrorKind::Interrupted,
                "injected rollback cleanup interruption",
            ),
        ));
    }
    sync_journal(journal, staged, JournalPhase::CleanupRollback)?;
    cleanup_rollback_artifacts(journal, staged, fault)
}

fn cleanup_rollback_artifacts(
    journal: &Path,
    staged: &[StagedDocument],
    fault: CommitFault,
) -> Result<(), AssetMigrationError> {
    for document in staged {
        for artifact in [
            Some(&document.staging),
            document.backup.as_ref(),
            document.retired_backup.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            remove_if_exists(artifact).map_err(|source| {
                transaction_error(
                    AssetMigrationTransactionPhase::Rollback,
                    artifact.to_path_buf(),
                    source,
                )
            })?;
        }
    }
    #[cfg(test)]
    if matches!(fault, CommitFault::FailRollbackJournalDelete { .. }) {
        return Err(transaction_error(
            AssetMigrationTransactionPhase::Rollback,
            journal.to_path_buf(),
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected rollback journal deletion failure",
            ),
        ));
    }
    remove_if_exists(journal).map_err(|source| {
        transaction_error(
            AssetMigrationTransactionPhase::Rollback,
            journal.to_path_buf(),
            source,
        )
    })
}

pub(super) fn cleanup_committed_artifacts(
    journal: &Path,
    staged: &[StagedDocument],
) -> Result<(), AssetMigrationError> {
    for document in staged {
        for artifact in [
            Some(&document.staging),
            document.backup.as_ref(),
            document.retired_backup.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            remove_if_exists(artifact).map_err(|source| {
                transaction_error(
                    AssetMigrationTransactionPhase::Commit,
                    artifact.to_path_buf(),
                    source,
                )
            })?;
        }
    }
    fs::remove_file(journal).map_err(|source| {
        transaction_error(
            AssetMigrationTransactionPhase::Commit,
            journal.to_path_buf(),
            source,
        )
    })
}

pub(super) fn should_fail(fault: CommitFault, index: usize) -> bool {
    #[cfg(not(test))]
    let _ = index;
    match fault {
        CommitFault::Never => false,
        #[cfg(test)]
        CommitFault::CrashAfter(_) => false,
        #[cfg(test)]
        CommitFault::CrashAfterAllCommitted | CommitFault::CrashAfterCleanup => false,
        #[cfg(test)]
        CommitFault::CrashAfterRollbackCompleted { commit_index }
        | CommitFault::FailRollbackJournalDelete { commit_index } => commit_index == index,
        #[cfg(test)]
        CommitFault::At(fault_index) => fault_index == index,
        #[cfg(test)]
        CommitFault::AtWithRestoreFailure { commit_index, .. } => commit_index == index,
        #[cfg(test)]
        CommitFault::FailStageWrite(_)
        | CommitFault::FailBackupCopy(_)
        | CommitFault::FailRetiredBackupSync(_)
        | CommitFault::CrashAfterStaging(_)
        | CommitFault::CrashAfterTargetReplace(_)
        | CommitFault::CrashAfterRetiredDelete(_) => false,
    }
}

fn should_fail_restore(fault: CommitFault, index: usize) -> bool {
    #[cfg(not(test))]
    let _ = index;
    match fault {
        #[cfg(test)]
        CommitFault::AtWithRestoreFailure { restore_index, .. } => restore_index == index,
        _ => false,
    }
}
