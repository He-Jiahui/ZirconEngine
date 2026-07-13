//! Durable transaction journal schema. These fields are the only recovery authority.

use std::path::PathBuf;

use serde::{Deserialize, Serialize};

pub(super) const JOURNAL_VERSION: u32 = 2;
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

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum JournalState {
    Prepared,
    Committing,
    Committed,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(super) enum JournalPhase {
    Intent,
    Active,
    RollbackCompleted,
    CleanupRollback,
    AllCommitted,
    Cleanup,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct TransactionJournal {
    pub(super) version: u32,
    pub(super) transaction_id: String,
    pub(super) phase: JournalPhase,
    pub(super) documents: Vec<JournalDocument>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(super) struct JournalDocument {
    pub(super) state: JournalState,
    pub(super) target_existed: bool,
    pub(super) original_digest: Option<String>,
    pub(super) new_digest: String,
    pub(super) retired_digest: Option<String>,
    pub(super) target: PathBuf,
    pub(super) staging: PathBuf,
    pub(super) backup: Option<PathBuf>,
    pub(super) retired_path: Option<PathBuf>,
    pub(super) retired_backup: Option<PathBuf>,
}
