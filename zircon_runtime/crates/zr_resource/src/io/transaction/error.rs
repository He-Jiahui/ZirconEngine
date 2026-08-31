use std::io;
use std::path::PathBuf;

use thiserror::Error;

use crate::io::ArtifactIdentityExhausted;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransactionPhase {
    Recovery,
    Stage,
    Commit,
    Rollback,
}

#[derive(Debug, Error)]
pub enum DurableTransactionError {
    #[error(transparent)]
    ArtifactIdentityExhausted(#[from] ArtifactIdentityExhausted),
    #[error("invalid durable transaction journal {path}: {reason}")]
    InvalidJournal { path: PathBuf, reason: String },
    #[error("failed to deserialize durable transaction journal {path}: {source}")]
    JournalDeserialize {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("durable file transaction failed during {phase:?} for {path}: {source}")]
    Operation {
        phase: TransactionPhase,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl DurableTransactionError {
    pub(crate) fn invalid(path: impl Into<PathBuf>, reason: impl Into<String>) -> Self {
        Self::InvalidJournal {
            path: path.into(),
            reason: reason.into(),
        }
    }

    pub(crate) fn operation(
        phase: TransactionPhase,
        path: impl Into<PathBuf>,
        source: io::Error,
    ) -> Self {
        Self::Operation {
            phase,
            path: path.into(),
            source,
        }
    }
}
