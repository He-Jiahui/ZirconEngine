//! Transaction preparation and commit orchestration.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::commit::{
    cleanup_committed, commit_file, rollback_and_cleanup, should_fail_before_commit,
};
use super::error::{DurableTransactionError, TransactionPhase};
use super::journal::{
    create_intent, record_commit_point, record_phase, record_prepared, record_state,
    CommitPointRecord,
};
use super::observation::DurableCommitReport;
use super::owner_lock::TransactionOwnerLock;
use super::pathing::{next_transaction_id, valid_tag, PathIdentity};
use super::schema::{JournalIntent, JournalPhase, JournalState, TransactionFault};
use super::stage::{cleanup_intents, remove_reserved_if_exists, stage_file, verify_originals};
use crate::core::resource::io::ensure_parent_directories;

#[derive(Debug)]
pub(crate) struct PreparedFileWrite {
    pub(crate) path: PathBuf,
    pub(crate) bytes: Vec<u8>,
    pub(super) retired_path: Option<PathBuf>,
}

impl PreparedFileWrite {
    pub(crate) fn new(path: impl Into<PathBuf>, bytes: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            bytes,
            retired_path: None,
        }
    }

    pub(crate) fn retiring(mut self, path: impl Into<PathBuf>) -> Self {
        self.retired_path = Some(path.into());
        self
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum DurableCommitDisposition {
    Durable,
    CommitRecoveryDeferred,
    CleanupDeferred,
}

pub(crate) fn commit_prepared_files(
    journal_directory: &Path,
    tag: &str,
    writes: Vec<PreparedFileWrite>,
    fault: TransactionFault,
    report: &mut DurableCommitReport,
) -> Result<DurableCommitDisposition, DurableTransactionError> {
    if writes.is_empty() {
        return Ok(DurableCommitDisposition::Durable);
    }
    let (journal_directory, writes) = validate_inputs(journal_directory, tag, writes)?;
    ensure_journal_directory(&journal_directory)?;
    let _owner = TransactionOwnerLock::acquire(&journal_directory, TransactionPhase::Stage)?;
    reject_pending_recovery(&journal_directory)?;
    let transaction_id = next_transaction_id();
    let (journal, intents) = create_intent(&journal_directory, tag, &transaction_id, &writes)?;
    let cleanup_intents_snapshot = intents.clone();
    let mut staged = Vec::with_capacity(writes.len());

    for (index, (write, intent)) in writes.into_iter().zip(intents).enumerate() {
        match stage_file(write, intent, fault, index) {
            Ok(document) => {
                staged.push(document);
                if let Err(error) = record_prepared(&journal, index, &staged[index]) {
                    return Err(abort_pre_active(&journal, &cleanup_intents_snapshot, error));
                }
                #[cfg(test)]
                if matches!(fault, TransactionFault::CrashAfterStaging(fault_index) if fault_index == index)
                {
                    return Err(interruption(
                        TransactionPhase::Stage,
                        &journal,
                        "injected interruption after staging",
                    ));
                }
            }
            Err(error) => {
                return Err(abort_pre_active(&journal, &cleanup_intents_snapshot, error));
            }
        }
    }
    for document in &staged {
        if let Err(source) = verify_originals(document) {
            let error = DurableTransactionError::operation(
                TransactionPhase::Stage,
                &document.intent.target,
                source,
            );
            return Err(abort_pre_active(&journal, &cleanup_intents_snapshot, error));
        }
    }
    if let Err(error) = record_phase(&journal, JournalPhase::Active) {
        return Err(abort_pre_active(&journal, &cleanup_intents_snapshot, error));
    }

    for index in 0..staged.len() {
        if should_fail_before_commit(fault, index) {
            let error = io::Error::other("injected transaction commit failure");
            rollback_and_cleanup(&journal, &mut staged, index, true, fault, report)?;
            return Err(DurableTransactionError::operation(
                TransactionPhase::Commit,
                &staged[index].intent.target,
                error,
            ));
        }
        if let Err(error) = record_state(&journal, index, JournalState::Committing) {
            rollback_and_cleanup(&journal, &mut staged, index, false, fault, report)?;
            return Err(error);
        }
        if let Err(source) = commit_file(&mut staged[index], fault, index) {
            #[cfg(test)]
            if matches!(
                fault,
                TransactionFault::CrashAfterTargetReplace(fault_index)
                    | TransactionFault::CrashAfterRetiredDelete(fault_index)
                    if fault_index == index
            ) {
                return Err(DurableTransactionError::operation(
                    TransactionPhase::Commit,
                    &staged[index].intent.target,
                    source,
                ));
            }
            let path = staged[index].intent.target.clone();
            rollback_and_cleanup(&journal, &mut staged, index + 1, true, fault, report)?;
            return Err(DurableTransactionError::operation(
                TransactionPhase::Commit,
                path,
                source,
            ));
        }
        if let Err(error) = record_state(&journal, index, JournalState::Committed) {
            rollback_and_cleanup(&journal, &mut staged, index + 1, false, fault, report)?;
            return Err(error);
        }
        #[cfg(test)]
        if matches!(fault, TransactionFault::CrashAfterCommit(fault_index) if fault_index == index)
        {
            return Err(interruption(
                TransactionPhase::Commit,
                &staged[index].intent.target,
                "injected interruption after committed transition",
            ));
        }
    }

    #[cfg(test)]
    if fault == TransactionFault::FailCommitPointWrite {
        let error = DurableTransactionError::operation(
            TransactionPhase::Commit,
            &journal,
            io::Error::other("injected commit-point write failure"),
        );
        let committed_end = staged.len();
        rollback_and_cleanup(&journal, &mut staged, committed_end, false, fault, report)?;
        return Err(error);
    }
    let commit_point = match record_commit_point(&journal) {
        Ok(record) => record,
        Err(error) => {
            let committed_end = staged.len();
            rollback_and_cleanup(&journal, &mut staged, committed_end, false, fault, report)?;
            return Err(error);
        }
    };
    #[cfg(test)]
    let commit_point = if fault == TransactionFault::FailCommitPointSync {
        CommitPointRecord::PublishedWithoutSync
    } else {
        commit_point
    };
    if commit_point == CommitPointRecord::PublishedWithoutSync {
        // The visible generation and complete commit frame agree. Keep all recovery evidence so a
        // restart can arbitrate based on whether the commit frame reached durable storage.
        report.record_deferred_commit_recovery();
        return Ok(DurableCommitDisposition::CommitRecoveryDeferred);
    }
    #[cfg(test)]
    if fault == TransactionFault::CrashAfterAllCommitted {
        return Err(interruption(
            TransactionPhase::Commit,
            &journal,
            "injected interruption after all-committed transition",
        ));
    }
    #[cfg(test)]
    if fault == TransactionFault::FailCleanupTransition {
        report.record_deferred_cleanup();
        return Ok(DurableCommitDisposition::CleanupDeferred);
    }
    if record_phase(&journal, JournalPhase::Cleanup).is_err() {
        report.record_deferred_cleanup();
        return Ok(DurableCommitDisposition::CleanupDeferred);
    }
    #[cfg(test)]
    if fault == TransactionFault::CrashAfterCleanup {
        return Err(interruption(
            TransactionPhase::Commit,
            &journal,
            "injected interruption after cleanup transition",
        ));
    }
    #[cfg(test)]
    if fault == TransactionFault::FailCommittedCleanup {
        report.record_deferred_cleanup();
        return Ok(DurableCommitDisposition::CleanupDeferred);
    }
    if cleanup_committed(&journal, &staged).is_err() {
        report.record_deferred_cleanup();
        return Ok(DurableCommitDisposition::CleanupDeferred);
    }
    Ok(DurableCommitDisposition::Durable)
}

fn validate_inputs(
    journal_directory: &Path,
    tag: &str,
    writes: Vec<PreparedFileWrite>,
) -> Result<(PathBuf, Vec<PreparedFileWrite>), DurableTransactionError> {
    if !journal_directory.is_absolute() {
        return Err(invalid_input(
            journal_directory,
            "journal directory must be absolute",
        ));
    }
    if !valid_tag(tag) {
        return Err(invalid_input(
            journal_directory,
            "transaction tag must be a lowercase ASCII slug",
        ));
    }
    let journal_directory = resolve_path(journal_directory)?;
    let mut live_paths = BTreeSet::new();
    let mut normalized_writes = Vec::with_capacity(writes.len());
    for mut write in writes {
        if !write.path.is_absolute() {
            return Err(invalid_input(
                &write.path,
                "transaction target must be absolute",
            ));
        }
        let target = PathIdentity::resolve(&write.path).map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Stage, &write.path, source)
        })?;
        let target_path = target.operation_path().to_path_buf();
        if !live_paths.insert(target) {
            return Err(invalid_input(&target_path, "duplicate transaction target"));
        }
        write.path = target_path;
        if let Some(retired) = write.retired_path.take() {
            if !retired.is_absolute() {
                return Err(invalid_input(&retired, "retired path must be absolute"));
            }
            let retired = PathIdentity::resolve(&retired).map_err(|source| {
                DurableTransactionError::operation(TransactionPhase::Stage, &retired, source)
            })?;
            let retired_path = retired.operation_path().to_path_buf();
            if !live_paths.insert(retired) {
                return Err(invalid_input(&retired_path, "transaction live paths alias"));
            }
            write.retired_path = Some(retired_path);
        }
        normalized_writes.push(write);
    }
    Ok((journal_directory, normalized_writes))
}

fn resolve_path(path: &Path) -> Result<PathBuf, DurableTransactionError> {
    PathIdentity::resolve(path)
        .map(PathIdentity::into_operation_path)
        .map_err(|source| DurableTransactionError::operation(TransactionPhase::Stage, path, source))
}

fn abort_pre_active(
    journal: &Path,
    intents: &[JournalIntent],
    original: DurableTransactionError,
) -> DurableTransactionError {
    if let Err(cleanup) = remove_reserved_if_exists(journal) {
        return DurableTransactionError::operation(
            TransactionPhase::Stage,
            journal,
            io::Error::new(
                cleanup.kind(),
                format!("{original}; failed to remove aborted transaction journal: {cleanup}"),
            ),
        );
    }
    if let Err(cleanup) = cleanup_intents(intents) {
        return DurableTransactionError::operation(
            TransactionPhase::Stage,
            journal,
            io::Error::new(
                cleanup.kind(),
                format!("{original}; failed to clean staged transaction artifacts: {cleanup}"),
            ),
        );
    }
    original
}

fn ensure_journal_directory(path: &Path) -> Result<(), DurableTransactionError> {
    if !path.exists() {
        ensure_parent_directories(&path.join(".journal-owner")).map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Stage, path, source)
        })?;
    }
    let metadata = fs::symlink_metadata(path).map_err(|source| {
        DurableTransactionError::operation(TransactionPhase::Stage, path, source)
    })?;
    if metadata.file_type().is_symlink() || !metadata.is_dir() {
        return Err(invalid_input(
            path,
            "journal owner must be a real directory",
        ));
    }
    Ok(())
}

fn reject_pending_recovery(directory: &Path) -> Result<(), DurableTransactionError> {
    let pending = fs::read_dir(directory)
        .map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Stage, directory, source)
        })?
        .next()
        .transpose()
        .map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Stage, directory, source)
        })?;
    if let Some(entry) = pending {
        return Err(DurableTransactionError::operation(
            TransactionPhase::Stage,
            entry.path(),
            io::Error::new(
                io::ErrorKind::InvalidData,
                "durable transaction owner has pending recovery",
            ),
        ));
    }
    Ok(())
}

fn invalid_input(path: &Path, message: &str) -> DurableTransactionError {
    DurableTransactionError::operation(
        TransactionPhase::Stage,
        path,
        io::Error::new(io::ErrorKind::InvalidInput, message),
    )
}

#[cfg(test)]
fn interruption(phase: TransactionPhase, path: &Path, message: &str) -> DurableTransactionError {
    DurableTransactionError::operation(
        phase,
        path,
        io::Error::new(io::ErrorKind::Interrupted, message),
    )
}

#[cfg(test)]
mod tests;
