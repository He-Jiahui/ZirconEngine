use std::sync::{Arc, Mutex, MutexGuard, OnceLock};

use thiserror::Error;

use crate::asset::project::{ResolvedProjectPath, ResolvedProjectPathIdentity};
use crate::core::runtime::{
    BoundedKeyedIoAdmissionError, BoundedKeyedIoCancelAuthority, BoundedKeyedIoCancelError,
    BoundedKeyedIoDiagnostics, BoundedKeyedIoFailure, BoundedKeyedIoKey, BoundedKeyedIoLane,
    BoundedKeyedIoLimits, BoundedKeyedIoShutdownGuard, BoundedKeyedIoTicket,
    BoundedKeyedIoWorkDeadline, JobScheduler,
};
use crate::core::{CoreHandle, CoreWeak};

use super::super::{
    MAX_RUNTIME_SESSION_ARCHIVE_ARTIFACT_BYTES, RuntimeSessionArchiveArtifact,
    RuntimeSessionArchiveError,
};
use super::atomic::{
    admit_archive_path_write, reserve_archive_path_write, save_artifact_to_prepared_path_atomically,
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
    cancel_authority: BoundedKeyedIoCancelAuthority,
    outcome: Arc<Mutex<Option<Result<(), RuntimeSessionArchiveError>>>>,
}

#[derive(Debug, Error)]
pub enum RuntimeSessionArchiveWriterSubmitError {
    #[error("runtime session archive writer runtime task owner is unavailable")]
    RuntimeUnavailable,
    #[error("runtime session archive writer path authority rejected the write: {0}")]
    PathAuthority(#[from] RuntimeSessionArchiveError),
    #[error("runtime session archive writer admission failed: {0:?}")]
    Admission(BoundedKeyedIoAdmissionError),
}

impl RuntimeSessionArchiveWriteSubmission {
    pub fn ticket(&self) -> BoundedKeyedIoTicket {
        self.ticket.clone()
    }

    pub fn cancel_before_start(&self) -> Result<(), BoundedKeyedIoCancelError> {
        self.ticket.cancel_before_start(&self.cancel_authority)
    }

    pub fn take_outcome(&self) -> Option<Result<(), RuntimeSessionArchiveError>> {
        lock(&self.outcome).take()
    }
}

pub struct RuntimeSessionArchiveWriter {
    lane: BoundedKeyedIoLane,
    owner: RuntimeSessionArchiveWriterOwner,
}

enum RuntimeSessionArchiveWriterOwner {
    Runtime(CoreWeak),
    #[cfg(test)]
    Fixture,
}

impl RuntimeSessionArchiveWriter {
    pub fn with_runtime(limits: RuntimeSessionArchiveWriterLimits, runtime: &CoreHandle) -> Self {
        let scheduler = JobScheduler::from_pool(runtime.task_graph().worker_pool().clone());
        Self::with_owner(
            limits,
            scheduler,
            RuntimeSessionArchiveWriterOwner::Runtime(runtime.downgrade()),
        )
    }

    #[cfg(test)]
    pub(crate) fn with_scheduler(
        limits: RuntimeSessionArchiveWriterLimits,
        scheduler: JobScheduler,
    ) -> Self {
        Self::with_owner(limits, scheduler, RuntimeSessionArchiveWriterOwner::Fixture)
    }

    fn with_owner(
        limits: RuntimeSessionArchiveWriterLimits,
        scheduler: JobScheduler,
        owner: RuntimeSessionArchiveWriterOwner,
    ) -> Self {
        Self {
            lane: BoundedKeyedIoLane::new(
                BoundedKeyedIoLimits::new(limits.max_entries, limits.max_retained_bytes),
                scheduler,
            ),
            owner,
        }
    }

    pub fn try_submit(
        &self,
        artifact: RuntimeSessionArchiveArtifact,
        target: ResolvedProjectPath,
        deadline: BoundedKeyedIoWorkDeadline,
    ) -> Result<RuntimeSessionArchiveWriteSubmission, RuntimeSessionArchiveWriterSubmitError> {
        let runtime_admission_lease = match &self.owner {
            RuntimeSessionArchiveWriterOwner::Runtime(runtime) => Some(
                runtime
                    .upgrade()
                    .ok_or(RuntimeSessionArchiveWriterSubmitError::RuntimeUnavailable)?,
            ),
            #[cfg(test)]
            RuntimeSessionArchiveWriterOwner::Fixture => None,
        };
        let retained_bytes = retained_write_bytes(&artifact, &target).ok_or(
            RuntimeSessionArchiveWriterSubmitError::Admission(
                BoundedKeyedIoAdmissionError::RetainedBytesOverflow,
            ),
        )?;
        let path_identity = ResolvedProjectPathIdentity::from(target.clone());
        let key = BoundedKeyedIoKey::from_value(path_identity.clone());
        let write_reservation = reserve_archive_path_write(&artifact, path_identity)?;
        let write_generation = write_reservation.write_generation();
        let write_ticket = Arc::new(OnceLock::new());
        let write_ticket_for_work = Arc::clone(&write_ticket);
        let artifact_for_work = artifact.clone();
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
                    let write_ticket = write_ticket_for_work
                        .get()
                        .expect("activated archive write must have an admitted path ticket");
                    let result = save_artifact_to_prepared_path_atomically(
                        &artifact_for_work,
                        &target,
                        write_ticket,
                    );
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
        let cancel_authority = admission.cancel_authority();
        let admitted_ticket = admit_archive_path_write(&artifact, write_reservation)?;
        assert!(
            write_ticket.set(admitted_ticket).is_ok(),
            "archive write path ticket must be published exactly once"
        );
        let ticket = admission.activate();
        drop(runtime_admission_lease);
        Ok(RuntimeSessionArchiveWriteSubmission {
            ticket,
            cancel_authority,
            outcome,
        })
    }

    pub fn diagnostics(&self) -> BoundedKeyedIoDiagnostics {
        self.lane.diagnostics()
    }

    pub fn shutdown(&self) -> BoundedKeyedIoShutdownGuard {
        self.lane.shutdown()
    }
}

fn retained_write_bytes(
    artifact: &RuntimeSessionArchiveArtifact,
    path: &ResolvedProjectPath,
) -> Option<usize> {
    artifact
        .serialized_bytes()
        .len()
        .checked_add(path.operation_path().as_os_str().len())?
        .checked_add(ARCHIVE_WRITER_METADATA_BYTES)
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

#[cfg(test)]
mod tests;
