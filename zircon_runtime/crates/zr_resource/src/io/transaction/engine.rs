//! Transaction preparation and commit orchestration.

use std::collections::BTreeSet;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use super::commit::{
    cleanup_committed, commit_file, rollback_and_cleanup, should_fail_before_commit,
};
use super::error::{DurableTransactionError, TransactionPhase};
#[cfg(test)]
use super::journal::create_intent;
use super::journal::{
    CommitPointRecord, persist_intent, plan_intent, record_commit_point, record_phase,
    record_prepared, record_state,
};
use super::observation::DurableCommitReport;
use super::owner_lock::{TransactionOwnerLock, owner_lock_path};
use super::pathing::{PathIdentity, next_transaction_id, valid_tag};
use super::schema::{JournalIntent, JournalPhase, JournalState, TransactionFault};
use super::stage::{
    cleanup_intents, cleanup_intents_journal_first, remove_reserved_if_exists, stage_file,
    verify_originals,
};
use crate::io::ensure_parent_directories;

#[derive(Debug)]
pub struct PreparedFileWrite {
    pub(crate) path: PathBuf,
    pub(crate) bytes: Vec<u8>,
    pub(super) retirements: Vec<PreparedFileRetirement>,
}

#[derive(Debug)]
pub(super) struct PreparedFileRetirement {
    pub(super) path: PathBuf,
    pub(super) expected_digest: Option<String>,
}

impl PreparedFileWrite {
    pub fn new(path: impl Into<PathBuf>, bytes: Vec<u8>) -> Self {
        Self {
            path: path.into(),
            bytes,
            retirements: Vec::new(),
        }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub fn retiring(mut self, path: impl Into<PathBuf>) -> Self {
        self.retirements.push(PreparedFileRetirement {
            path: path.into(),
            expected_digest: None,
        });
        self
    }

    /// Requires the retired live file to retain the digest observed during preparation.
    pub fn retiring_with_expected_digest(
        mut self,
        path: impl Into<PathBuf>,
        expected_digest: impl Into<String>,
    ) -> Self {
        self.retirements.push(PreparedFileRetirement {
            path: path.into(),
            expected_digest: Some(expected_digest.into()),
        });
        self
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DurableCommitDisposition {
    Durable,
    CommitRecoveryDeferred,
    CleanupDeferred,
}

pub fn commit_prepared_files(
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
    let transaction_id = next_transaction_id(&journal_directory)?;
    let (journal, intents) = plan_intent(&journal_directory, tag, &transaction_id, &writes)?;
    ensure_journal_directory(&journal_directory)?;
    let _owner = TransactionOwnerLock::acquire(&journal_directory, TransactionPhase::Stage)?;
    reject_pending_recovery(&journal_directory)?;
    persist_intent(&journal, tag, &transaction_id, &intents)?;
    let cleanup_intents_snapshot = intents.clone();
    let mut staged = Vec::with_capacity(writes.len());

    for (index, (write, intent)) in writes.into_iter().zip(intents).enumerate() {
        match stage_file(write, intent, fault, index) {
            Ok(document) => {
                staged.push(document);
                if let Err(error) = record_prepared(&journal, index, &staged[index]) {
                    return Err(abort_pre_active(
                        &journal,
                        &cleanup_intents_snapshot,
                        error,
                        false,
                    ));
                }
                #[cfg(any(test, feature = "test-support"))]
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
                return Err(abort_pre_active(
                    &journal,
                    &cleanup_intents_snapshot,
                    error,
                    true,
                ));
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
            return Err(abort_pre_active(
                &journal,
                &cleanup_intents_snapshot,
                error,
                true,
            ));
        }
    }
    if let Err(error) = record_phase(&journal, JournalPhase::Active) {
        return Err(abort_pre_active(
            &journal,
            &cleanup_intents_snapshot,
            error,
            false,
        ));
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
            #[cfg(any(test, feature = "test-support"))]
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
        #[cfg(any(test, feature = "test-support"))]
        if matches!(fault, TransactionFault::CrashAfterCommit(fault_index) if fault_index == index)
        {
            return Err(interruption(
                TransactionPhase::Commit,
                &staged[index].intent.target,
                "injected interruption after committed transition",
            ));
        }
    }

    #[cfg(any(test, feature = "test-support"))]
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
    #[cfg(any(test, feature = "test-support"))]
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
    #[cfg(any(test, feature = "test-support"))]
    if fault == TransactionFault::CrashAfterAllCommitted {
        return Err(interruption(
            TransactionPhase::Commit,
            &journal,
            "injected interruption after all-committed transition",
        ));
    }
    #[cfg(any(test, feature = "test-support"))]
    if fault == TransactionFault::FailCleanupTransition {
        report.record_deferred_cleanup();
        return Ok(DurableCommitDisposition::CleanupDeferred);
    }
    if record_phase(&journal, JournalPhase::Cleanup).is_err() {
        report.record_deferred_cleanup();
        return Ok(DurableCommitDisposition::CleanupDeferred);
    }
    #[cfg(any(test, feature = "test-support"))]
    if fault == TransactionFault::CrashAfterCleanup {
        return Err(interruption(
            TransactionPhase::Commit,
            &journal,
            "injected interruption after cleanup transition",
        ));
    }
    #[cfg(any(test, feature = "test-support"))]
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
    let journal_directory = PathIdentity::resolve(journal_directory).map_err(|source| {
        DurableTransactionError::operation(TransactionPhase::Stage, journal_directory, source)
    })?;
    ensure_wire_compatible(journal_directory.operation_path())?;
    let owner_lock = owner_lock_path(journal_directory.operation_path()).map_err(|source| {
        DurableTransactionError::operation(
            TransactionPhase::Stage,
            journal_directory.operation_path(),
            source,
        )
    })?;
    let owner_lock = PathIdentity::resolve(&owner_lock).map_err(|source| {
        DurableTransactionError::operation(TransactionPhase::Stage, &owner_lock, source)
    })?;
    let mut live_paths = BTreeSet::new();
    let mut normalized_writes = Vec::with_capacity(writes.len());
    for mut write in writes {
        if !write.path.is_absolute() {
            return Err(invalid_input(
                &write.path,
                "transaction target must be absolute",
            ));
        }
        ensure_wire_compatible(&write.path)?;
        let target = PathIdentity::resolve(&write.path).map_err(|source| {
            DurableTransactionError::operation(TransactionPhase::Stage, &write.path, source)
        })?;
        ensure_wire_compatible(target.operation_path())?;
        reject_journal_namespace_overlap(&target, &journal_directory)?;
        let target_path = target.operation_path().to_path_buf();
        if !live_paths.insert(target) {
            return Err(invalid_input(&target_path, "duplicate transaction target"));
        }
        write.path = target_path;
        for retirement in &mut write.retirements {
            if !retirement.path.is_absolute() {
                return Err(invalid_input(
                    &retirement.path,
                    "retired path must be absolute",
                ));
            }
            ensure_wire_compatible(&retirement.path)?;
            let retired = PathIdentity::resolve(&retirement.path).map_err(|source| {
                DurableTransactionError::operation(
                    TransactionPhase::Stage,
                    &retirement.path,
                    source,
                )
            })?;
            ensure_wire_compatible(retired.operation_path())?;
            reject_journal_namespace_overlap(&retired, &journal_directory)?;
            let retired_path = retired.operation_path().to_path_buf();
            if !live_paths.insert(retired) {
                return Err(invalid_input(&retired_path, "transaction live paths alias"));
            }
            retirement.path = retired_path;
        }
        normalized_writes.push(write);
    }
    reject_owner_lock_namespace(&live_paths, &owner_lock)?;
    reject_live_namespace_overlaps(&live_paths)?;
    Ok((journal_directory.into_operation_path(), normalized_writes))
}

fn ensure_wire_compatible(path: &Path) -> Result<(), DurableTransactionError> {
    if path.as_os_str().to_str().is_some() {
        Ok(())
    } else {
        Err(invalid_input(
            path,
            "transaction path must be UTF-8 encodable for the journal wire format",
        ))
    }
}

fn reject_owner_lock_namespace(
    live_paths: &BTreeSet<PathIdentity>,
    owner_lock: &PathIdentity,
) -> Result<(), DurableTransactionError> {
    if live_paths.iter().any(|path| {
        path.is_same_or_descendant_of(owner_lock) || owner_lock.is_same_or_descendant_of(path)
    }) {
        return Err(invalid_input(
            owner_lock.operation_path(),
            "transaction live path overlaps the owner lock namespace",
        ));
    }
    Ok(())
}

fn reject_live_namespace_overlaps(
    live_paths: &BTreeSet<PathIdentity>,
) -> Result<(), DurableTransactionError> {
    // Component ordering makes each directory subtree contiguous, so an antichain only needs
    // adjacent containment checks. This avoids materializing every strict ancestor identity.
    let mut ordered_paths = live_paths.iter().collect::<Vec<_>>();
    ordered_paths
        .sort_unstable_by(|left, right| PathIdentity::compare_namespace_paths(*left, *right));
    for pair in ordered_paths.windows(2) {
        if pair[1].is_same_or_descendant_of(pair[0]) {
            return Err(invalid_input(
                pair[1].operation_path(),
                "transaction live paths overlap an ancestor or descendant",
            ));
        }
    }
    Ok(())
}

fn reject_journal_namespace_overlap(
    live_path: &PathIdentity,
    journal_directory: &PathIdentity,
) -> Result<(), DurableTransactionError> {
    if live_path.is_same_or_descendant_of(journal_directory)
        || journal_directory.is_same_or_descendant_of(live_path)
    {
        return Err(invalid_input(
            live_path.operation_path(),
            "transaction live path overlaps the journal owner namespace",
        ));
    }
    Ok(())
}

fn abort_pre_active(
    journal: &Path,
    intents: &[JournalIntent],
    original: DurableTransactionError,
    journal_append_safe: bool,
) -> DurableTransactionError {
    if !journal_append_safe {
        if let Err(cleanup) = cleanup_intents_journal_first(journal, intents) {
            return DurableTransactionError::operation(
                TransactionPhase::Stage,
                journal,
                io::Error::new(
                    cleanup.kind(),
                    format!(
                        "{original}; failed journal-first cleanup after uncertain append: {cleanup}"
                    ),
                ),
            );
        }
        return original;
    }
    if let Err(transition) = record_phase(journal, JournalPhase::CleanupIntent) {
        return preserve_original_operation(
            original,
            format!("failed to record pre-active cleanup transition: {transition}"),
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
    original
}

fn preserve_original_operation(
    original: DurableTransactionError,
    context: String,
) -> DurableTransactionError {
    match original {
        DurableTransactionError::Operation {
            phase,
            path,
            source,
        } => DurableTransactionError::operation(
            phase,
            path,
            io::Error::new(source.kind(), OperationErrorContext { source, context }),
        ),
        other => other,
    }
}

#[derive(Debug)]
struct OperationErrorContext {
    source: io::Error,
    context: String,
}

impl std::fmt::Display for OperationErrorContext {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{}; {}", self.source, self.context)
    }
}

impl std::error::Error for OperationErrorContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

fn ensure_journal_directory(path: &Path) -> Result<(), DurableTransactionError> {
    match fs::symlink_metadata(path) {
        Ok(_) => {}
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            ensure_parent_directories(&path.join(".journal-owner")).map_err(|source| {
                DurableTransactionError::operation(TransactionPhase::Stage, path, source)
            })?;
        }
        Err(source) => {
            return Err(DurableTransactionError::operation(
                TransactionPhase::Stage,
                path,
                source,
            ));
        }
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

#[cfg(any(test, feature = "test-support"))]
fn interruption(phase: TransactionPhase, path: &Path, message: &str) -> DurableTransactionError {
    DurableTransactionError::operation(
        phase,
        path,
        io::Error::new(io::ErrorKind::Interrupted, message),
    )
}

#[cfg(test)]
mod tests;
