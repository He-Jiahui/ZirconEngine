//! Live publication, idempotent rollback, and terminal artifact cleanup.

use std::fs;
use std::io;
use std::path::Path;

use super::error::{DurableTransactionError, TransactionPhase};
use super::journal::{record_phase, record_state};
use super::observation::DurableCommitReport;
use super::schema::{JournalDocument, JournalPhase, JournalState, TransactionFault};
use super::stage::{copy_and_sync_hash, remove_reserved_if_exists, StagedFile};
use crate::core::resource::io::{replace_staged_file, sync_parent_directory};

pub(super) fn commit_file(
    staged: &mut StagedFile,
    fault: TransactionFault,
    index: usize,
) -> io::Result<()> {
    #[cfg(not(test))]
    let _ = (fault, index);
    // Publication errors can occur after the live namespace changed. Treat every replace attempt
    // as rollback-eligible; restoring an unchanged target is idempotent when replacement failed
    // before publication.
    staged.committed = true;
    replace_staged_file(&staged.intent.staging, &staged.intent.target)?;
    #[cfg(test)]
    if matches!(fault, TransactionFault::FailAfterTargetReplace(fault_index) if fault_index == index)
    {
        return Err(io::Error::other(
            "injected durability failure after target replacement",
        ));
    }
    #[cfg(test)]
    if matches!(fault, TransactionFault::CrashAfterTargetReplace(fault_index) if fault_index == index)
    {
        return Err(io::Error::new(
            io::ErrorKind::Interrupted,
            "injected interruption after target replacement",
        ));
    }
    if let Some(retired) = &staged.intent.retired_path {
        fs::remove_file(retired)?;
        sync_parent_directory(retired)?;
        #[cfg(test)]
        if matches!(fault, TransactionFault::CrashAfterRetiredDelete(fault_index) if fault_index == index)
        {
            return Err(io::Error::new(
                io::ErrorKind::Interrupted,
                "injected interruption after retired-file deletion",
            ));
        }
    }
    Ok(())
}

pub(super) fn rollback_and_cleanup(
    journal: &Path,
    staged: &mut [StagedFile],
    committed_end: usize,
    mut journal_append_safe: bool,
    fault: TransactionFault,
    report: &mut DurableCommitReport,
) -> Result<(), DurableTransactionError> {
    #[cfg(not(test))]
    let _ = fault;
    let mut first_restore_error = None;
    for (index, document) in staged[..committed_end].iter_mut().enumerate().rev() {
        if !document.committed {
            continue;
        }
        if journal_append_safe {
            #[cfg(test)]
            let transition = if matches!(
                fault,
                TransactionFault::FailRollbackTransition { restore_index, .. }
                    if restore_index == index
            ) {
                Err(DurableTransactionError::operation(
                    TransactionPhase::Rollback,
                    journal,
                    io::Error::other("injected rollback transition failure"),
                ))
            } else {
                record_state(journal, index, JournalState::RollingBack)
            };
            #[cfg(not(test))]
            let transition = record_state(journal, index, JournalState::RollingBack);
            if transition.is_err() {
                // A failed append can leave a torn terminal frame. Never append behind an
                // uncertain tail; restored live files are enough to finish with direct cleanup.
                journal_append_safe = false;
            }
        }
        report.record_rollback_restore_attempt();
        #[cfg(test)]
        if matches!(fault, TransactionFault::RestoreFailure { restore_index, .. } if restore_index == index)
        {
            first_restore_error.get_or_insert_with(|| {
                DurableTransactionError::operation(
                    TransactionPhase::Rollback,
                    journal,
                    io::Error::other("injected restore failure"),
                )
            });
            continue;
        }
        match restore_staged(document) {
            Ok(()) => {
                document.committed = false;
                report.record_rollback_restore_success();
            }
            Err(error) => {
                first_restore_error.get_or_insert_with(|| {
                    DurableTransactionError::operation(TransactionPhase::Rollback, journal, error)
                });
            }
        }
    }
    if let Some(error) = first_restore_error {
        return Err(error);
    }
    if !journal_append_safe {
        return cleanup_restored_after_append_failure(journal, staged);
    }
    record_phase(journal, JournalPhase::RollbackCompleted)?;
    #[cfg(test)]
    if matches!(fault, TransactionFault::CrashAfterRollbackCompleted { .. }) {
        return Err(DurableTransactionError::operation(
            TransactionPhase::Rollback,
            journal,
            io::Error::new(
                io::ErrorKind::Interrupted,
                "injected interruption after rollback completion",
            ),
        ));
    }
    record_phase(journal, JournalPhase::CleanupRollback)?;
    #[cfg(test)]
    if matches!(fault, TransactionFault::FailRollbackJournalDelete { .. }) {
        return Err(DurableTransactionError::operation(
            TransactionPhase::Rollback,
            journal,
            io::Error::new(
                io::ErrorKind::PermissionDenied,
                "injected rollback journal deletion failure",
            ),
        ));
    }
    cleanup_staged(journal, staged, TransactionPhase::Rollback)
}

pub(super) fn restore_document(document: &JournalDocument) -> io::Result<()> {
    let target_existed = document.target_existed.ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            "journal document has no target origin evidence",
        )
    })?;
    if target_existed {
        restore_from_backup(
            &document.backup,
            &document.rollback_staging,
            &document.target,
            document.original_digest.as_deref().unwrap_or_default(),
        )?;
    } else {
        remove_live_if_exists(&document.target)?;
    }
    if let (Some(retired), Some(backup), Some(rollback), Some(digest)) = (
        document.retired_path.as_deref(),
        document.retired_backup.as_deref(),
        document.retired_rollback_staging.as_deref(),
        document.retired_digest.as_deref(),
    ) {
        restore_from_backup(backup, rollback, retired, digest)?;
    }
    Ok(())
}

pub(super) fn cleanup_documents(
    journal: &Path,
    documents: &[JournalDocument],
    phase: TransactionPhase,
) -> Result<(), DurableTransactionError> {
    for document in documents {
        for artifact in document_artifacts(document) {
            remove_reserved_if_exists(artifact)
                .map_err(|source| DurableTransactionError::operation(phase, artifact, source))?;
        }
    }
    remove_reserved_if_exists(journal)
        .map_err(|source| DurableTransactionError::operation(phase, journal, source))
}

pub(super) fn cleanup_documents_journal_first(
    journal: &Path,
    documents: &[JournalDocument],
) -> Result<(), DurableTransactionError> {
    remove_reserved_if_exists(journal).map_err(|source| {
        DurableTransactionError::operation(TransactionPhase::Recovery, journal, source)
    })?;
    for document in documents {
        for artifact in document_artifacts(document) {
            remove_reserved_if_exists(artifact).map_err(|source| {
                DurableTransactionError::operation(TransactionPhase::Recovery, artifact, source)
            })?;
        }
    }
    Ok(())
}

pub(super) fn cleanup_committed(
    journal: &Path,
    staged: &[StagedFile],
) -> Result<(), DurableTransactionError> {
    cleanup_staged(journal, staged, TransactionPhase::Commit)
}

pub(super) fn should_fail_before_commit(fault: TransactionFault, index: usize) -> bool {
    #[cfg(not(test))]
    let _ = index;
    match fault {
        TransactionFault::None => false,
        #[cfg(test)]
        TransactionFault::BeforeCommit(fault_index) => fault_index == index,
        #[cfg(test)]
        TransactionFault::RestoreFailure { commit_index, .. }
        | TransactionFault::FailRollbackTransition { commit_index, .. }
        | TransactionFault::CrashAfterRollbackCompleted { commit_index }
        | TransactionFault::FailRollbackJournalDelete { commit_index } => commit_index == index,
        #[cfg(test)]
        TransactionFault::CrashAfterCommit(_)
        | TransactionFault::CrashAfterAllCommitted
        | TransactionFault::FailCommitPointWrite
        | TransactionFault::FailCommitPointSync
        | TransactionFault::CrashAfterCleanup
        | TransactionFault::FailCleanupTransition
        | TransactionFault::FailCommittedCleanup
        | TransactionFault::FailStageWrite(_)
        | TransactionFault::FailStagingDirectorySync(_)
        | TransactionFault::FailBackupCopy(_)
        | TransactionFault::FailRetiredBackupSync(_)
        | TransactionFault::CrashAfterStaging(_)
        | TransactionFault::CrashAfterTargetReplace(_)
        | TransactionFault::FailAfterTargetReplace(_)
        | TransactionFault::CrashAfterRetiredDelete(_) => false,
    }
}

fn restore_staged(document: &StagedFile) -> io::Result<()> {
    if document.target_existed {
        restore_from_backup(
            &document.intent.backup,
            &document.intent.rollback_staging,
            &document.intent.target,
            document.original_digest.as_deref().unwrap_or_default(),
        )?;
    } else {
        remove_live_if_exists(&document.intent.target)?;
    }
    if let (Some(retired), Some(backup), Some(rollback), Some(digest)) = (
        document.intent.retired_path.as_deref(),
        document.intent.retired_backup.as_deref(),
        document.intent.retired_rollback_staging.as_deref(),
        document.retired_digest.as_deref(),
    ) {
        restore_from_backup(backup, rollback, retired, digest)?;
    }
    Ok(())
}

fn restore_from_backup(
    backup: &Path,
    rollback_staging: &Path,
    target: &Path,
    expected_digest: &str,
) -> io::Result<()> {
    let digest = copy_and_sync_hash(backup, rollback_staging)?;
    if digest != expected_digest {
        let _ = remove_reserved_if_exists(rollback_staging);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "rollback backup digest changed",
        ));
    }
    replace_staged_file(rollback_staging, target)
}

fn remove_live_if_exists(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Ok(()) => sync_parent_directory(path),
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
    }
}

fn cleanup_staged(
    journal: &Path,
    staged: &[StagedFile],
    phase: TransactionPhase,
) -> Result<(), DurableTransactionError> {
    for document in staged {
        for artifact in staged_artifacts(document) {
            remove_reserved_if_exists(artifact)
                .map_err(|source| DurableTransactionError::operation(phase, artifact, source))?;
        }
    }
    remove_reserved_if_exists(journal)
        .map_err(|source| DurableTransactionError::operation(phase, journal, source))
}

fn cleanup_restored_after_append_failure(
    journal: &Path,
    staged: &[StagedFile],
) -> Result<(), DurableTransactionError> {
    // Every live path is already durably restored. Remove the active journal before its backup
    // evidence so a partial cleanup cannot leave a recoverable-looking record without evidence.
    remove_reserved_if_exists(journal).map_err(|source| {
        DurableTransactionError::operation(TransactionPhase::Rollback, journal, source)
    })?;
    for document in staged {
        for artifact in staged_artifacts(document) {
            remove_reserved_if_exists(artifact).map_err(|source| {
                DurableTransactionError::operation(TransactionPhase::Rollback, artifact, source)
            })?;
        }
    }
    Ok(())
}

fn staged_artifacts(document: &StagedFile) -> impl Iterator<Item = &Path> {
    [
        Some(document.intent.staging.as_path()),
        Some(document.intent.backup.as_path()),
        Some(document.intent.rollback_staging.as_path()),
        document.intent.retired_backup.as_deref(),
        document.intent.retired_rollback_staging.as_deref(),
    ]
    .into_iter()
    .flatten()
}

pub(super) fn document_artifacts(document: &JournalDocument) -> impl Iterator<Item = &Path> {
    [
        Some(document.staging.as_path()),
        Some(document.backup.as_path()),
        Some(document.rollback_staging.as_path()),
        document.retired_backup.as_deref(),
        document.retired_rollback_staging.as_deref(),
    ]
    .into_iter()
    .flatten()
}
