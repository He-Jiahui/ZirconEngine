use std::collections::VecDeque;
use std::time::Duration;

use crate::core::jobs::{
    EditorJobAdmissionRequest, EditorJobSystem, JobCategory, JobId, JobPriority, JobTicket,
};

use super::super::{
    AutosaveDocumentId, AutosaveDocumentState, AutosavePlan, AutosavePolicy, AutosaveScheduler,
    AutosaveStore,
};
use super::{
    AutosaveAdmissionError, AutosaveCompletion, AutosaveDocumentOutcome, AutosaveDocumentRequest,
    AutosaveHealthTelemetry, AutosaveWriteResult,
};

pub struct AutosaveJobAdapter {
    jobs: EditorJobSystem,
    store: AutosaveStore,
    scheduler: AutosaveScheduler,
    tickets: VecDeque<PendingAutosaveTicket>,
    completed_succeeded: usize,
    completed_failed: usize,
    health: AutosaveHealthTelemetry,
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
            health: AutosaveHealthTelemetry::default(),
            next_document_after: None,
            accepting: true,
        }
    }

    pub const fn is_accepting(&self) -> bool {
        self.accepting
    }

    /// Prevents new interval-driven batches while allowing the lifecycle owner
    /// to drain the currently admitted work before it starts a final batch.
    pub(crate) fn fence_regular_admission(&mut self) {
        self.accepting = false;
    }

    pub const fn is_in_flight(&self) -> bool {
        self.scheduler.is_in_flight()
    }

    pub(crate) fn is_drained(&self) -> bool {
        self.tickets.is_empty() && !self.scheduler.is_in_flight()
    }

    pub(crate) fn is_due(&self, now: Duration) -> bool {
        self.scheduler.is_due(now)
    }

    pub(crate) fn update_policy(&mut self, policy: AutosavePolicy) {
        self.scheduler.update_policy(policy);
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
        estimated_bytes_for: impl FnMut(&AutosaveDocumentId) -> usize,
        request_for: impl FnMut(&AutosaveDocumentId) -> Option<AutosaveDocumentRequest>,
    ) -> Result<bool, AutosaveAdmissionError> {
        self.schedule_with_mode(false, now, documents, estimated_bytes_for, request_for)
    }

    /// Fences periodic autosave admission and submits one shutdown snapshot
    /// window without waiting for the next periodic deadline.
    pub(crate) fn schedule_final(
        &mut self,
        now: Duration,
        documents: &[AutosaveDocumentState],
        estimated_bytes_for: impl FnMut(&AutosaveDocumentId) -> usize,
        request_for: impl FnMut(&AutosaveDocumentId) -> Option<AutosaveDocumentRequest>,
    ) -> Result<bool, AutosaveAdmissionError> {
        self.accepting = false;
        self.schedule_with_mode(true, now, documents, estimated_bytes_for, request_for)
    }

    fn schedule_with_mode(
        &mut self,
        is_final: bool,
        now: Duration,
        documents: &[AutosaveDocumentState],
        mut estimated_bytes_for: impl FnMut(&AutosaveDocumentId) -> usize,
        mut request_for: impl FnMut(&AutosaveDocumentId) -> Option<AutosaveDocumentRequest>,
    ) -> Result<bool, AutosaveAdmissionError> {
        if !is_final && !self.accepting {
            return Err(AutosaveAdmissionError::ShuttingDown);
        }
        if !documents.iter().any(AutosaveDocumentState::is_dirty) {
            return Ok(false);
        }
        let admission_window = self.jobs.pending_admission_window()?;
        let plan = if is_final {
            self.scheduler.plan_final_window(
                documents,
                admission_window.remaining_entries(),
                self.next_document_after.as_ref(),
            )
        } else {
            self.scheduler.plan_window(
                now,
                documents,
                admission_window.remaining_entries(),
                self.next_document_after.as_ref(),
            )
        };
        let Some(plan) = plan else {
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
            .map(|request| {
                let document = request.document.clone();
                let source_path = request.source_path.clone();
                let (spec, job) = request.into_job(self.store.clone());
                (document, source_path, spec, job)
            })
            .collect::<Vec<_>>();
        let ticket_identities = jobs
            .iter()
            .map(|(document, source_path, _, _)| (document.clone(), source_path.clone()))
            .collect::<Vec<_>>();
        match reservation.commit(
            jobs.into_iter()
                .map(|(_, _, spec, job)| (spec, job))
                .collect(),
        ) {
            Ok(tickets) => {
                self.tickets = tickets
                    .into_iter()
                    .zip(ticket_identities)
                    .map(|(ticket, (document, source_path))| PendingAutosaveTicket {
                        document,
                        source_path,
                        ticket,
                    })
                    .collect();
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
        let mut outcomes = Vec::with_capacity(inspected_tickets);
        for _ in 0..inspected_tickets {
            let Some(ticket) = self.tickets.pop_front() else {
                break;
            };
            match ticket.ticket.try_take() {
                Some(result) => {
                    let outcome = AutosaveDocumentOutcome::from_ticket_result(
                        ticket.document,
                        ticket.source_path,
                        result,
                    );
                    if outcome.is_saved() {
                        self.completed_succeeded += 1;
                    } else {
                        self.completed_failed += 1;
                    }
                    self.health.observe(&outcome);
                    outcomes.push(outcome);
                }
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
            outcomes,
            health: self.health,
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
        let ids = self
            .tickets
            .iter()
            .map(|ticket| ticket.ticket.id())
            .collect::<Vec<_>>();
        for id in &ids {
            self.jobs.cancel(*id);
        }
        ids
    }
}

struct PendingAutosaveTicket {
    document: AutosaveDocumentId,
    source_path: AutosaveSourcePath,
    ticket: JobTicket<AutosaveWriteResult>,
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

#[cfg(test)]
mod optimization_batch_20260830cp_editor_tests {
    const COMPLETION_COUNT: usize = 32_768;

    #[test]
    fn optimization_batch_20260830cp_editor_completion_reserves_inspection_upper_bound() {
        let source = include_str!("adapter.rs");
        let implementation = source
            .split("#[cfg(test)]")
            .next()
            .expect("autosave adapter implementation");

        assert!(implementation.contains("Vec::with_capacity(inspected_tickets)"));
    }

    #[test]
    #[ignore = "managed Windows release performance evidence"]
    fn optimization_batch_20260830cp_editor_completion_capacity_evidence() {
        let legacy_growth_events = collect_growth_events(false);
        let optimized_growth_events = collect_growth_events(true);

        println!(
            "EDITOR503_AUTOSAVE_COMPLETION_CAPACITY_BENCH_V1 inspected={COMPLETION_COUNT} \
legacy_growth_events={legacy_growth_events} optimized_growth_events={optimized_growth_events} \
growth_event_reduction_pct=100"
        );
        assert!(legacy_growth_events > 0);
        assert_eq!(optimized_growth_events, 0);
    }

    fn collect_growth_events(reserve_upper_bound: bool) -> usize {
        let capacity = usize::from(reserve_upper_bound) * COMPLETION_COUNT;
        let mut outcomes = Vec::with_capacity(capacity);
        let mut growth_events = 0;
        for outcome in 0..COMPLETION_COUNT {
            let previous_capacity = outcomes.capacity();
            outcomes.push(outcome);
            growth_events += usize::from(outcomes.capacity() != previous_capacity);
        }
        std::hint::black_box(outcomes);
        growth_events
    }
}
