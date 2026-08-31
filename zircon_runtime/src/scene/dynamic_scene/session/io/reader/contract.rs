use std::sync::Arc;

use thiserror::Error;

use crate::core::runtime::{
    BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelError, BoundedKeyedIoDiagnostics,
    BoundedKeyedIoTicket, RetainedByteBudgetError, RetainedByteLease,
};

use super::super::super::{
    MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES, RuntimeSessionArchive, RuntimeSessionArchiveError,
};
use super::service::RuntimeSessionArchiveReadRequest;

const DEFAULT_ARCHIVE_READER_ENTRIES: usize = 8;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSessionArchiveReaderLimits {
    pub max_entries: usize,
    pub max_archive_bytes: usize,
    pub max_retained_result_bytes: usize,
}

impl Default for RuntimeSessionArchiveReaderLimits {
    fn default() -> Self {
        Self {
            max_entries: DEFAULT_ARCHIVE_READER_ENTRIES,
            max_archive_bytes: MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES,
            max_retained_result_bytes: MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RuntimeSessionArchiveReaderDiagnostics {
    pub io: BoundedKeyedIoDiagnostics,
    pub retained_result_bytes: usize,
    pub retained_result_leases: usize,
}

#[derive(Debug, Error)]
pub enum RuntimeSessionArchiveReaderSubmitError {
    #[error("runtime session archive reader is closed")]
    Closed,
    #[error("runtime session archive reader runtime task owner is unavailable")]
    RuntimeUnavailable,
    #[error("runtime session archive reader exhausted its request generation range")]
    GenerationExhausted,
    #[error("runtime session archive reader result budget rejected the request: {0}")]
    ResultBytes(#[from] RetainedByteBudgetError),
    #[error("runtime session archive reader I/O admission failed: {0:?}")]
    Admission(BoundedKeyedIoAdmissionError),
}

#[derive(Clone)]
pub struct RuntimeSessionArchiveReadArtifact {
    archive: RuntimeSessionArchive,
    retained_bytes: RetainedByteLease,
}

impl RuntimeSessionArchiveReadArtifact {
    pub(super) fn new(archive: RuntimeSessionArchive, retained_bytes: RetainedByteLease) -> Self {
        Self {
            archive,
            retained_bytes,
        }
    }

    pub fn archive(&self) -> &RuntimeSessionArchive {
        &self.archive
    }

    pub fn reserved_bytes(&self) -> usize {
        self.retained_bytes.retained_bytes()
    }
}

#[derive(Clone)]
pub enum RuntimeSessionArchiveReadOutcome {
    Succeeded(RuntimeSessionArchiveReadArtifact),
    Failed(Arc<RuntimeSessionArchiveError>),
}

#[derive(Clone)]
pub struct RuntimeSessionArchiveReadSubmission {
    pub(super) request: Arc<RuntimeSessionArchiveReadRequest>,
}

impl RuntimeSessionArchiveReadSubmission {
    pub fn ticket(&self) -> BoundedKeyedIoTicket {
        self.request.ticket.clone()
    }

    /// Cancels the shared path request when it has not started yet.
    pub fn cancel_shared_before_start(&self) -> Result<(), BoundedKeyedIoCancelError> {
        self.request
            .ticket
            .cancel_before_start(&self.request.cancel_authority)
    }

    pub fn outcome(&self) -> Option<RuntimeSessionArchiveReadOutcome> {
        self.request.outcome()
    }
}
