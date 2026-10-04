use std::io;
use std::path::Path;

use super::document::PendingDocument;
use super::{AssetMigrationError, AssetMigrationTransactionPhase};
use crate::core::resource::io::transaction::{
    commit_prepared_files, DurableCommitDisposition, DurableCommitReport, DurableTransactionError,
    PreparedFileWrite, TransactionFault, TransactionPhase,
};

mod journal_owner;
mod recovery;
mod toml_evidence;

pub(in crate::asset::migration) const JOURNAL_DIRECTORY: &str = "asset-migration";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(in crate::asset::migration) enum CommitFault {
    Never,
    #[cfg(test)]
    At(usize),
    #[cfg(test)]
    AtWithRestoreFailure {
        commit_index: usize,
        restore_index: usize,
    },
    #[cfg(test)]
    CrashAfter(usize),
    #[cfg(test)]
    CrashAfterAllCommitted,
    #[cfg(test)]
    FailCommitPointSync,
    #[cfg(test)]
    CrashAfterCleanup,
    #[cfg(test)]
    CrashAfterRollbackCompleted {
        commit_index: usize,
    },
    #[cfg(test)]
    FailRollbackJournalDelete {
        commit_index: usize,
    },
    #[cfg(test)]
    FailStageWrite(usize),
    #[cfg(test)]
    FailBackupCopy(usize),
    #[cfg(test)]
    FailRetiredBackupSync(usize),
    #[cfg(test)]
    CrashAfterStaging(usize),
    #[cfg(test)]
    CrashAfterTargetReplace(usize),
    #[cfg(test)]
    CrashAfterRetiredDelete(usize),
}

pub(super) fn apply_transaction(
    project_root: &Path,
    pending: Vec<PendingDocument>,
    fault: CommitFault,
) -> Result<(), AssetMigrationError> {
    if pending.is_empty() {
        return Ok(());
    }
    let journal_directory = journal_owner::ensure_journal_directory(project_root)?;
    let mut writes = Vec::with_capacity(pending.len());
    writes.extend(pending.into_iter().map(|document| {
        let write = PreparedFileWrite::new(document.path, document.bytes);
        match document.retired_path {
            Some(path) => write.retiring(path),
            None => write,
        }
    }));
    let mut report = DurableCommitReport::default();
    let result = commit_prepared_files(
        &journal_directory,
        "migrate",
        writes,
        fault.into_core(),
        &mut report,
    );
    let disposition = result.map_err(map_transaction_error)?;
    if disposition == DurableCommitDisposition::CommitRecoveryDeferred {
        return Err(AssetMigrationError::Transaction {
            phase: AssetMigrationTransactionPhase::Commit,
            path: journal_directory,
            source: io::Error::other(
                "migration commit marker durability is unresolved; rerun apply mode to recover the pending transaction",
            ),
        });
    }
    Ok(())
}

impl CommitFault {
    fn into_core(self) -> TransactionFault {
        match self {
            Self::Never => TransactionFault::None,
            #[cfg(test)]
            Self::At(index) => TransactionFault::BeforeCommit(index),
            #[cfg(test)]
            Self::AtWithRestoreFailure {
                commit_index,
                restore_index,
            } => TransactionFault::RestoreFailure {
                commit_index,
                restore_index,
            },
            #[cfg(test)]
            Self::CrashAfter(index) => TransactionFault::CrashAfterCommit(index),
            #[cfg(test)]
            Self::CrashAfterAllCommitted => TransactionFault::CrashAfterAllCommitted,
            #[cfg(test)]
            Self::FailCommitPointSync => TransactionFault::FailCommitPointSync,
            #[cfg(test)]
            Self::CrashAfterCleanup => TransactionFault::CrashAfterCleanup,
            #[cfg(test)]
            Self::CrashAfterRollbackCompleted { commit_index } => {
                TransactionFault::CrashAfterRollbackCompleted { commit_index }
            }
            #[cfg(test)]
            Self::FailRollbackJournalDelete { commit_index } => {
                TransactionFault::FailRollbackJournalDelete { commit_index }
            }
            #[cfg(test)]
            Self::FailStageWrite(index) => TransactionFault::FailStageWrite(index),
            #[cfg(test)]
            Self::FailBackupCopy(index) => TransactionFault::FailBackupCopy(index),
            #[cfg(test)]
            Self::FailRetiredBackupSync(index) => TransactionFault::FailRetiredBackupSync(index),
            #[cfg(test)]
            Self::CrashAfterStaging(index) => TransactionFault::CrashAfterStaging(index),
            #[cfg(test)]
            Self::CrashAfterTargetReplace(index) => {
                TransactionFault::CrashAfterTargetReplace(index)
            }
            #[cfg(test)]
            Self::CrashAfterRetiredDelete(index) => {
                TransactionFault::CrashAfterRetiredDelete(index)
            }
        }
    }
}

pub(in crate::asset::migration) fn map_transaction_error(
    error: DurableTransactionError,
) -> AssetMigrationError {
    match error {
        DurableTransactionError::ArtifactIdentityExhausted(source) => {
            AssetMigrationError::ArtifactIdentityExhausted(source)
        }
        DurableTransactionError::InvalidJournal { path, reason } => {
            AssetMigrationError::InvalidJournal { path, reason }
        }
        DurableTransactionError::JournalDeserialize { path, source } => {
            AssetMigrationError::JournalDeserialize { path, source }
        }
        DurableTransactionError::Operation {
            phase,
            path,
            source,
        } => AssetMigrationError::Transaction {
            phase: match phase {
                TransactionPhase::Recovery => AssetMigrationTransactionPhase::Recovery,
                TransactionPhase::Stage => AssetMigrationTransactionPhase::Stage,
                TransactionPhase::Commit => AssetMigrationTransactionPhase::Commit,
                TransactionPhase::Rollback => AssetMigrationTransactionPhase::Rollback,
            },
            path,
            source,
        },
    }
}

pub(in crate::asset::migration) use recovery::{
    detect_pending_transactions, recover_pending_transactions,
};

fn recovery_io(path: &Path, source: io::Error) -> AssetMigrationError {
    AssetMigrationError::Transaction {
        phase: AssetMigrationTransactionPhase::Recovery,
        path: path.to_path_buf(),
        source,
    }
}

#[cfg(test)]
#[path = "transaction/tests/optimization_tests.rs"]
mod optimization_tests;

#[cfg(test)]
#[path = "tests/transaction.rs"]
mod tests;
