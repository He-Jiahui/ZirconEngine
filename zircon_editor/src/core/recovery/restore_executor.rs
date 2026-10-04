use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use thiserror::Error;
use zircon_runtime::core::resource::io::atomic_write_new;

use super::{
    AutosaveDocumentId, RestoreAction, RestoreCandidate, RestorePlan, RestoreResolution,
    RestoreStartup,
};

const RECOVERY_OUTPUT_DIRECTORY: &str = "recovered";
const RECOVERED_COPY_DIRECTORY: &str = "restore";
const COMPARISON_COPY_DIRECTORY: &str = "comparison";
const COPY_NAME_ALLOCATION_ATTEMPTS: u32 = 64;

/// Executes a validated recovery plan without ever replacing an authoritative source file.
pub struct RestoreExecutor {
    project_root: PathBuf,
}

impl RestoreExecutor {
    pub fn new(project_root: impl Into<PathBuf>) -> Self {
        Self {
            project_root: project_root.into(),
        }
    }

    pub fn execute(
        &self,
        startup: &RestoreStartup,
        plan: &RestorePlan,
    ) -> Result<RestoreExecutionReport, RestoreExecutionError> {
        let mut candidates = BTreeMap::new();
        for candidate in startup.candidates() {
            let document = candidate.document().clone();
            if candidates.insert(document.clone(), candidate).is_some() {
                return Err(RestoreExecutionError::DuplicateCandidate {
                    document: document.as_str().to_string(),
                });
            }
        }
        for resolution in plan.resolutions() {
            if !candidates.contains_key(resolution.document()) {
                return Err(RestoreExecutionError::UnexpectedResolution {
                    document: resolution.document().as_str().to_string(),
                });
            }
        }

        let mut records = Vec::with_capacity(plan.resolutions().len());
        for resolution in plan.resolutions() {
            let Some(candidate) = candidates.get(resolution.document()) else {
                return Err(RestoreExecutionError::UnexpectedResolution {
                    document: resolution.document().as_str().to_string(),
                });
            };
            records.push(RestoreExecutionRecord {
                resolution: resolution.clone(),
                result: self.execute_resolution(candidate, resolution.action()),
            });
        }

        Ok(RestoreExecutionReport { records })
    }

    fn execute_resolution(
        &self,
        candidate: &RestoreCandidate,
        action: RestoreAction,
    ) -> Result<RestoreExecutionOutcome, RestoreDocumentExecutionError> {
        self.validate_candidate_path(candidate)?;
        match action {
            RestoreAction::RestoreAutosave => self
                .materialize_copy(candidate, RECOVERED_COPY_DIRECTORY)
                .map(RestoreExecutionOutcome::RecoveredCopy),
            RestoreAction::OpenComparison => self
                .materialize_copy(candidate, COMPARISON_COPY_DIRECTORY)
                .map(RestoreExecutionOutcome::ComparisonCopy),
            RestoreAction::DiscardAutosave => {
                self.discard_candidate(candidate)?;
                Ok(RestoreExecutionOutcome::Discarded {
                    document: candidate.document().clone(),
                })
            }
        }
    }

    fn materialize_copy(
        &self,
        candidate: &RestoreCandidate,
        purpose: &str,
    ) -> Result<RecoveredDocumentCopy, RestoreDocumentExecutionError> {
        let bytes = fs::read(candidate.autosave_path()).map_err(|source| {
            RestoreDocumentExecutionError::Io {
                operation: "read autosave snapshot for recovery",
                path: candidate.autosave_path().to_path_buf(),
                source,
            }
        })?;
        let directory = self
            .project_root
            .join(".zircon")
            .join(RECOVERY_OUTPUT_DIRECTORY)
            .join(purpose);
        for attempt in 0..COPY_NAME_ALLOCATION_ATTEMPTS {
            let path = recovered_copy_path(&directory, candidate, attempt);
            match atomic_write_new(&path, &bytes) {
                Ok(()) => {
                    return Ok(RecoveredDocumentCopy {
                        document: candidate.document().clone(),
                        source_path: candidate.source_path().to_path_buf(),
                        recovered_path: path,
                    });
                }
                Err(source) if source.kind() == io::ErrorKind::AlreadyExists => continue,
                Err(source) => {
                    return Err(RestoreDocumentExecutionError::Io {
                        operation: "publish recovered document copy",
                        path,
                        source,
                    });
                }
            }
        }
        Err(RestoreDocumentExecutionError::CopyNameExhausted {
            document: candidate.document().as_str().to_string(),
            directory,
        })
    }

    fn discard_candidate(
        &self,
        candidate: &RestoreCandidate,
    ) -> Result<(), RestoreDocumentExecutionError> {
        let directory = self.autosave_document_directory(candidate.document());
        match fs::remove_dir_all(&directory) {
            Ok(()) => Ok(()),
            Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(source) => Err(RestoreDocumentExecutionError::Io {
                operation: "discard autosave recovery document",
                path: directory,
                source,
            }),
        }
    }

    fn autosave_document_directory(&self, document: &AutosaveDocumentId) -> PathBuf {
        self.project_root
            .join(".zircon")
            .join("autosave")
            .join(document.as_str())
    }

    fn validate_candidate_path(
        &self,
        candidate: &RestoreCandidate,
    ) -> Result<(), RestoreDocumentExecutionError> {
        let directory = self.autosave_document_directory(candidate.document());
        if candidate.autosave_path().starts_with(&directory) {
            Ok(())
        } else {
            Err(RestoreDocumentExecutionError::InvalidCandidatePath {
                document: candidate.document().as_str().to_string(),
                path: candidate.autosave_path().to_path_buf(),
            })
        }
    }
}

fn recovered_copy_path(directory: &Path, candidate: &RestoreCandidate, attempt: u32) -> PathBuf {
    let extension = candidate
        .source_path()
        .extension()
        .and_then(|extension| extension.to_str())
        .filter(|extension| !extension.is_empty())
        .or_else(|| {
            candidate
                .autosave_path()
                .extension()
                .and_then(|extension| extension.to_str())
                .filter(|extension| !extension.is_empty())
        });
    let suffix = if attempt == 0 {
        String::new()
    } else {
        format!("-{attempt}")
    };
    let file_name = match extension {
        Some(extension) => format!(
            "{}-recovered{suffix}.{extension}",
            candidate.document().as_str()
        ),
        None => format!("{}-recovered{suffix}", candidate.document().as_str()),
    };
    directory.join(file_name)
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RecoveredDocumentCopy {
    document: AutosaveDocumentId,
    source_path: PathBuf,
    recovered_path: PathBuf,
}

impl RecoveredDocumentCopy {
    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub fn source_path(&self) -> &Path {
        &self.source_path
    }

    pub fn recovered_path(&self) -> &Path {
        &self.recovered_path
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RestoreExecutionOutcome {
    RecoveredCopy(RecoveredDocumentCopy),
    ComparisonCopy(RecoveredDocumentCopy),
    Discarded { document: AutosaveDocumentId },
}

impl RestoreExecutionOutcome {
    pub fn document(&self) -> &AutosaveDocumentId {
        match self {
            Self::RecoveredCopy(copy) | Self::ComparisonCopy(copy) => copy.document(),
            Self::Discarded { document } => document,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RestoreExecutionRetryability {
    Retryable,
    RequiresOperatorIntervention,
}

#[derive(Debug, Error)]
pub enum RestoreDocumentExecutionError {
    #[error("recovery candidate for `{document}` escaped its autosave directory: {path}")]
    InvalidCandidatePath { document: String, path: PathBuf },
    #[error("could not allocate a recovered copy name for `{document}` below {directory}")]
    CopyNameExhausted {
        document: String,
        directory: PathBuf,
    },
    #[error("failed to {operation} `{path}`: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        #[source]
        source: io::Error,
    },
}

impl RestoreDocumentExecutionError {
    pub const fn retryability(&self) -> RestoreExecutionRetryability {
        match self {
            Self::InvalidCandidatePath { .. } => {
                RestoreExecutionRetryability::RequiresOperatorIntervention
            }
            Self::CopyNameExhausted { .. } | Self::Io { .. } => {
                RestoreExecutionRetryability::Retryable
            }
        }
    }
}

#[derive(Debug)]
pub struct RestoreExecutionRecord {
    resolution: RestoreResolution,
    result: Result<RestoreExecutionOutcome, RestoreDocumentExecutionError>,
}

impl RestoreExecutionRecord {
    pub fn resolution(&self) -> &RestoreResolution {
        &self.resolution
    }

    pub fn document(&self) -> &AutosaveDocumentId {
        self.resolution.document()
    }

    pub const fn action(&self) -> RestoreAction {
        self.resolution.action()
    }

    pub fn outcome(&self) -> Option<&RestoreExecutionOutcome> {
        self.result.as_ref().ok()
    }

    pub fn failure(&self) -> Option<&RestoreDocumentExecutionError> {
        self.result.as_ref().err()
    }

    pub fn retryability(&self) -> Option<RestoreExecutionRetryability> {
        self.failure()
            .map(RestoreDocumentExecutionError::retryability)
    }
}

#[derive(Debug, Default)]
pub struct RestoreExecutionReport {
    records: Vec<RestoreExecutionRecord>,
}

impl RestoreExecutionReport {
    pub fn records(&self) -> &[RestoreExecutionRecord] {
        &self.records
    }

    pub fn success_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.outcome().is_some())
            .count()
    }

    pub fn failure_count(&self) -> usize {
        self.records.len().saturating_sub(self.success_count())
    }

    pub fn has_failures(&self) -> bool {
        self.records.iter().any(|record| record.failure().is_some())
    }

    pub fn retryable_resolutions(&self) -> Vec<RestoreResolution> {
        self.records
            .iter()
            .filter(|record| record.retryability() == Some(RestoreExecutionRetryability::Retryable))
            .map(|record| record.resolution().clone())
            .collect()
    }
}

#[derive(Debug, Error)]
pub enum RestoreExecutionError {
    #[error("recovery startup contains duplicate candidate `{document}`")]
    DuplicateCandidate { document: String },
    #[error("recovery plan referenced unexpected document `{document}`")]
    UnexpectedResolution { document: String },
}

#[cfg(test)]
#[path = "tests/restore_executor.rs"]
mod tests;
