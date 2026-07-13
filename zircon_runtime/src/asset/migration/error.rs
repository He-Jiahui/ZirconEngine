use std::path::PathBuf;

use thiserror::Error;

use crate::asset::project::ProjectManifestError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssetMigrationTransactionPhase {
    Recovery,
    Stage,
    Commit,
    Rollback,
}

#[derive(Debug, Error)]
pub enum AssetMigrationError {
    #[error("failed to resolve project root {path}: {source}")]
    ProjectRoot {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to load project manifest {path}: {source}")]
    Manifest {
        path: PathBuf,
        #[source]
        source: ProjectManifestError,
    },
    #[error("failed to scan authoring assets below {path}: {source}")]
    Scan {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid asset migration journal {path}: {reason}")]
    InvalidJournal { path: PathBuf, reason: String },
    #[error("failed to deserialize asset migration journal {path}: {source}")]
    JournalDeserialize {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("failed to write migrated authoring asset {path}: {source}")]
    Write {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("asset migration transaction failed during {phase:?} for {path}: {source}")]
    Transaction {
        phase: AssetMigrationTransactionPhase,
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
}
