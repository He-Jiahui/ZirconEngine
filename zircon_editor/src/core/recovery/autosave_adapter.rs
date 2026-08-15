use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;

use crate::core::jobs::{
    EditorJob, EditorJobAdmissionRequest, EditorJobSpec, EditorJobSystem, JobCategory, JobContext,
    JobError, JobId, JobPriority, JobSubmitError, JobTicket,
};

use super::{
    AutosaveDocumentId, AutosaveDocumentState, AutosaveExtension, AutosaveJobPolicy, AutosavePlan,
    AutosaveScheduler, AutosaveSourcePath, AutosaveStore,
};

pub const DEFAULT_AUTOSAVE_COMPLETION_BUDGET: usize = 64;

/// The immutable bytes produced by the document authority after a queued
/// autosave ticket has started. They are intentionally absent from admission.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AutosaveSnapshot {
    sequence: u64,
    extension: AutosaveExtension,
    source_path: AutosaveSourcePath,
    bytes: Vec<u8>,
}

impl AutosaveSnapshot {
    pub fn new(
        sequence: u64,
        extension: AutosaveExtension,
        source_path: AutosaveSourcePath,
        bytes: Vec<u8>,
    ) -> Self {
        Self {
            sequence,
            extension,
            source_path,
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
    inspected_tickets: usize,
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

    pub const fn inspected_tickets(self) -> usize {
        self.inspected_tickets
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
pub struct AutosaveJobAdapter {
    jobs: EditorJobSystem,
    store: AutosaveStore,
    scheduler: AutosaveScheduler,
    tickets: VecDeque<JobTicket<AutosaveWriteResult>>,
    completed_succeeded: usize,
    completed_failed: usize,
    next_document_after: Option<AutosaveDocumentId>,
    accepting: bool,
}

impl AutosaveJobAdapter {
    pub fn new(jobs: EditorJobSystem, store: AutosaveStore, scheduler: AutosaveScheduler) -> Self {
        Self {
            jobs,
            store,
            scheduler,
            tickets: VecDeque::new(),
            completed_succeeded: 0,
            completed_failed: 0,
            next_document_after: None,
            accepting: true,
        }
    }

    pub const fn is_accepting(&self) -> bool {
        self.accepting
    }

    pub const fn is_in_flight(&self) -> bool {
        self.scheduler.is_in_flight()
    }

    pub(crate) const fn is_due(&self, now: Duration) -> bool {
        self.scheduler.is_due(now)
    }

    pub(crate) fn preflight_schedule(&self, now: Duration) -> Result<bool, AutosaveAdmissionError> {
        if !self.accepting {
            return Err(AutosaveAdmissionError::ShuttingDown);
        }
        if !self.scheduler.is_due(now) {
            return Ok(false);
        }
        let admission_window = self.jobs.pending_admission_window()?;
        Ok(admission_window.remaining_entries() != 0
            && admission_window.remaining_estimated_bytes() != 0)
    }

    /// Plans and admits one bounded due-document window as an atomic group.
    ///
    /// It reserves the selected entry and byte set before resolving request
    /// sources or constructing job payloads. A rejected group releases the
    /// scheduler immediately, so a later tick can retry. Once admitted, all
    /// ticket terminal states advance the next normal interval; individual
    /// write failures never pin scheduler single-flight.
    pub fn schedule(
        &mut self,
        now: Duration,
        documents: &[AutosaveDocumentState],
        mut estimated_bytes_for: impl FnMut(&AutosaveDocumentId) -> usize,
        mut request_for: impl FnMut(&AutosaveDocumentId) -> Option<AutosaveDocumentRequest>,
    ) -> Result<bool, AutosaveAdmissionError> {
        if !self.accepting {
            return Err(AutosaveAdmissionError::ShuttingDown);
        }
        if !self.scheduler.is_due(now) || !documents.iter().any(AutosaveDocumentState::is_dirty) {
            return Ok(false);
        }
        let admission_window = self.jobs.pending_admission_window()?;
        let Some(plan) = self.scheduler.plan_window(
            now,
            documents,
            admission_window.remaining_entries(),
            self.next_document_after.as_ref(),
        ) else {
            return Ok(false);
        };

        let selection = select_documents_for_window(
            &plan,
            &mut estimated_bytes_for,
            admission_window.remaining_estimated_bytes(),
            admission_window.pending_estimated_bytes(),
            admission_window.max_pending_estimated_bytes(),
        );
        if selection.documents.is_empty() {
            self.scheduler.mark_submission_failed();
            self.next_document_after = selection.last_examined;
            return Err(selection
                .first_rejection
                .expect("a non-empty autosave plan without selections has a byte rejection")
                .into());
        }
        let selected_documents = selection.documents;
        let reservation = match self.jobs.reserve_batch_admission(
            selected_documents
                .iter()
                .map(|(_, estimated_pending_bytes)| {
                    EditorJobAdmissionRequest::new(JobCategory::Misc, *estimated_pending_bytes)
                        .with_priority(JobPriority::Background)
                })
                .collect(),
        ) {
            Ok(reservation) => reservation,
            Err(error) => {
                self.scheduler.mark_submission_failed();
                return Err(error.into());
            }
        };
        let requests = match requests_for_documents(&selected_documents, &mut request_for) {
            Ok(requests) => requests,
            Err(error) => {
                self.scheduler.mark_submission_failed();
                return Err(error);
            }
        };
        let next_document_after = selection.resume_after_skipped.or(selection.last_examined);
        let jobs = requests
            .into_iter()
            .map(|request| request.into_job(self.store.clone()))
            .collect::<Vec<_>>();
        match reservation.commit(jobs) {
            Ok(tickets) => {
                self.tickets = tickets.into();
                self.completed_succeeded = 0;
                self.completed_failed = 0;
                self.next_document_after = next_document_after;
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
        self.pump_completed_with_budget(now, DEFAULT_AUTOSAVE_COMPLETION_BUDGET)
    }

    pub fn pump_completed_with_budget(
        &mut self,
        now: Duration,
        max_tickets: usize,
    ) -> AutosaveCompletion {
        let inspected_tickets = max_tickets.min(self.tickets.len());
        for _ in 0..inspected_tickets {
            let Some(ticket) = self.tickets.pop_front() else {
                break;
            };
            match ticket.try_take() {
                Some(Ok(_)) => self.completed_succeeded += 1,
                Some(Err(_)) => self.completed_failed += 1,
                None => self.tickets.push_back(ticket),
            }
        }

        let pending = self.tickets.len();
        let terminal =
            pending == 0 && (self.completed_succeeded != 0 || self.completed_failed != 0);
        let completion = AutosaveCompletion {
            succeeded: self.completed_succeeded,
            failed: self.completed_failed,
            pending,
            inspected_tickets,
        };
        if terminal {
            self.scheduler.mark_finished(now);
            self.completed_succeeded = 0;
            self.completed_failed = 0;
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

fn select_documents_for_window(
    plan: &AutosavePlan,
    estimated_bytes_for: &mut impl FnMut(&AutosaveDocumentId) -> usize,
    remaining_estimated_bytes: usize,
    pending_estimated_bytes: usize,
    max_pending_estimated_bytes: usize,
) -> AutosaveDocumentWindowSelection {
    let mut selected = Vec::with_capacity(plan.documents().len());
    let mut estimated_bytes = 0_usize;
    let mut last_examined = None;
    let mut previous_document = plan.documents().last().cloned();
    let mut resume_after_skipped = None;
    let mut first_rejection = None;
    for document in plan.documents() {
        last_examined = Some(document.clone());
        let document_estimated_bytes = estimated_bytes_for(document).max(1);
        let projected_bytes = estimated_bytes.saturating_add(document_estimated_bytes);
        if projected_bytes > remaining_estimated_bytes {
            let temporarily_blocked = document_estimated_bytes <= remaining_estimated_bytes;
            if temporarily_blocked && resume_after_skipped.is_none() {
                resume_after_skipped = previous_document.clone();
            }
            if first_rejection.is_none() {
                first_rejection = Some(JobSubmitError::AdmissionByteLimitExceeded {
                    limit: max_pending_estimated_bytes,
                    current: pending_estimated_bytes,
                    requested: document_estimated_bytes,
                });
            }
            continue;
        }
        estimated_bytes = projected_bytes;
        selected.push((document.clone(), document_estimated_bytes));
        previous_document = Some(document.clone());
    }
    AutosaveDocumentWindowSelection {
        documents: selected,
        last_examined,
        resume_after_skipped,
        first_rejection,
    }
}

struct AutosaveDocumentWindowSelection {
    documents: Vec<(AutosaveDocumentId, usize)>,
    last_examined: Option<AutosaveDocumentId>,
    resume_after_skipped: Option<AutosaveDocumentId>,
    first_rejection: Option<JobSubmitError>,
}

fn requests_for_documents(
    documents: &[(AutosaveDocumentId, usize)],
    request_for: &mut impl FnMut(&AutosaveDocumentId) -> Option<AutosaveDocumentRequest>,
) -> Result<Vec<AutosaveDocumentRequest>, AutosaveAdmissionError> {
    let mut ordered = Vec::with_capacity(documents.len());
    for (document, estimated_pending_bytes) in documents {
        let mut request =
            request_for(document).ok_or_else(|| AutosaveAdmissionError::MissingRequest {
                document: document.as_str().to_string(),
            })?;
        if request.document != *document {
            return Err(AutosaveAdmissionError::MismatchedRequest {
                expected: document.as_str().to_string(),
                actual: request.document.as_str().to_string(),
            });
        }
        request.estimated_pending_bytes = *estimated_pending_bytes;
        ordered.push(request);
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
        let sequence = self
            .store
            .next_sequence(&self.document, snapshot.sequence)
            .map_err(JobError::failed)?;
        let snapshot_path = self
            .store
            .write_snapshot(
                &self.document,
                &snapshot.source_path,
                sequence,
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
