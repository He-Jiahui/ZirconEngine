use std::path::{Path, PathBuf};

use crate::core::recovery::{AutosaveError, RestoreCandidate};

/// The result of scanning one autosave catalog.
///
/// A catalog-root I/O failure still prevents a result. Once the root is enumerable, malformed
/// document entries are quarantined into diagnostics so valid recovery candidates remain usable.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AutosaveRecoveryCatalogReport {
    candidates: Vec<RestoreCandidate>,
    diagnostics: Vec<AutosaveRecoveryCatalogDiagnostic>,
}

impl AutosaveRecoveryCatalogReport {
    pub(super) fn new(
        candidates: Vec<RestoreCandidate>,
        diagnostics: Vec<AutosaveRecoveryCatalogDiagnostic>,
    ) -> Self {
        Self {
            candidates,
            diagnostics,
        }
    }

    pub fn candidates(&self) -> &[RestoreCandidate] {
        &self.candidates
    }

    pub fn diagnostics(&self) -> &[AutosaveRecoveryCatalogDiagnostic] {
        &self.diagnostics
    }
}

/// One recovery directory excluded from the valid candidate set during catalog scanning.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveRecoveryCatalogDiagnostic {
    path: PathBuf,
    kind: AutosaveRecoveryCatalogDiagnosticKind,
}

impl AutosaveRecoveryCatalogDiagnostic {
    pub(super) fn from_entry_error(path: impl Into<PathBuf>, error: AutosaveError) -> Self {
        let path = path.into();
        let kind = match error {
            AutosaveError::RecoveryMetadataMissing { .. } => {
                AutosaveRecoveryCatalogDiagnosticKind::MetadataMissing
            }
            AutosaveError::InvalidRecoveryMetadata { message, .. } => {
                AutosaveRecoveryCatalogDiagnosticKind::InvalidMetadata { message }
            }
            AutosaveError::InvalidRecoveryDocumentDirectory { .. }
            | AutosaveError::InvalidDocumentId { .. } => {
                AutosaveRecoveryCatalogDiagnosticKind::InvalidDocumentDirectory
            }
            AutosaveError::SnapshotChecksumMismatch { .. } => {
                AutosaveRecoveryCatalogDiagnosticKind::CommittedChecksumMismatch
            }
            AutosaveError::Io {
                operation,
                path,
                source,
            } => AutosaveRecoveryCatalogDiagnosticKind::Io {
                operation,
                affected_path: path,
                message: source.to_string(),
            },
            error => AutosaveRecoveryCatalogDiagnosticKind::Other {
                message: error.to_string(),
            },
        };
        Self { path, kind }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> &AutosaveRecoveryCatalogDiagnosticKind {
        &self.kind
    }
}

/// The typed cause for excluding one autosave recovery directory.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AutosaveRecoveryCatalogDiagnosticKind {
    InvalidDocumentDirectory,
    MetadataMissing,
    InvalidMetadata {
        message: String,
    },
    CommittedChecksumMismatch,
    Io {
        operation: &'static str,
        affected_path: PathBuf,
        message: String,
    },
    Other {
        message: String,
    },
}
