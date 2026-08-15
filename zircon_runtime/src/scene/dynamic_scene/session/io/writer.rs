use std::path::Path;
use std::sync::{Arc, Mutex, MutexGuard};

use thiserror::Error;

use crate::core::runtime::{
    BoundedKeyedIoAdmissionError, BoundedKeyedIoDiagnostics, BoundedKeyedIoFailure,
    BoundedKeyedIoLane, BoundedKeyedIoLimits, BoundedKeyedIoShutdownGuard, BoundedKeyedIoTicket,
    BoundedKeyedIoWorkDeadline, JobScheduler, TaskPools,
};

use super::super::{
    RuntimeSessionArchiveArtifact, RuntimeSessionArchiveError,
    MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES,
};
use super::atomic::{
    archive_path_identity, canonical_archive_target, prepare_archive_path_write,
    save_artifact_to_prepared_path_atomically,
};

const DEFAULT_ARCHIVE_WRITER_ENTRIES: usize = 64;
const ARCHIVE_WRITER_METADATA_BYTES: usize = 256;
const DEFAULT_ARCHIVE_WRITER_METADATA_BUDGET_BYTES: usize = 1024 * 1024;
const ARCHIVE_WRITE_FAILURE: &str = "runtime_session_archive_write_failed";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeSessionArchiveWriterLimits {
    pub max_entries: usize,
    pub max_retained_bytes: usize,
}

impl Default for RuntimeSessionArchiveWriterLimits {
    fn default() -> Self {
        Self {
            max_entries: DEFAULT_ARCHIVE_WRITER_ENTRIES,
            max_retained_bytes: MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES
                .saturating_add(DEFAULT_ARCHIVE_WRITER_METADATA_BUDGET_BYTES),
        }
    }
}

#[derive(Debug)]
pub struct RuntimeSessionArchiveWriteSubmission {
    ticket: BoundedKeyedIoTicket,
    outcome: Arc<Mutex<Option<Result<(), RuntimeSessionArchiveError>>>>,
}

#[derive(Debug, Error)]
pub enum RuntimeSessionArchiveWriterSubmitError {
    #[error("runtime session archive writer could not prepare the target: {0}")]
    Target(#[from] RuntimeSessionArchiveError),
    #[error("runtime session archive writer admission failed: {0:?}")]
    Admission(BoundedKeyedIoAdmissionError),
}

impl RuntimeSessionArchiveWriteSubmission {
    pub fn ticket(&self) -> BoundedKeyedIoTicket {
        self.ticket.clone()
    }

    pub fn take_outcome(&self) -> Option<Result<(), RuntimeSessionArchiveError>> {
        lock(&self.outcome).take()
    }
}

pub struct RuntimeSessionArchiveWriter {
    lane: BoundedKeyedIoLane,
}

impl RuntimeSessionArchiveWriter {
    pub fn new(limits: RuntimeSessionArchiveWriterLimits) -> Self {
        let scheduler = JobScheduler::from_pool(TaskPools::process_default().io().clone());
        Self::with_scheduler(limits, scheduler)
    }

    pub(crate) fn with_scheduler(
        limits: RuntimeSessionArchiveWriterLimits,
        scheduler: JobScheduler,
    ) -> Self {
        Self {
            lane: BoundedKeyedIoLane::new(
                BoundedKeyedIoLimits::new(limits.max_entries, limits.max_retained_bytes),
                scheduler,
            ),
        }
    }

    pub fn try_submit(
        &self,
        artifact: RuntimeSessionArchiveArtifact,
        path: impl AsRef<Path>,
        deadline: BoundedKeyedIoWorkDeadline,
    ) -> Result<RuntimeSessionArchiveWriteSubmission, RuntimeSessionArchiveWriterSubmitError> {
        let target = canonical_archive_target(path.as_ref())?;
        let retained_bytes = retained_write_bytes(&artifact, &target).ok_or(
            RuntimeSessionArchiveWriterSubmitError::Admission(
                BoundedKeyedIoAdmissionError::RetainedBytesOverflow,
            ),
        )?;
        let key: Arc<str> = archive_path_identity(&target).into();
        let write_ticket = prepare_archive_path_write(&artifact, &target)?;
        let write_generation = write_ticket.write_generation();
        let outcome = Arc::new(Mutex::new(None));
        let outcome_for_work = Arc::clone(&outcome);
        let admission = self
            .lane
            .try_admit(
                key,
                write_generation,
                retained_bytes,
                deadline,
                Box::new(move || {
                    let result =
                        save_artifact_to_prepared_path_atomically(&artifact, &target, write_ticket);
                    let terminal = if result.is_ok() {
                        Ok(())
                    } else {
                        Err(BoundedKeyedIoFailure::new(ARCHIVE_WRITE_FAILURE))
                    };
                    *lock(&outcome_for_work) = Some(result);
                    terminal
                }),
            )
            .map_err(RuntimeSessionArchiveWriterSubmitError::Admission)?;
        let ticket = admission.activate();
        Ok(RuntimeSessionArchiveWriteSubmission { ticket, outcome })
    }

    pub fn diagnostics(&self) -> BoundedKeyedIoDiagnostics {
        self.lane.diagnostics()
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        self.lane.shutdown()
    }
}

fn retained_write_bytes(artifact: &RuntimeSessionArchiveArtifact, path: &Path) -> Option<usize> {
    artifact
        .serialized_bytes()
        .len()
        .checked_add(path.as_os_str().len())?
        .checked_add(ARCHIVE_WRITER_METADATA_BYTES)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}
