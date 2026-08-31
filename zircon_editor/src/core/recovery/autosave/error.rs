use std::io;
use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum AutosaveError {
    #[error("autosave document id `{value}` must use only ASCII letters, digits, `_`, or `-`")]
    InvalidDocumentId { value: String },
    #[error("autosave extension `{value}` must use only ASCII letters, digits, `_`, or `-`")]
    InvalidExtension { value: String },
    #[error("autosave interval must be greater than zero")]
    ZeroInterval,
    #[error("autosave sequence must be greater than zero, received {sequence}")]
    InvalidSequence { sequence: u64 },
    #[error("autosave sequence space is exhausted for document `{document}`")]
    SequenceExhausted { document: String },
    #[error(
        "autosave recovery source path `{path}` must be non-empty, UTF-8, and project-relative"
    )]
    InvalidRecoverySourcePath { path: PathBuf },
    #[error(
        "autosave document `{document}` is already mapped to `{recorded}` and cannot be remapped to `{requested}`"
    )]
    RecoverySourceConflict {
        document: String,
        recorded: PathBuf,
        requested: PathBuf,
    },
    #[error("autosave recovery metadata is missing at `{path}`")]
    RecoveryMetadataMissing { path: PathBuf },
    #[error("autosave recovery metadata at `{path}` is invalid: {message}")]
    InvalidRecoveryMetadata { path: PathBuf, message: String },
    #[error("autosave recovery directory `{path}` is not a valid document identifier")]
    InvalidRecoveryDocumentDirectory { path: PathBuf },
    #[error("autosave snapshot `{snapshot}` does not match its committed checksum")]
    SnapshotChecksumMismatch { snapshot: PathBuf },
    #[error("autosave snapshot already exists at `{path}`")]
    SnapshotAlreadyExists { path: PathBuf },
    #[error("autosave snapshot sequence {sequence} is already in use for `{directory}`")]
    SnapshotSequenceUnavailable { directory: PathBuf, sequence: u64 },
    #[error(
        "autosave snapshot `{snapshot}` was persisted, but retention rotation failed: {source}"
    )]
    RotationAfterWrite {
        snapshot: PathBuf,
        #[source]
        source: Box<AutosaveError>,
    },
    #[error("failed to {operation} `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}
