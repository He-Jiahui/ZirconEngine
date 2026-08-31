use std::error::Error as StdError;
use std::fmt;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::core::jobs::{EditorJobSpec, JobError, JobSubmitError};

use super::super::{
    AutosaveDocumentId, AutosaveExtension, AutosaveJobPolicy, AutosaveSnapshotProvenance,
    AutosaveSourcePath, AutosaveStore,
};
use super::write_job::AutosaveWriteJob;

pub const DEFAULT_AUTOSAVE_COMPLETION_BUDGET: usize = 64;

pub struct AutosaveSnapshot {
    pub(super) sequence: u64,
    pub(super) extension: AutosaveExtension,
    pub(super) source_path: AutosaveSourcePath,
    pub(super) provenance: AutosaveSnapshotProvenance,
    pub(super) bytes: Vec<u8>,
}

impl AutosaveSnapshot {
    pub fn new(
        sequence: u64,
        extension: AutosaveExtension,
        source_path: AutosaveSourcePath,
        provenance: AutosaveSnapshotProvenance,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            sequence,
            extension,
            source_path,
            provenance,
            bytes,
        }
    }
}

/// The one document/transaction owner supplies a snapshot only when a worker
/// has been admitted. It never writes the authoritative source file here.
pub trait AutosaveSnapshotSource: Send + Sync + 'static {
    /// Supplies the immutable source identity before a worker is admitted.
    ///
    /// Completion diagnostics need a document/source binding even when capture
    /// fails before it can produce snapshot bytes.
    fn source_path(&self) -> AutosaveSourcePath;

    fn capture(&self, document: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError>;
}

/// A light admission intent. The source remains opaque until the job executes;
/// no serialized payload is retained in the pending queue.
#[derive(Clone)]
pub struct AutosaveDocumentRequest {
    pub(super) document: AutosaveDocumentId,
    pub(super) source_path: AutosaveSourcePath,
    pub(super) policy: AutosaveJobPolicy,
    pub(super) source: Arc<dyn AutosaveSnapshotSource>,
    pub(super) estimated_pending_bytes: usize,
}

impl AutosaveDocumentRequest {
    pub fn new(
        document: AutosaveDocumentId,
        policy: AutosaveJobPolicy,
        source: Arc<dyn AutosaveSnapshotSource>,
    ) -> Self {
        let source_path = source.source_path();
        Self {
            document,
            source_path,
            policy,
            source,
            estimated_pending_bytes: 1,
        }
    }

    pub(crate) fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub(crate) fn source_path(&self) -> &AutosaveSourcePath {
        &self.source_path
    }

    pub(crate) const fn estimated_pending_bytes(&self) -> usize {
        self.estimated_pending_bytes
    }

    pub(super) fn into_job(self, store: AutosaveStore) -> (EditorJobSpec, AutosaveWriteJob) {
        let spec = self
            .policy
            .build_job_spec(&self.document)
            .with_estimated_bytes(self.estimated_pending_bytes);
        let job = AutosaveWriteJob {
            document: self.document,
            source_path: self.source_path,
            source: self.source,
            store,
        };
        (spec, job)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveWriteResult {
    pub(super) document: AutosaveDocumentId,
    pub(super) snapshot_path: PathBuf,
    pub(super) diagnostic_persisted: bool,
}

impl AutosaveWriteResult {
    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub fn snapshot_path(&self) -> &std::path::Path {
        &self.snapshot_path
    }
}

/// The write boundary at which an autosave terminal result was decided.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutosaveFailureStage {
    Capture,
    SourceIdentity,
    Sequence,
    SnapshotCommit,
    Retention,
    DiagnosticPersistence,
    Cancelled,
    JobLifecycle,
}

impl fmt::Display for AutosaveFailureStage {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Capture => "capture",
            Self::SourceIdentity => "source_identity",
            Self::Sequence => "sequence",
            Self::SnapshotCommit => "snapshot_commit",
            Self::Retention => "retention",
            Self::DiagnosticPersistence => "diagnostic_persistence",
            Self::Cancelled => "cancelled",
            Self::JobLifecycle => "job_lifecycle",
        })
    }
}

/// Whether a document-level autosave failure may be offered for retry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutosaveRetryability {
    Retryable,
    NotRetryable,
}

/// One terminal result from a document-bound autosave ticket.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AutosaveDocumentOutcome {
    document: AutosaveDocumentId,
    source_path: AutosaveSourcePath,
    kind: AutosaveDocumentOutcomeKind,
    diagnostic_persisted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum AutosaveDocumentOutcomeKind {
    Saved {
        snapshot_path: PathBuf,
    },
    Failed {
        stage: AutosaveFailureStage,
        retryability: AutosaveRetryability,
        error_chain: Vec<String>,
        usable_snapshot: Option<PathBuf>,
    },
    Cancelled {
        stage: AutosaveFailureStage,
        retryability: AutosaveRetryability,
        error_chain: Vec<String>,
    },
}

impl AutosaveDocumentOutcome {
    pub(super) fn saved(
        document: AutosaveDocumentId,
        source_path: AutosaveSourcePath,
        snapshot_path: PathBuf,
        diagnostic_persisted: bool,
    ) -> Self {
        Self {
            document,
            source_path,
            kind: AutosaveDocumentOutcomeKind::Saved { snapshot_path },
            diagnostic_persisted,
        }
    }

    pub(super) fn failed(
        document: AutosaveDocumentId,
        source_path: AutosaveSourcePath,
        failure: &AutosaveWriteFailure,
    ) -> Self {
        Self {
            document,
            source_path,
            kind: AutosaveDocumentOutcomeKind::Failed {
                stage: failure.stage,
                retryability: failure.retryability,
                error_chain: failure.error_chain.clone(),
                usable_snapshot: failure.usable_snapshot.clone(),
            },
            diagnostic_persisted: failure.diagnostic_persisted,
        }
    }

    pub(super) fn from_ticket_result(
        document: AutosaveDocumentId,
        source_path: AutosaveSourcePath,
        result: Result<AutosaveWriteResult, JobError>,
    ) -> Self {
        match result {
            Ok(result) => Self::saved(
                document,
                source_path,
                result.snapshot_path,
                result.diagnostic_persisted,
            ),
            Err(error) => {
                if let Some(failure) = error.downcast_ref::<AutosaveWriteFailure>() {
                    return Self::failed(document, source_path, failure);
                }
                match &error {
                    JobError::Cancelled => Self {
                        document,
                        source_path,
                        kind: AutosaveDocumentOutcomeKind::Cancelled {
                            stage: AutosaveFailureStage::Cancelled,
                            retryability: AutosaveRetryability::NotRetryable,
                            error_chain: error_chain(&error),
                        },
                        diagnostic_persisted: false,
                    },
                    _ => Self {
                        document,
                        source_path,
                        kind: AutosaveDocumentOutcomeKind::Failed {
                            stage: AutosaveFailureStage::JobLifecycle,
                            retryability: AutosaveRetryability::Retryable,
                            error_chain: error_chain(&error),
                            usable_snapshot: None,
                        },
                        diagnostic_persisted: false,
                    },
                }
            }
        }
    }

    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub fn source_path(&self) -> &AutosaveSourcePath {
        &self.source_path
    }

    pub fn kind(&self) -> &AutosaveDocumentOutcomeKind {
        &self.kind
    }

    pub fn failure_stage(&self) -> Option<AutosaveFailureStage> {
        match &self.kind {
            AutosaveDocumentOutcomeKind::Saved { .. } => None,
            AutosaveDocumentOutcomeKind::Failed { stage, .. }
            | AutosaveDocumentOutcomeKind::Cancelled { stage, .. } => Some(*stage),
        }
    }

    pub fn retryability(&self) -> AutosaveRetryability {
        match &self.kind {
            AutosaveDocumentOutcomeKind::Saved { .. } => AutosaveRetryability::NotRetryable,
            AutosaveDocumentOutcomeKind::Failed { retryability, .. }
            | AutosaveDocumentOutcomeKind::Cancelled { retryability, .. } => *retryability,
        }
    }

    pub fn usable_snapshot(&self) -> Option<&Path> {
        match &self.kind {
            AutosaveDocumentOutcomeKind::Saved { snapshot_path } => Some(snapshot_path),
            AutosaveDocumentOutcomeKind::Failed {
                usable_snapshot, ..
            } => usable_snapshot.as_deref(),
            AutosaveDocumentOutcomeKind::Cancelled { .. } => None,
        }
    }

    pub fn error_chain(&self) -> &[String] {
        match &self.kind {
            AutosaveDocumentOutcomeKind::Saved { .. } => &[],
            AutosaveDocumentOutcomeKind::Failed { error_chain, .. }
            | AutosaveDocumentOutcomeKind::Cancelled { error_chain, .. } => error_chain,
        }
    }

    pub const fn diagnostic_persisted(&self) -> bool {
        self.diagnostic_persisted
    }

    pub(crate) fn mark_diagnostic_persisted(&mut self) {
        self.diagnostic_persisted = true;
    }

    pub(crate) fn shutdown_deadline(
        document: AutosaveDocumentId,
        source_path: AutosaveSourcePath,
    ) -> Self {
        Self {
            document,
            source_path,
            kind: AutosaveDocumentOutcomeKind::Cancelled {
                stage: AutosaveFailureStage::JobLifecycle,
                retryability: AutosaveRetryability::Retryable,
                error_chain: vec![
                    "editor shutdown deadline elapsed before autosave reached a terminal result"
                        .to_string(),
                ],
            },
            diagnostic_persisted: false,
        }
    }

    pub(crate) fn shutdown_unavailable(
        document: AutosaveDocumentId,
        source_path: AutosaveSourcePath,
    ) -> Self {
        Self {
            document,
            source_path,
            kind: AutosaveDocumentOutcomeKind::Cancelled {
                stage: AutosaveFailureStage::JobLifecycle,
                retryability: AutosaveRetryability::Retryable,
                error_chain: vec![
                    "editor shutdown could not bind final autosave requests to an active project"
                        .to_string(),
                ],
            },
            diagnostic_persisted: false,
        }
    }

    pub(super) fn is_saved(&self) -> bool {
        matches!(&self.kind, AutosaveDocumentOutcomeKind::Saved { .. })
    }
}

/// Bounded monotonic health facts for one project adapter generation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutosaveHealthTelemetry {
    completed: u64,
    succeeded: u64,
    failed: u64,
    cancelled: u64,
}

impl AutosaveHealthTelemetry {
    pub(super) fn observe(&mut self, outcome: &AutosaveDocumentOutcome) {
        self.completed = self.completed.saturating_add(1);
        match &outcome.kind {
            AutosaveDocumentOutcomeKind::Saved { .. } => {
                self.succeeded = self.succeeded.saturating_add(1);
            }
            AutosaveDocumentOutcomeKind::Failed { .. } => {
                self.failed = self.failed.saturating_add(1);
            }
            AutosaveDocumentOutcomeKind::Cancelled { .. } => {
                self.cancelled = self.cancelled.saturating_add(1);
            }
        }
    }

    pub const fn completed(self) -> u64 {
        self.completed
    }

    pub const fn succeeded(self) -> u64 {
        self.succeeded
    }

    pub const fn failed(self) -> u64 {
        self.failed
    }

    pub const fn cancelled(self) -> u64 {
        self.cancelled
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AutosaveCompletion {
    pub(super) succeeded: usize,
    pub(super) failed: usize,
    pub(super) pending: usize,
    pub(super) inspected_tickets: usize,
    pub(super) outcomes: Vec<AutosaveDocumentOutcome>,
    pub(super) health: AutosaveHealthTelemetry,
}

impl AutosaveCompletion {
    pub const fn succeeded(&self) -> usize {
        self.succeeded
    }

    pub const fn failed(&self) -> usize {
        self.failed
    }

    pub const fn pending(&self) -> usize {
        self.pending
    }

    pub const fn inspected_tickets(&self) -> usize {
        self.inspected_tickets
    }

    /// Contains only terminal results newly observed by this poll. Counts above
    /// remain cumulative for the current admitted batch.
    pub fn outcomes(&self) -> &[AutosaveDocumentOutcome] {
        &self.outcomes
    }

    pub(crate) fn outcomes_mut(&mut self) -> &mut [AutosaveDocumentOutcome] {
        &mut self.outcomes
    }

    pub const fn health(&self) -> AutosaveHealthTelemetry {
        self.health
    }
}

#[derive(Debug, Error)]
pub enum AutosaveAdmissionError {
    #[error("autosave adapter is shutting down and no longer accepts admissions")]
    ShuttingDown,
    #[error("autosave plan requires `{document}`, but no request was supplied")]
    MissingRequest { document: String },
    #[error("autosave plan requires `{expected}`, but the request source returned `{actual}`")]
    MismatchedRequest { expected: String, actual: String },
    #[error(transparent)]
    JobSubmit(#[from] JobSubmitError),
}

/// Bridges Editor17's immutable autosave scheduler to the one Editor14 job
/// system. It owns no workers and performs no synchronous serialization.

const MAX_AUTOSAVE_ERROR_CHAIN_ENTRIES: usize = 8;
const MAX_AUTOSAVE_ERROR_MESSAGE_BYTES: usize = 1024;

#[derive(Clone, Debug)]
pub(super) struct AutosaveWriteFailure {
    stage: AutosaveFailureStage,
    retryability: AutosaveRetryability,
    error_chain: Vec<String>,
    usable_snapshot: Option<PathBuf>,
    diagnostic_persisted: bool,
}

impl AutosaveWriteFailure {
    pub(super) fn from_error(
        stage: AutosaveFailureStage,
        error: &(dyn StdError + 'static),
    ) -> Self {
        Self {
            stage,
            retryability: match stage {
                AutosaveFailureStage::Cancelled => AutosaveRetryability::NotRetryable,
                _ => AutosaveRetryability::Retryable,
            },
            error_chain: error_chain(error),
            usable_snapshot: None,
            diagnostic_persisted: true,
        }
    }

    pub(super) fn from_job_error(stage: AutosaveFailureStage, error: JobError) -> Self {
        Self::from_error(stage, &error)
    }

    pub(super) fn from_autosave_error(
        stage: AutosaveFailureStage,
        error: crate::core::recovery::AutosaveError,
    ) -> Self {
        let mut failure = Self::from_error(stage, &error);
        if let crate::core::recovery::AutosaveError::RotationAfterWrite { snapshot, .. } = &error {
            failure.stage = AutosaveFailureStage::Retention;
            failure.usable_snapshot = Some(snapshot.clone());
        }
        failure
    }

    pub(super) fn source_identity_changed() -> Self {
        Self {
            stage: AutosaveFailureStage::SourceIdentity,
            retryability: AutosaveRetryability::Retryable,
            error_chain: vec!["autosave source identity changed after admission".to_string()],
            usable_snapshot: None,
            diagnostic_persisted: true,
        }
    }

    pub(super) fn with_diagnostic_persistence_failure(
        mut self,
        error: &(dyn StdError + 'static),
    ) -> Self {
        self.diagnostic_persisted = false;
        self.error_chain.extend(
            error_chain(error)
                .into_iter()
                .map(|message| format!("autosave diagnostic persistence: {message}")),
        );
        self.error_chain.truncate(MAX_AUTOSAVE_ERROR_CHAIN_ENTRIES);
        self
    }

    pub(super) fn with_usable_snapshot(mut self, snapshot_path: PathBuf) -> Self {
        self.usable_snapshot = Some(snapshot_path);
        self
    }
}

impl fmt::Display for AutosaveWriteFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = self
            .error_chain
            .first()
            .map(String::as_str)
            .unwrap_or("autosave write failed without an error message");
        write!(formatter, "autosave {} failed: {message}", self.stage)
    }
}

impl StdError for AutosaveWriteFailure {}

fn error_chain(error: &(dyn StdError + 'static)) -> Vec<String> {
    let mut chain = Vec::new();
    let mut current = Some(error);
    while let Some(error) = current {
        if chain.len() == MAX_AUTOSAVE_ERROR_CHAIN_ENTRIES {
            break;
        }
        chain.push(truncate_error_message(&error.to_string()));
        current = error.source();
    }
    chain
}

fn truncate_error_message(message: &str) -> String {
    if message.len() <= MAX_AUTOSAVE_ERROR_MESSAGE_BYTES {
        return message.to_string();
    }
    let mut boundary = MAX_AUTOSAVE_ERROR_MESSAGE_BYTES.saturating_sub(3);
    while !message.is_char_boundary(boundary) {
        boundary = boundary.saturating_sub(1);
    }
    format!("{}...", &message[..boundary])
}
