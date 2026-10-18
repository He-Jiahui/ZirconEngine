use std::collections::{BTreeMap, VecDeque};
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::thread;
use std::time::Instant;

use crate::core::jobs::{
    EditorJob, EditorJobShutdownState, EditorJobSpec, JobCategory, JobContext, JobError, JobTicket,
    UnfinishedEditorJob,
};

use super::autosave_service::{
    persist_fallback_diagnostics, ActiveAutosaveProject, AutosaveDiagnosticPersistenceIssue,
    EditorAutosaveService, RetiredAutosaveProject,
};
use super::{
    AutosaveCompletion, AutosaveDocumentId, AutosaveDocumentOutcome, AutosaveDocumentOutcomeKind,
    AutosaveDocumentRequest, AutosaveDocumentState, DEFAULT_AUTOSAVE_COMPLETION_BUDGET,
};

/// Terminal evidence gathered while the editor drains autosave before shared
/// jobs stop. Every outcome is document-bound, including deadline exhaustion.
#[derive(Clone, Debug, Default)]
pub(crate) struct AutosaveShutdownReport {
    outcomes: Vec<AutosaveDocumentOutcome>,
    diagnostic_persistence_issues: Vec<AutosaveDiagnosticPersistenceIssue>,
    unfinished_jobs: Vec<UnfinishedEditorJob>,
}

impl AutosaveShutdownReport {
    pub(crate) fn outcomes(&self) -> &[AutosaveDocumentOutcome] {
        &self.outcomes
    }

    pub(crate) fn diagnostic_persistence_issues(&self) -> &[AutosaveDiagnosticPersistenceIssue] {
        &self.diagnostic_persistence_issues
    }

    pub(crate) fn unfinished_jobs(&self) -> &[UnfinishedEditorJob] {
        &self.unfinished_jobs
    }
}

#[derive(Clone, Debug)]
pub(crate) enum FinalAutosaveFailure {
    ProjectUnavailable,
    ProjectChanged,
    AdmissionRejected { message: String },
    SubmissionPanicked { message: String },
    DocumentFailed { document: AutosaveDocumentId },
    DocumentCancelled { document: AutosaveDocumentId },
    DiagnosticPersistenceFailed,
    DiagnosticJobFailed { message: String },
    DeadlineExceeded,
    ForeignDrain,
}

impl FinalAutosaveFailure {
    pub(crate) fn message(&self) -> String {
        match self {
            Self::ProjectUnavailable => "final autosave has no active project".to_owned(),
            Self::ProjectChanged => "the final autosave project changed".to_owned(),
            Self::AdmissionRejected { message } | Self::SubmissionPanicked { message } => {
                message.clone()
            }
            Self::DocumentFailed { document } => {
                format!("final autosave failed for {}", document.as_str())
            }
            Self::DocumentCancelled { document } => {
                format!("final autosave was cancelled for {}", document.as_str())
            }
            Self::DiagnosticPersistenceFailed => {
                "final autosave recovery diagnostics could not be persisted".to_owned()
            }
            Self::DiagnosticJobFailed { message } => message.clone(),
            Self::DeadlineExceeded => "the final autosave deadline elapsed".to_owned(),
            Self::ForeignDrain => "the final autosave owner belongs to another service".to_owned(),
        }
    }
}

/// Both the service and the closing UI retain this owner until terminal evidence is inspected.
#[derive(Clone)]
pub(crate) struct FinalAutosaveDrain {
    inner: Arc<Mutex<FinalAutosaveState>>,
}

impl std::fmt::Debug for FinalAutosaveDrain {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let state = self.lock();
        formatter
            .debug_struct("FinalAutosaveDrain")
            .field("complete", &state.complete)
            .field("failure", &state.failure)
            .field("remaining_documents", &state.remaining.len())
            .finish_non_exhaustive()
    }
}

impl FinalAutosaveDrain {
    pub(crate) fn is_complete(&self) -> bool {
        let state = self.lock();
        state.complete && !state.poll_in_flight
    }

    pub(crate) fn succeeded(&self) -> bool {
        let state = self.lock();
        state.complete
            && !state.poll_in_flight
            && state.failure.is_none()
            && state.remaining.is_empty()
            && state.saved_documents == state.expected_documents
            && state.report.diagnostic_persistence_issues.is_empty()
    }

    /// Global cancellation must follow final write success, rather than cancel those writes.
    pub(crate) fn ready_to_shutdown_jobs(&self) -> bool {
        self.succeeded()
    }

    pub(crate) fn failure(&self) -> Option<FinalAutosaveFailure> {
        self.lock().failure.clone()
    }

    pub(crate) fn report(&self) -> AutosaveShutdownReport {
        self.lock().report.clone()
    }

    pub(crate) fn remaining_documents(&self) -> usize {
        self.lock().remaining.len()
    }

    fn lock(&self) -> MutexGuard<'_, FinalAutosaveState> {
        self.inner
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

struct FinalAutosaveState {
    project_root: Option<PathBuf>,
    deadline: Instant,
    remaining: BTreeMap<AutosaveDocumentId, AutosaveDocumentRequest>,
    expected_documents: usize,
    saved_documents: usize,
    final_batch_submitted: bool,
    report: AutosaveShutdownReport,
    failure: Option<FinalAutosaveFailure>,
    complete: bool,
    diagnostics: VecDeque<FinalDiagnosticItem>,
    diagnostic_job: Option<(JobTicket<Vec<FinalDiagnosticResult>>, usize)>,
    diagnostic_failed: bool,
    poll_in_flight: bool,
}

#[derive(Clone)]
struct FinalDiagnosticItem {
    report_index: usize,
    project_root: PathBuf,
    outcome: AutosaveDocumentOutcome,
}

struct FinalDiagnosticResult {
    report_index: usize,
    outcome: AutosaveDocumentOutcome,
    issues: Vec<AutosaveDiagnosticPersistenceIssue>,
}

struct FinalDiagnosticJob {
    items: Vec<FinalDiagnosticItem>,
}

impl EditorJob for FinalDiagnosticJob {
    type Output = Vec<FinalDiagnosticResult>;

    fn run(self, context: JobContext) -> Result<Self::Output, JobError> {
        let mut results = Vec::with_capacity(self.items.len());
        for item in self.items {
            if context.is_cancelled() {
                return Err(JobError::Cancelled);
            }
            let mut outcome = item.outcome;
            let issues = persist_fallback_diagnostics(
                &item.project_root,
                std::slice::from_mut(&mut outcome),
            );
            results.push(FinalDiagnosticResult {
                report_index: item.report_index,
                outcome,
                issues,
            });
        }
        Ok(results)
    }
}

impl FinalAutosaveState {
    fn collect_retired_fallback(
        &mut self,
        retired: &mut RetiredAutosaveProject,
        budget: &mut usize,
    ) {
        while *budget != 0 {
            let Some(outcome) = retired.fallback_diagnostics.pop_front() else {
                break;
            };
            *budget -= 1;
            let report_index = self.report.outcomes.len();
            self.report.outcomes.push(outcome.clone());
            self.diagnostics.push_back(FinalDiagnosticItem {
                report_index,
                project_root: retired.project_root.clone(),
                outcome,
            });
        }
    }

    fn reap_diagnostic_job(&mut self) {
        let finished = self
            .diagnostic_job
            .as_ref()
            .and_then(|(ticket, submitted_count)| {
                ticket
                    .try_take_terminal()
                    .map(|result| (*submitted_count, result))
            });
        let Some((submitted_count, result)) = finished else {
            return;
        };
        self.diagnostic_job = None;
        match result {
            Ok(results) => {
                // The private queue only appends while a ticket is in flight.
                // Acknowledge the exact submitted prefix, never the later tail.
                let persisted = results.len() == submitted_count
                    && self.diagnostics.len() >= submitted_count
                    && results
                        .iter()
                        .zip(self.diagnostics.iter())
                        .all(|(result, item)| {
                            result.report_index == item.report_index
                                && result.issues.is_empty()
                                && result.outcome.diagnostic_persisted()
                        });
                for result in results {
                    if let Some(outcome) = self.report.outcomes.get_mut(result.report_index) {
                        *outcome = result.outcome;
                    }
                    self.report
                        .diagnostic_persistence_issues
                        .extend(result.issues);
                }
                if persisted {
                    self.diagnostics.drain(..submitted_count);
                } else {
                    self.diagnostic_failed = true;
                    self.fail(FinalAutosaveFailure::DiagnosticPersistenceFailed);
                }
            }
            Err(error) => {
                self.diagnostic_failed = true;
                self.fail(FinalAutosaveFailure::DiagnosticJobFailed {
                    message: error.to_string(),
                });
            }
        }
    }

    fn update_complete(
        &mut self,
        active_drained: bool,
        retired_drained: bool,
        fallback_drained: bool,
    ) {
        self.complete = !self.poll_in_flight
            && active_drained
            && retired_drained
            && self.diagnostic_job.is_none()
            && (self.failure.is_some()
                || (self.remaining.is_empty()
                    && self.saved_documents == self.expected_documents
                    && self.diagnostics.is_empty()
                    && fallback_drained));
    }
    fn fail(&mut self, failure: FinalAutosaveFailure) {
        if self.failure.is_none() {
            self.failure = Some(failure);
        }
    }

    fn collect(&mut self, project_root: &Path, completion: &AutosaveCompletion, is_final: bool) {
        for outcome in completion.outcomes() {
            if is_final && self.remaining.contains_key(outcome.document()) {
                match outcome.kind() {
                    AutosaveDocumentOutcomeKind::Saved { .. } => {
                        self.remaining.remove(outcome.document());
                        self.saved_documents += 1;
                    }
                    AutosaveDocumentOutcomeKind::Cancelled { .. } => {
                        self.fail(FinalAutosaveFailure::DocumentCancelled {
                            document: outcome.document().clone(),
                        });
                    }
                    AutosaveDocumentOutcomeKind::Failed { .. } => {
                        self.fail(FinalAutosaveFailure::DocumentFailed {
                            document: outcome.document().clone(),
                        });
                    }
                }
            }
            let report_index = self.report.outcomes.len();
            self.report.outcomes.push(outcome.clone());
            if !outcome.diagnostic_persisted() {
                self.diagnostics.push_back(FinalDiagnosticItem {
                    report_index,
                    project_root: project_root.to_path_buf(),
                    outcome: outcome.clone(),
                });
            }
        }
    }
}

impl EditorAutosaveService {
    /// Fences interval autosave, drains final snapshots while the shared job
    /// system remains live, then performs the global deadline shutdown.
    pub(crate) fn shutdown_with_final_autosave(
        &self,
        requests: Vec<AutosaveDocumentRequest>,
        deadline: Instant,
    ) -> AutosaveShutdownReport {
        let mut report = AutosaveShutdownReport::default();
        let mut remaining = requests
            .into_iter()
            .map(|request| (request.document().clone(), request))
            .collect::<BTreeMap<_, _>>();
        let mut active_project_root = None;
        let mut final_batch_submitted = false;

        {
            let mut state = self.lock_state();
            state.closing = true;
            if let Some(active) = state.active.as_mut() {
                active_project_root = Some(active.project_root.clone());
                active.adapter.fence_regular_admission();
                final_batch_submitted = drain_final_autosave_until_deadline(
                    active,
                    &mut remaining,
                    deadline,
                    &mut report,
                );
            } else if !remaining.is_empty() {
                let unbound_outcomes = remaining
                    .values()
                    .map(|request| {
                        AutosaveDocumentOutcome::shutdown_unavailable(
                            request.document().clone(),
                            request.source_path().clone(),
                        )
                    })
                    .collect::<Vec<_>>();
                remaining.clear();
                report.outcomes.extend(unbound_outcomes);
            }
            if let Some(active) = state.active.as_mut() {
                active.adapter.begin_shutdown();
            }
            for retired in &mut state.retired {
                retired.adapter.begin_shutdown();
            }
        }

        report.unfinished_jobs = self.jobs.shutdown(deadline);

        let mut state = self.lock_state();
        let instant = Instant::now();
        if let Some(active) = state.active.as_mut() {
            let now = instant.saturating_duration_since(active.project_started_at);
            let mut completion = active.adapter.pump_completed(now);
            collect_active_completion(
                &mut report,
                &active.project_root,
                &mut remaining,
                &mut completion,
                final_batch_submitted,
                final_batch_submitted,
            );
        }
        for retired in &mut state.retired {
            collect_retired_completion(&mut report, retired, instant);
        }

        if let Some(project_root) = active_project_root {
            let mut deadline_outcomes = remaining
                .into_values()
                .map(|request| {
                    AutosaveDocumentOutcome::shutdown_deadline(
                        request.document().clone(),
                        request.source_path().clone(),
                    )
                })
                .collect::<Vec<_>>();
            report
                .diagnostic_persistence_issues
                .extend(persist_fallback_diagnostics(
                    &project_root,
                    &mut deadline_outcomes,
                ));
            report.outcomes.extend(deadline_outcomes);
        }
        if self.jobs.shutdown_progress().state() == EditorJobShutdownState::Quiescent
            && state
                .active
                .as_ref()
                .map_or(true, |active| active.adapter.is_drained())
            && state
                .retired
                .iter()
                .all(RetiredAutosaveProject::is_finished)
            && report.diagnostic_persistence_issues.is_empty()
            && report
                .outcomes
                .iter()
                .all(|outcome| matches!(outcome.kind(), AutosaveDocumentOutcomeKind::Saved { .. }))
        {
            state.active = None;
            state.retired.clear();
        }
        report
    }
}

fn drain_final_autosave_until_deadline(
    active: &mut ActiveAutosaveProject,
    remaining: &mut BTreeMap<AutosaveDocumentId, AutosaveDocumentRequest>,
    deadline: Instant,
    report: &mut AutosaveShutdownReport,
) -> bool {
    let mut final_batch_submitted = false;
    loop {
        let instant = Instant::now();
        let now = instant.saturating_duration_since(active.project_started_at);
        let mut completion = active.adapter.pump_completed(now);
        collect_active_completion(
            report,
            &active.project_root,
            remaining,
            &mut completion,
            final_batch_submitted,
            final_batch_submitted,
        );

        if active.adapter.is_drained() && !remaining.is_empty() {
            let documents = remaining
                .keys()
                .cloned()
                .map(|document| AutosaveDocumentState::from_dirty_projection(document, true))
                .collect::<Vec<_>>();
            match active.adapter.schedule_final(
                now,
                &documents,
                |document| {
                    remaining
                        .get(document)
                        .map_or(1, AutosaveDocumentRequest::estimated_pending_bytes)
                },
                |document| remaining.get(document).cloned(),
            ) {
                Ok(true) => {
                    final_batch_submitted = true;
                    continue;
                }
                Ok(false) | Err(_) => {}
            }
        }

        if remaining.is_empty() && active.adapter.is_drained() {
            return final_batch_submitted;
        }
        if instant >= deadline {
            return final_batch_submitted;
        }
        thread::yield_now();
    }
}

fn collect_active_completion(
    report: &mut AutosaveShutdownReport,
    project_root: &std::path::Path,
    remaining: &mut BTreeMap<AutosaveDocumentId, AutosaveDocumentRequest>,
    completion: &mut AutosaveCompletion,
    consume_requests: bool,
    include_in_report: bool,
) {
    report
        .diagnostic_persistence_issues
        .extend(persist_fallback_diagnostics(
            project_root,
            completion.outcomes_mut(),
        ));
    if consume_requests {
        for outcome in completion.outcomes() {
            remaining.remove(outcome.document());
        }
    }
    if include_in_report {
        report
            .outcomes
            .extend(completion.outcomes().iter().cloned());
    }
}

fn collect_retired_completion(
    report: &mut AutosaveShutdownReport,
    retired: &mut RetiredAutosaveProject,
    instant: Instant,
) {
    let now = instant.saturating_duration_since(retired.project_started_at);
    let completion = retired.adapter.pump_completed(now);
    retired.fallback_diagnostics.extend(
        completion
            .outcomes()
            .iter()
            .filter(|outcome| !outcome.diagnostic_persisted())
            .cloned(),
    );
    report
        .outcomes
        .extend(completion.outcomes().iter().cloned());
    report
        .diagnostic_persistence_issues
        .extend(retired.persist_fallback_diagnostics());
}

impl EditorAutosaveService {
    /// Seals ordinary admission. The immutable final batch is supplied by the committing UI owner.
    /// Repeating this request returns the original drain without replacing its batch.
    pub(crate) fn begin_final_autosave(
        &self,
        project_root: Option<&Path>,
        requests: Option<Vec<AutosaveDocumentRequest>>,
        deadline: Instant,
    ) -> FinalAutosaveDrain {
        let mut service = self.lock_state();
        if let Some(drain) = service.final_shutdown.as_ref() {
            return drain.clone();
        }
        let was_closing = service.closing;
        service.closing = true;
        if let Some(active) = service.active.as_mut() {
            active.adapter.fence_regular_admission();
        }
        for retired in &mut service.retired {
            retired.adapter.fence_regular_admission();
        }
        let remaining = requests
            .unwrap_or_default()
            .into_iter()
            .map(|request| (request.document().clone(), request))
            .collect::<BTreeMap<_, _>>();
        let expected_documents = remaining.len();
        let failure = if was_closing {
            Some(FinalAutosaveFailure::AdmissionRejected {
                message: "autosave shutdown was already requested".to_owned(),
            })
        } else if !remaining.is_empty() {
            match (service.active.as_ref(), project_root) {
                (None, _) => Some(FinalAutosaveFailure::ProjectUnavailable),
                (Some(active), Some(root)) if active.project_root.as_path() == root => None,
                _ => Some(FinalAutosaveFailure::ProjectChanged),
            }
        } else {
            None
        };
        let drain = FinalAutosaveDrain {
            inner: Arc::new(Mutex::new(FinalAutosaveState {
                project_root: project_root.map(Path::to_path_buf),
                deadline,
                remaining,
                expected_documents,
                saved_documents: 0,
                final_batch_submitted: false,
                report: AutosaveShutdownReport::default(),
                failure,
                complete: false,
                diagnostics: VecDeque::new(),
                diagnostic_job: None,
                diagnostic_failed: false,
                poll_in_flight: false,
            })),
        };
        service.final_shutdown = Some(drain.clone());
        drain
    }

    /// Inspects bounded terminal work once. Submissions run without either lifecycle mutex.
    pub(crate) fn poll_final_autosave(&self, drain: &mut FinalAutosaveDrain) {
        let mut service = self.lock_state();
        let belongs = service
            .final_shutdown
            .as_ref()
            .is_some_and(|owned| Arc::ptr_eq(&owned.inner, &drain.inner));
        let mut final_state = drain.lock();
        if !belongs {
            final_state.fail(FinalAutosaveFailure::ForeignDrain);
            return;
        }
        if final_state.complete || final_state.poll_in_flight {
            return;
        }
        let instant = Instant::now();
        if instant >= final_state.deadline {
            final_state.fail(FinalAutosaveFailure::DeadlineExceeded);
        }
        final_state.reap_diagnostic_job();

        let mut budget = DEFAULT_AUTOSAVE_COMPLETION_BUDGET;
        let mut active_drained = true;
        if let Some(active) = service.active.as_mut() {
            if !final_state.remaining.is_empty()
                && final_state.project_root.as_deref() != Some(active.project_root.as_path())
            {
                final_state.fail(FinalAutosaveFailure::ProjectChanged);
            }
            let now = instant.saturating_duration_since(active.project_started_at);
            let completion = active.adapter.pump_completed_with_budget(now, budget);
            budget = budget.saturating_sub(completion.inspected_tickets());
            let is_final = final_state.final_batch_submitted;
            final_state.collect(&active.project_root, &completion, is_final);
            active_drained = active.adapter.is_drained();
            if active_drained {
                final_state.final_batch_submitted = false;
            }
        }
        let mut retired_drained = true;
        let mut fallback_budget = DEFAULT_AUTOSAVE_COMPLETION_BUDGET;
        for retired in &mut service.retired {
            let now = instant.saturating_duration_since(retired.project_started_at);
            let completion = retired.adapter.pump_completed_with_budget(now, budget);
            budget = budget.saturating_sub(completion.inspected_tickets());
            final_state.collect(&retired.project_root, &completion, false);
            final_state.collect_retired_fallback(retired, &mut fallback_budget);
            retired_drained &= retired.adapter.is_drained();
        }

        enum Submission {
            Diagnostics(Vec<FinalDiagnosticItem>),
            Final {
                active: ActiveAutosaveProject,
                requests: BTreeMap<AutosaveDocumentId, AutosaveDocumentRequest>,
            },
        }
        let submission = if !final_state.diagnostic_failed
            && final_state.diagnostic_job.is_none()
            && !final_state.diagnostics.is_empty()
            && instant < final_state.deadline
        {
            Some(Submission::Diagnostics(
                final_state
                    .diagnostics
                    .iter()
                    .take(DEFAULT_AUTOSAVE_COMPLETION_BUDGET)
                    .cloned()
                    .collect(),
            ))
        } else if final_state.failure.is_none()
            && active_drained
            && retired_drained
            && final_state.diagnostic_job.is_none()
            && final_state.diagnostics.is_empty()
            && !final_state.remaining.is_empty()
        {
            match service.active.take() {
                Some(active) => Some(Submission::Final {
                    active,
                    requests: final_state.remaining.clone(),
                }),
                None => {
                    final_state.fail(FinalAutosaveFailure::ProjectUnavailable);
                    None
                }
            }
        } else {
            None
        };
        let Some(submission) = submission else {
            let fallback_drained = service
                .retired
                .iter()
                .all(|retired| retired.fallback_diagnostics.is_empty());
            final_state.update_complete(active_drained, retired_drained, fallback_drained);
            final_state.report.unfinished_jobs =
                self.jobs.shutdown_progress().unfinished_jobs().to_vec();
            return;
        };

        // A synchronous progress observer may query or poll this service.
        // The flag keeps the temporarily owned adapter and not-yet-installed
        // ticket visible as pending while both locks are released.
        final_state.poll_in_flight = true;
        final_state.complete = false;
        drop(final_state);
        drop(service);
        match submission {
            Submission::Diagnostics(items) => {
                let submitted_count = items.len();
                let submitted = catch_unwind(AssertUnwindSafe(|| {
                    self.jobs.submit(
                        EditorJobSpec::new("Persist final autosave diagnostics", JobCategory::Misc),
                        FinalDiagnosticJob { items },
                    )
                }));
                let _service = self.lock_state();
                let mut final_state = drain.lock();
                match submitted {
                    Ok(Ok(ticket)) => {
                        final_state.diagnostic_job = Some((ticket, submitted_count));
                    }
                    Ok(Err(error)) => {
                        final_state.diagnostic_failed = true;
                        final_state.fail(FinalAutosaveFailure::AdmissionRejected {
                            message: error.to_string(),
                        });
                    }
                    Err(payload) => {
                        final_state.diagnostic_failed = true;
                        final_state.fail(FinalAutosaveFailure::SubmissionPanicked {
                            message: panic_message(payload),
                        });
                    }
                }
                final_state.poll_in_flight = false;
            }
            Submission::Final {
                mut active,
                requests,
            } => {
                let documents = requests
                    .keys()
                    .cloned()
                    .map(|document| AutosaveDocumentState::from_dirty_projection(document, true))
                    .collect::<Vec<_>>();
                let now = instant.saturating_duration_since(active.project_started_at);
                // The adapter remains owned outside the unwind closure.
                let submitted = catch_unwind(AssertUnwindSafe(|| {
                    active.adapter.schedule_final(
                        now,
                        &documents,
                        |document| {
                            requests
                                .get(document)
                                .map_or(1, AutosaveDocumentRequest::estimated_pending_bytes)
                        },
                        |document| requests.get(document).cloned(),
                    )
                }));
                let mut service = self.lock_state();
                let mut final_state = drain.lock();
                if service.active.is_none() {
                    service.active = Some(active);
                } else {
                    service.retired.push_back(RetiredAutosaveProject {
                        project_root: active.project_root,
                        project_started_at: active.project_started_at,
                        adapter: active.adapter,
                        fallback_diagnostics: VecDeque::new(),
                    });
                    final_state.fail(FinalAutosaveFailure::ProjectChanged);
                }
                match submitted {
                    Ok(Ok(true)) => final_state.final_batch_submitted = true,
                    Ok(Ok(false)) => {}
                    Ok(Err(error)) => final_state.fail(FinalAutosaveFailure::AdmissionRejected {
                        message: error.to_string(),
                    }),
                    Err(payload) => final_state.fail(FinalAutosaveFailure::SubmissionPanicked {
                        message: panic_message(payload),
                    }),
                }
                final_state.poll_in_flight = false;
            }
        }
        // A later tick consumes real terminal tickets; submission itself never
        // makes the drain complete, even if a worker finished very quickly.
    }

    /// Removes adapters only after their real tasks and the shared jobs owner are quiescent.
    pub(crate) fn release_drained_shutdown_adapters(&self) -> bool {
        if self.jobs.shutdown_progress().state() != EditorJobShutdownState::Quiescent {
            return false;
        }
        let mut service = self.lock_state();
        if !service
            .final_shutdown
            .as_ref()
            .is_some_and(FinalAutosaveDrain::succeeded)
            || service
                .active
                .as_ref()
                .is_some_and(|active| !active.adapter.is_drained())
            || service.retired.iter().any(|retired| !retired.is_finished())
        {
            return false;
        }
        service.active = None;
        service.retired.clear();
        true
    }
}

fn panic_message(payload: Box<dyn std::any::Any + Send>) -> String {
    if let Some(message) = payload.downcast_ref::<&str>() {
        (*message).to_owned()
    } else if let Some(message) = payload.downcast_ref::<String>() {
        message.clone()
    } else {
        "final autosave submission panicked".to_owned()
    }
}

#[cfg(test)]
pub(super) fn final_autosave_test_root(label: &str) -> PathBuf {
    let output = PathBuf::from(
        std::env::var_os("CARGO_TARGET_DIR")
            .expect("final autosave tests require the registered physical Cargo output"),
    );
    let physical = output
        .canonicalize()
        .expect("registered Cargo output must exist");
    assert!(
        ["D:/cargo-targets", "E:/cargo-targets", "F:/cargo-targets"]
            .iter()
            .filter_map(|base| Path::new(base).canonicalize().ok())
            .any(|base| physical.starts_with(base)),
        "test output must physically stay under a drive-root cargo-targets"
    );
    physical.join("zircon-test-output").join(format!(
        "editor09-final-{label}-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("fixture clock")
            .as_nanos(),
    ))
}

#[cfg(test)]
mod editor09_owned_final_tests {
    use super::*;
    use crate::core::jobs::{
        test_job_scheduler, test_job_system, EditorJobLimits, EditorJobProgressObserver,
        EditorJobProgressSource, EditorJobSystem, JobId, MutexGroup,
    };
    use crate::core::recovery::{
        AutosaveExtension, AutosaveJobAdapter, AutosaveJobPolicy, AutosavePolicy,
        AutosaveScheduler, AutosaveSnapshot, AutosaveSnapshotProvenance, AutosaveSnapshotSource,
        AutosaveSourceDigest, AutosaveSourcePath, AutosaveStore,
    };
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
    use std::sync::{mpsc, Weak};
    use std::time::Duration;

    fn wait_until(mut ready: impl FnMut() -> bool) {
        let deadline = Instant::now() + Duration::from_secs(5);
        while !ready() {
            assert!(
                Instant::now() < deadline,
                "owned final autosave did not become terminal"
            );
            std::thread::yield_now();
        }
    }

    fn retired(jobs: &EditorJobSystem, root: &Path, names: &[&str]) -> RetiredAutosaveProject {
        RetiredAutosaveProject {
            project_root: root.to_path_buf(),
            project_started_at: Instant::now(),
            adapter: AutosaveJobAdapter::new(
                jobs.clone(),
                AutosaveStore::new(root),
                AutosaveScheduler::new(AutosavePolicy::new(Duration::from_secs(10)).unwrap()),
            ),
            fallback_diagnostics: names
                .iter()
                .map(|name| {
                    AutosaveDocumentOutcome::shutdown_unavailable(
                        AutosaveDocumentId::parse(name).unwrap(),
                        AutosaveSourcePath::parse("scenes/main.zscene").unwrap(),
                    )
                })
                .collect(),
        }
    }

    struct GatedDiagnosticStore {
        items: Vec<FinalDiagnosticItem>,
        started: mpsc::Sender<()>,
        release: mpsc::Receiver<()>,
        fail: bool,
    }

    impl EditorJob for GatedDiagnosticStore {
        type Output = Vec<FinalDiagnosticResult>;

        fn run(self, _: JobContext) -> Result<Self::Output, JobError> {
            self.started.send(()).unwrap();
            self.release.recv_timeout(Duration::from_secs(5)).unwrap();
            if self.fail {
                return Err(JobError::Cancelled);
            }
            // A memory-store fixture acknowledges only the submitted records.
            Ok(self
                .items
                .into_iter()
                .map(|item| {
                    let mut outcome = item.outcome;
                    outcome.mark_diagnostic_persisted();
                    FinalDiagnosticResult {
                        report_index: item.report_index,
                        outcome,
                        issues: Vec::new(),
                    }
                })
                .collect())
        }
    }

    #[test]
    fn diagnostic_real_ticket_acknowledges_only_its_submitted_retired_prefix() {
        for fail_first in [false, true] {
            let root = final_autosave_test_root(if fail_first {
                "prefix-failure"
            } else {
                "prefix"
            });
            let jobs = test_job_system();
            let service = EditorAutosaveService::new(
                jobs.clone(),
                AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
            );
            let mut drain =
                service.begin_final_autosave(None, None, Instant::now() + Duration::from_secs(5));
            let mut first = retired(&jobs, &root, &["prefix_alpha", "prefix_bravo"]);
            let items = {
                let mut state = drain.lock();
                let mut budget = DEFAULT_AUTOSAVE_COMPLETION_BUDGET;
                state.collect_retired_fallback(&mut first, &mut budget);
                state.diagnostics.iter().cloned().collect::<Vec<_>>()
            };
            let submitted_count = items.len();
            assert_eq!(submitted_count, 2);
            assert!(submitted_count < DEFAULT_AUTOSAVE_COMPLETION_BUDGET);
            let (started_tx, started_rx) = mpsc::channel();
            let (release_tx, release_rx) = mpsc::channel();
            let ticket = jobs
                .submit(
                    EditorJobSpec::new("gated memory diagnostic store", JobCategory::Misc),
                    GatedDiagnosticStore {
                        items,
                        started: started_tx,
                        release: release_rx,
                        fail: fail_first,
                    },
                )
                .unwrap();
            drain.lock().diagnostic_job = Some((ticket, submitted_count));
            started_rx.recv_timeout(Duration::from_secs(5)).unwrap();

            // Use the production retired-owner ingestion while the first real
            // worker remains blocked. These records were not submitted to it.
            service.lock_state().retired.push_back(retired(
                &jobs,
                &root,
                &["late_alpha", "late_bravo", "late_charlie"],
            ));
            service.poll_final_autosave(&mut drain);
            assert_eq!(drain.lock().diagnostics.len(), 5);
            assert!(!drain.succeeded());
            release_tx.send(()).unwrap();
            wait_until(|| {
                drain
                    .lock()
                    .diagnostic_job
                    .as_ref()
                    .is_some_and(|(ticket, _)| ticket.is_terminal())
            });
            service.poll_final_autosave(&mut drain);

            if fail_first {
                assert_eq!(drain.lock().diagnostics.len(), 5);
                assert!(drain.lock().diagnostic_job.is_none());
                assert!(matches!(
                    drain.failure(),
                    Some(FinalAutosaveFailure::DiagnosticJobFailed { .. })
                ));
                assert!(!drain.succeeded());
                service.poll_final_autosave(&mut drain);
                assert_eq!(drain.lock().diagnostics.len(), 5);
            } else {
                {
                    let state = drain.lock();
                    assert_eq!(state.diagnostics.len(), 3);
                    assert!(state.report.outcomes[..2]
                        .iter()
                        .all(AutosaveDocumentOutcome::diagnostic_persisted));
                    assert!(state.report.outcomes[2..]
                        .iter()
                        .all(|outcome| !outcome.diagnostic_persisted()));
                    assert_eq!(
                        state.diagnostic_job.as_ref().map(|(_, count)| *count),
                        Some(3)
                    );
                }
                assert!(!drain.succeeded());
                wait_until(|| {
                    service.poll_final_autosave(&mut drain);
                    drain.is_complete()
                });
                assert!(drain.succeeded());
                assert!(drain.lock().diagnostics.is_empty());
                assert!(drain
                    .report()
                    .outcomes()
                    .iter()
                    .all(AutosaveDocumentOutcome::diagnostic_persisted));
            }
            if root.exists() {
                std::fs::remove_dir_all(root).unwrap();
            }
        }
    }

    struct ReenteringObserver {
        service: Mutex<Weak<EditorAutosaveService>>,
        admissions: AtomicUsize,
        unlocked: AtomicBool,
    }

    impl EditorJobProgressObserver for ReenteringObserver {
        fn job_admitted(&self, _: JobId, _: &EditorJobProgressSource) {
            let Some(service) = self.service.lock().unwrap().upgrade() else {
                return;
            };
            let owned = match service.state.try_lock() {
                Ok(state) => state.final_shutdown.clone(),
                Err(_) => {
                    self.unlocked.store(false, Ordering::Release);
                    return;
                }
            };
            let Some(mut drain) = owned else {
                return;
            };
            if drain.inner.try_lock().is_err() {
                self.unlocked.store(false, Ordering::Release);
                return;
            }
            self.admissions.fetch_add(1, Ordering::AcqRel);
            service.poll_final_autosave(&mut drain);
            let same = service.begin_final_autosave(None, None, Instant::now());
            if same.is_complete() || service.release_drained_shutdown_adapters() {
                self.unlocked.store(false, Ordering::Release);
            }
        }
        fn job_finished(&self, _: JobId, _: &EditorJobProgressSource) {}
        fn jobs_resynchronized(&self, _: &EditorJobProgressSource) {}
    }

    struct SnapshotSource;

    impl AutosaveSnapshotSource for SnapshotSource {
        fn source_path(&self) -> AutosaveSourcePath {
            AutosaveSourcePath::parse("scenes/main.zscene").unwrap()
        }
        fn capture(&self, _: &AutosaveDocumentId) -> Result<AutosaveSnapshot, JobError> {
            Ok(AutosaveSnapshot::new(
                1,
                AutosaveExtension::parse("zscene").unwrap(),
                self.source_path(),
                AutosaveSnapshotProvenance::capture(0, AutosaveSourceDigest::missing()),
                b"observer reentry fixture".to_vec(),
            ))
        }
    }

    #[test]
    fn final_submission_observer_can_reenter_without_locks_or_early_completion() {
        let root = final_autosave_test_root("observer");
        let observer = Arc::new(ReenteringObserver {
            service: Mutex::new(Weak::new()),
            admissions: AtomicUsize::new(0),
            unlocked: AtomicBool::new(true),
        });
        let jobs = EditorJobSystem::with_scheduler_and_bus_and_progress_observer(
            test_job_scheduler(),
            Default::default(),
            EditorJobLimits::default(),
            observer.clone(),
        );
        let service = Arc::new(EditorAutosaveService::new(
            jobs,
            AutosavePolicy::new(Duration::from_secs(10)).unwrap(),
        ));
        *observer.service.lock().unwrap() = Arc::downgrade(&service);
        service.poll_project(Some(&root));
        let request = AutosaveDocumentRequest::new(
            AutosaveDocumentId::parse("observer_final").unwrap(),
            AutosaveJobPolicy::for_save_mutex(MutexGroup::parse("save_observer_final").unwrap()),
            Arc::new(SnapshotSource),
        );
        let mut drain = service.begin_final_autosave(
            Some(&root),
            Some(vec![request]),
            Instant::now() + Duration::from_secs(5),
        );
        service.poll_final_autosave(&mut drain);
        assert!(observer.admissions.load(Ordering::Acquire) >= 1);
        assert!(observer.unlocked.load(Ordering::Acquire));
        assert!(!drain.is_complete());
        wait_until(|| {
            service.poll_final_autosave(&mut drain);
            drain.is_complete()
        });
        assert!(drain.succeeded());
        assert!(observer.unlocked.load(Ordering::Acquire));
        std::fs::remove_dir_all(root).unwrap();
    }
}
