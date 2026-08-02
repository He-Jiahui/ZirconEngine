use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;

use crate::core::jobs::{
    EditorJob, EditorJobSpec, EditorJobSystem, JobContext, JobError, JobId, JobSubmitError,
    JobTicket,
};

use super::{
    AutosaveDocumentId, AutosaveDocumentState, AutosaveExtension, AutosaveJobPolicy, AutosavePlan,
    AutosaveScheduler, AutosaveStore,
};

/// The immutable bytes produced by the document authority after a queued
/// autosave ticket has started. They are intentionally absent from admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveSnapshot {
    sequence: u64,
    extension: AutosaveExtension,
    bytes: Vec<u8>,
}

impl AutosaveSnapshot {
    pub fn new(sequence: u64, extension: AutosaveExtension, bytes: Vec<u8>) -> Self {
        Self {
            sequence,
            extension,
            bytes,
        }
    }
}

/// The one document/transaction owner supplies a snapshot only when a worker
/// has been admitted. It never writes the authoritative source file here.
pub trait AutosaveSnapshotSource: Send + Sync + 'static {
    fn capture(&self, document: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError>;
}

/// A light admission intent. The source remains opaque until the job executes;
/// no serialized payload is retained in the pending queue.
pub struct AutosaveDocumentRequest {
    document: AutosaveDocumentId,
    policy: AutosaveJobPolicy,
    source: Arc<dyn AutosaveSnapshotSource>,
    estimated_pending_bytes: usize,
}

impl AutosaveDocumentRequest {
    pub fn new(
        document: AutosaveDocumentId,
        policy: AutosaveJobPolicy,
        source: Arc<dyn AutosaveSnapshotSource>,
    ) -> Self {
        Self {
            document,
            policy,
            source,
            estimated_pending_bytes: 1,
        }
    }

    pub fn with_estimated_pending_bytes(mut self, estimated_pending_bytes: usize) -> Self {
        self.estimated_pending_bytes = estimated_pending_bytes.max(1);
        self
    }

    fn into_job(self, store: AutosaveStore) -> (EditorJobSpec, AutosaveWriteJob) {
        let spec = self
            .policy
            .build_job_spec(&self.document)
            .with_estimated_bytes(self.estimated_pending_bytes);
        let job = AutosaveWriteJob {
            document: self.document,
            source: self.source,
            store,
        };
        (spec, job)
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveWriteResult {
    document: AutosaveDocumentId,
    snapshot_path: std::path::PathBuf,
}

impl AutosaveWriteResult {
    pub fn document(&self) -> &AutosaveDocumentId {
        &self.document
    }

    pub fn snapshot_path(&self) -> &std::path::Path {
        &self.snapshot_path
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct AutosaveCompletion {
    succeeded: usize,
    failed: usize,
    pending: usize,
}

impl AutosaveCompletion {
    pub const fn succeeded(self) -> usize {
        self.succeeded
    }

    pub const fn failed(self) -> usize {
        self.failed
    }

    pub const fn pending(self) -> usize {
        self.pending
    }
}

#[derive(Debug, Error)]
pub enum AutosaveAdmissionError {
    #[error("autosave adapter is shutting down and no longer accepts admissions")]
    ShuttingDown,
    #[error("autosave plan references `{document}` more than once")]
    DuplicateRequest { document: String },
    #[error("autosave plan requires `{document}`, but no request was supplied")]
    MissingRequest { document: String },
    #[error("autosave request `{document}` was not present in the due plan")]
    UnexpectedRequest { document: String },
    #[error(transparent)]
    JobSubmit(#[from] JobSubmitError),
}

/// Bridges Editor17's immutable autosave scheduler to the one Editor14 job
/// system. It owns no workers and performs no synchronous serialization.
pub struct AutosaveJobAdapter {
    jobs: EditorJobSystem,
    store: AutosaveStore,
    scheduler: AutosaveScheduler,
    tickets: Vec<JobTicket<AutosaveWriteResult>>,
    accepting: bool,
}

impl AutosaveJobAdapter {
    pub fn new(jobs: EditorJobSystem, store: AutosaveStore, scheduler: AutosaveScheduler) -> Self {
        Self {
            jobs,
            store,
            scheduler,
            tickets: Vec::new(),
            accepting: true,
        }
    }

    pub const fn is_accepting(&self) -> bool {
        self.accepting
    }

    pub const fn is_in_flight(&self) -> bool {
        self.scheduler.is_in_flight()
    }

    /// Plans and admits every due document as one atomic admission group.
    ///
    /// A rejected group releases the scheduler immediately, so a later tick can
    /// retry. Once admitted, all ticket terminal states advance the next normal
    /// interval; individual write failures never pin scheduler single-flight.
    pub fn schedule(
        &mut self,
        now: Duration,
        documents: &[AutosaveDocumentState],
        requests: impl IntoIterator<Item = AutosaveDocumentRequest>,
    ) -> Result<bool, AutosaveAdmissionError> {
        if !self.accepting {
            return Err(AutosaveAdmissionError::ShuttingDown);
        }
        let Some(plan) = self.scheduler.plan(now, documents) else {
            return Ok(false);
        };

        let requests = match requests_for_plan(&plan, requests) {
            Ok(requests) => requests,
            Err(error) => {
                self.scheduler.mark_submission_failed();
                return Err(error);
            }
        };
        let jobs = requests
            .into_iter()
            .map(|request| request.into_job(self.store.clone()))
            .collect::<Vec<_>>();
        match self.jobs.submit_batch(jobs) {
            Ok(tickets) => {
                self.tickets = tickets;
                Ok(true)
            }
            Err(error) => {
                self.scheduler.mark_submission_failed();
                Err(error.into())
            }
        }
    }

    /// Applies completed worker results without blocking the caller. When the
    /// last admitted ticket reaches any terminal state, the scheduler advances
    /// exactly once from this supplied editor-clock instant.
    pub fn pump_completed(&mut self, now: Duration) -> AutosaveCompletion {
        let mut completion = AutosaveCompletion::default();
        let mut pending = Vec::with_capacity(self.tickets.len());
        for ticket in std::mem::take(&mut self.tickets) {
            match ticket.try_take() {
                Some(Ok(_)) => completion.succeeded += 1,
                Some(Err(_)) => completion.failed += 1,
                None => pending.push(ticket),
            }
        }
        completion.pending = pending.len();
        self.tickets = pending;
        if completion.pending == 0 && (completion.succeeded != 0 || completion.failed != 0) {
            self.scheduler.mark_finished(now);
        }
        completion
    }

    /// Stops future admission and requests cooperative cancellation for every
    /// owned ticket. Global EditorJobSystem shutdown remains the deadline owner.
    pub fn begin_shutdown(&mut self) -> Vec<JobId> {
        self.accepting = false;
        let ids = self.tickets.iter().map(JobTicket::id).collect::<Vec<_>>();
        for id in &ids {
            self.jobs.cancel(*id);
        }
        ids
    }
}

fn requests_for_plan(
    plan: &AutosavePlan,
    requests: impl IntoIterator<Item = AutosaveDocumentRequest>,
) -> Result<Vec<AutosaveDocumentRequest>, AutosaveAdmissionError> {
    let mut by_document = BTreeMap::new();
    for request in requests {
        let document = request.document.clone();
        if by_document.insert(document.clone(), request).is_some() {
            return Err(AutosaveAdmissionError::DuplicateRequest {
                document: document.as_str().to_string(),
            });
        }
    }

    let mut ordered = Vec::with_capacity(plan.documents().len());
    for document in plan.documents() {
        let request =
            by_document
                .remove(document)
                .ok_or_else(|| AutosaveAdmissionError::MissingRequest {
                    document: document.as_str().to_string(),
                })?;
        ordered.push(request);
    }
    if let Some((document, _)) = by_document.into_iter().next() {
        return Err(AutosaveAdmissionError::UnexpectedRequest {
            document: document.as_str().to_string(),
        });
    }
    Ok(ordered)
}

struct AutosaveWriteJob {
    document: AutosaveDocumentId,
    source: Arc<dyn AutosaveSnapshotSource>,
    store: AutosaveStore,
}

impl EditorJob for AutosaveWriteJob {
    type Output = AutosaveWriteResult;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        context.check_cancelled()?;
        let snapshot = self.source.capture(&self.document)?;
        context.check_cancelled()?;
        let snapshot_path = self
            .store
            .write_snapshot(
                &self.document,
                snapshot.sequence,
                &snapshot.extension,
                &snapshot.bytes,
            )
            .map_err(JobError::failed)?;
        Ok(AutosaveWriteResult {
            document: self.document,
            snapshot_path,
        })
    }
}
