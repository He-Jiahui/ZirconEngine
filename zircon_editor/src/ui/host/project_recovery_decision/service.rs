use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use thiserror::Error;
use zircon_runtime_interface::project::session_lock::ProjectSessionAdmissionLifecycleV1;

use crate::core::jobs::{EditorJobSystem, JobError, JobId, JobSubmitError, JobTicket};
use crate::core::notifications::DecisionNotificationCenter;
use crate::core::recovery::{
    AutosaveDocumentId, RestoreExecutionReport, RestoreExecutionRetryability, RestoreFlow,
    RestoreFlowError, RestoreResolution, RestoreStartup, SessionGuard, SessionGuardError,
};

use super::coordinator::{
    ProjectRecoveryDecisionCoordinator, ProjectRecoveryDecisionError, RecoveryFailureChoice,
    RecoveryFailureChoiceReceipt, RecoveryFailureReview,
};
use super::execution::RecoveryRestoreJob;
use super::model::RecoveryRestoreWork;

/// This owned stamp identifies a flow. It cannot authorize an action without a fresh admission.
#[derive(Clone, Debug, PartialEq, Eq)]
struct RecoverySessionStamp {
    project_root: PathBuf,
    instance_id: String,
    generation: u64,
}

/// Created before entering the service gate, while the manager holds the actual session mutex.
/// The borrow keeps the verified OS-lease owner alive throughout action admission.
pub(super) struct RecoveryOwnerAdmission<'guard> {
    _guard: &'guard SessionGuard,
    stamp: RecoverySessionStamp,
}

impl<'guard> RecoveryOwnerAdmission<'guard> {
    pub(super) fn from_guard(
        guard: &'guard SessionGuard,
    ) -> Result<Self, ProjectRecoveryDecisionServiceError> {
        guard.validate_ownership()?;
        let record = guard.record();
        if record.lifecycle() != ProjectSessionAdmissionLifecycleV1::Ready {
            return Err(ProjectRecoveryDecisionServiceError::SessionNotReady);
        }
        let generation = record.session_generation()
            .ok_or(ProjectRecoveryDecisionServiceError::SessionNotReady)?.get();
        Ok(Self {
            _guard: guard,
            stamp: RecoverySessionStamp {
                project_root: guard.project_root().to_path_buf(),
                instance_id: record.instance_id().to_string(),
                generation,
            },
        })
    }
}

/// Manager-owned recovery lifecycle; every new worker and operator settlement requires a fresh
/// borrowed Ready owner, a matching flow stamp, and an exact terminal-attempt decision ticket.
pub(crate) struct ProjectRecoveryDecisionService {
    operation_gate: Mutex<()>,
    coordinator: ProjectRecoveryDecisionCoordinator,
    execution: Mutex<RecoveryExecutionState>,
}

impl Default for ProjectRecoveryDecisionService {
    fn default() -> Self {
        Self {
            operation_gate: Mutex::new(()),
            coordinator: ProjectRecoveryDecisionCoordinator::default(),
            execution: Mutex::new(RecoveryExecutionState::default()),
        }
    }
}

#[derive(Default)]
struct RecoveryExecutionState {
    owner: Option<RecoverySessionStamp>,
    pending: Option<QueuedRecoveryWork>,
    in_flight: Option<InFlightRecoveryWork>,
    failed: Option<FailedRecoveryExecution>,
    failed_attempt: Option<JobId>,
    failed_acknowledged: bool,
    review: RecoveryFailureReview,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RecoveryExecutionAttempt {
    Initial,
    Retry,
}

struct QueuedRecoveryWork {
    work: Arc<RecoveryRestoreWork>,
    attempt: RecoveryExecutionAttempt,
}

struct InFlightRecoveryWork {
    work: Arc<RecoveryRestoreWork>,
    attempt: RecoveryExecutionAttempt,
    ticket: JobTicket<RestoreExecutionReport>,
}

struct FailedRecoveryExecution {
    original_work: Arc<RecoveryRestoreWork>,
    documents: BTreeMap<AutosaveDocumentId, FailedRecoveryDocument>,
}

struct FailedRecoveryDocument {
    resolution: RestoreResolution,
    retryability: RestoreExecutionRetryability,
    detail: String,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(super) struct RecoveryUnresolvedSummary {
    pub(super) total: usize,
    pub(super) retryable: usize,
    pub(super) operator_required: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RecoveryCompletionDisposition {
    Completed,
    Incomplete,
    EvidenceKept,
}

pub(crate) struct RecoveryExecutionCompletion {
    job: JobId,
    result: Option<Result<RestoreExecutionReport, JobError>>,
    unresolved: RecoveryUnresolvedSummary,
    disposition: RecoveryCompletionDisposition,
}

impl RecoveryExecutionCompletion {
    pub(super) const fn job(&self) -> JobId { self.job }

    pub(super) fn result(&self) -> Option<&Result<RestoreExecutionReport, JobError>> {
        self.result.as_ref()
    }

    pub(super) const fn unresolved(&self) -> RecoveryUnresolvedSummary { self.unresolved }

    pub(super) const fn disposition(&self) -> RecoveryCompletionDisposition { self.disposition }
}

impl FailedRecoveryExecution {
    fn unresolved_summary(&self) -> RecoveryUnresolvedSummary {
        let retryable = self.documents.values().filter(|document| {
            document.retryability == RestoreExecutionRetryability::Retryable
        }).count();
        RecoveryUnresolvedSummary {
            total: self.documents.len(),
            retryable,
            operator_required: self.documents.len().saturating_sub(retryable),
        }
    }
}

impl ProjectRecoveryDecisionService {
    pub(super) fn begin(
        &self,
        center: &DecisionNotificationCenter,
        project_root: &Path,
        startup: RestoreStartup,
        admission: &RecoveryOwnerAdmission<'_>,
    ) -> Result<bool, ProjectRecoveryDecisionServiceError> {
        let _operation = self.lock_operation_gate();
        if admission.stamp.project_root != project_root {
            return Err(ProjectRecoveryDecisionServiceError::SessionChanged);
        }
        if self.execution_is_active() || self.coordinator.is_active() {
            return Err(ProjectRecoveryDecisionServiceError::ExecutionAlreadyActive);
        }
        let began = self.coordinator.begin(center, project_root, startup)?;
        *self.lock_execution() = RecoveryExecutionState {
            owner: Some(admission.stamp.clone()),
            ..RecoveryExecutionState::default()
        };
        Ok(began)
    }

    /// Completion collection cannot start work or settle a flow. It remains available when a
    /// session degrades, so an already running worker's terminal evidence is never abandoned.
    pub(super) fn poll_completion(
        &self,
    ) -> Result<Option<RecoveryExecutionCompletion>, ProjectRecoveryDecisionServiceError> {
        let _operation = self.lock_operation_gate();
        self.poll_execution()
    }

    pub(super) fn pump(
        &self,
        center: &DecisionNotificationCenter,
        jobs: &EditorJobSystem,
        admission: &RecoveryOwnerAdmission<'_>,
    ) -> Result<Option<RecoveryExecutionCompletion>, ProjectRecoveryDecisionServiceError> {
        let _operation = self.lock_operation_gate();
        {
            let execution = self.lock_execution();
            Self::require_owner(&execution, admission)?;
        }
        if let Some(completion) = self.poll_execution()? {
            return Ok(Some(completion));
        }
        let mut execution = self.lock_execution();
        Self::require_owner(&execution, admission)?;
        if execution.in_flight.is_some() {
            return Ok(None);
        }
        if execution.pending.is_some() {
            drop(execution);
            self.submit_pending_work(jobs)?;
            return Ok(None);
        }
        if execution.failed.is_some() && !execution.failed_acknowledged {
            let attempt = execution.failed_attempt
                .ok_or(ProjectRecoveryDecisionServiceError::MissingFailedExecution)?;
            let summary = execution.failed.as_ref()
                .ok_or(ProjectRecoveryDecisionServiceError::NoFailedExecution)?
                .unresolved_summary();
            let receipt = self.coordinator.pump_failure_review(
                &mut execution.review, center, attempt, summary.total, summary.retryable,
            )?;
            if let Some(receipt) = receipt {
                let completion = Self::apply_failure_choice_locked(
                    &mut execution, admission, &receipt,
                )?;
                let submit = execution.pending.is_some();
                drop(execution);
                if submit {
                    self.submit_pending_work(jobs)?;
                }
                return Ok(completion);
            }
            return Ok(None);
        }
        drop(execution);
        if let Some(work) = self.coordinator.pump(center)? {
            if work.project_root() != admission.stamp.project_root {
                return Err(ProjectRecoveryDecisionServiceError::SessionChanged);
            }
            self.lock_execution().pending = Some(QueuedRecoveryWork {
                work: Arc::new(work),
                attempt: RecoveryExecutionAttempt::Initial,
            });
            self.submit_pending_work(jobs)?;
        }
        Ok(None)
    }

    pub(super) fn is_active(&self) -> bool {
        let _operation = self.lock_operation_gate();
        self.coordinator.is_active() || self.execution_is_active()
    }

    /// Called only by the real Close preflight. Merely reading status never reopens a prompt.
    pub(super) fn request_review_if_active(&self) -> bool {
        let _operation = self.lock_operation_gate();
        let mut execution = self.lock_execution();
        if execution.failed.is_some() && !execution.failed_acknowledged
            && execution.pending.is_none() && execution.in_flight.is_none()
        {
            execution.review.request_review();
        }
        drop(execution);
        self.coordinator.is_active() || self.execution_is_active()
    }

    fn require_owner(
        execution: &RecoveryExecutionState,
        admission: &RecoveryOwnerAdmission<'_>,
    ) -> Result<(), ProjectRecoveryDecisionServiceError> {
        if execution.owner.as_ref() != Some(&admission.stamp) {
            return Err(ProjectRecoveryDecisionServiceError::SessionChanged);
        }
        Ok(())
    }

    /// The caller holds operation_gate. No method below reacquires it.
    fn apply_failure_choice_locked(
        execution: &mut RecoveryExecutionState,
        admission: &RecoveryOwnerAdmission<'_>,
        receipt: &RecoveryFailureChoiceReceipt,
    ) -> Result<Option<RecoveryExecutionCompletion>, ProjectRecoveryDecisionServiceError> {
        Self::require_owner(execution, admission)?;
        if execution.pending.is_some() || execution.in_flight.is_some() {
            return Err(ProjectRecoveryDecisionServiceError::ExecutionAlreadyActive);
        }
        if execution.failed_acknowledged
            || execution.failed_attempt != Some(receipt.attempt)
            || !execution.review.authorizes(receipt)
        {
            return Err(ProjectRecoveryDecisionServiceError::StaleFailureDecision);
        }
        let summary = execution.failed.as_ref()
            .ok_or(ProjectRecoveryDecisionServiceError::NoFailedExecution)?
            .unresolved_summary();
        match receipt.choice {
            RecoveryFailureChoice::Retry => {
                Self::retry_failed(execution)?;
                execution.review.finish(receipt.choice);
                Ok(None)
            }
            RecoveryFailureChoice::Defer => {
                execution.review.finish(receipt.choice);
                Ok(None)
            }
            RecoveryFailureChoice::KeepFiles => {
                // Preserve failed details and every physical recovery file; this is an explicit
                // operator settlement of this attempt, not a successful restore report.
                execution.failed_acknowledged = true;
                execution.review.finish(receipt.choice);
                Ok(Some(RecoveryExecutionCompletion {
                    job: receipt.attempt,
                    result: None,
                    unresolved: summary,
                    disposition: RecoveryCompletionDisposition::EvidenceKept,
                }))
            }
        }
    }

    fn retry_failed(
        execution: &mut RecoveryExecutionState,
    ) -> Result<(), ProjectRecoveryDecisionServiceError> {
        let failed = execution.failed.as_ref()
            .ok_or(ProjectRecoveryDecisionServiceError::NoFailedExecution)?;
        let work = failed.retry_work()?
            .ok_or(ProjectRecoveryDecisionServiceError::NoRetryableDocuments)?;
        execution.pending = Some(QueuedRecoveryWork {
            work: Arc::new(work),
            attempt: RecoveryExecutionAttempt::Retry,
        });
        Ok(())
    }

    fn poll_execution(
        &self,
    ) -> Result<Option<RecoveryExecutionCompletion>, ProjectRecoveryDecisionServiceError> {
        let mut execution = self.lock_execution();
        let Some(in_flight) = execution.in_flight.take() else {
            return Ok(None);
        };
        let Some(result) = in_flight.ticket.try_take() else {
            execution.in_flight = Some(in_flight);
            return Ok(None);
        };
        let job = in_flight.ticket.id();
        match in_flight.attempt {
            RecoveryExecutionAttempt::Initial => {
                execution.failed = FailedRecoveryExecution::from_initial_result(
                    in_flight.work, &result,
                )?;
            }
            RecoveryExecutionAttempt::Retry => {
                let failed = execution.failed.as_mut()
                    .ok_or(ProjectRecoveryDecisionServiceError::MissingFailedExecution)?;
                failed.apply_retry_result(&in_flight.work, &result);
                if failed.documents.is_empty() {
                    execution.failed = None;
                }
            }
        }
        execution.failed_attempt = execution.failed.as_ref().map(|_| job);
        execution.failed_acknowledged = false;
        execution.review = RecoveryFailureReview::default();
        let summary = execution.failed.as_ref()
            .map(FailedRecoveryExecution::unresolved_summary).unwrap_or_default();
        let disposition = if summary.total == 0 && result.is_ok() {
            RecoveryCompletionDisposition::Completed
        } else {
            RecoveryCompletionDisposition::Incomplete
        };
        Ok(Some(RecoveryExecutionCompletion {
            job,
            result: Some(result),
            unresolved: summary,
            disposition,
        }))
    }

    fn submit_pending_work(
        &self,
        jobs: &EditorJobSystem,
    ) -> Result<(), ProjectRecoveryDecisionServiceError> {
        let queued = self.lock_execution().pending.take()
            .ok_or(ProjectRecoveryDecisionServiceError::MissingPendingWork)?;
        let spec = RecoveryRestoreJob::spec(queued.work.as_ref());
        match jobs.submit(spec, RecoveryRestoreJob::new(Arc::clone(&queued.work))) {
            Ok(ticket) => {
                self.lock_execution().in_flight = Some(InFlightRecoveryWork {
                    work: queued.work, attempt: queued.attempt, ticket,
                });
            }
            Err(error) => {
                self.lock_execution().pending = Some(queued);
                return Err(error.into());
            }
        }
        Ok(())
    }

    fn execution_is_active(&self) -> bool {
        let execution = self.lock_execution();
        execution.pending.is_some() || execution.in_flight.is_some()
            || (execution.failed.is_some() && !execution.failed_acknowledged)
    }

    fn lock_operation_gate(&self) -> std::sync::MutexGuard<'_, ()> {
        self.operation_gate.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_execution(&self) -> std::sync::MutexGuard<'_, RecoveryExecutionState> {
        self.execution.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}


impl FailedRecoveryExecution {
    fn from_initial_result(
        original_work: Arc<RecoveryRestoreWork>,
        result: &Result<RestoreExecutionReport, JobError>,
    ) -> Result<Option<Self>, RestoreFlowError> {
        let documents = match result {
            Ok(report) => failed_documents_from_report(report),
            Err(error) => unknown_failed_documents(original_work.plan().resolutions(), error),
        };
        if documents.is_empty() {
            return Ok(None);
        }
        Ok(Some(Self {
            original_work,
            documents,
        }))
    }

    fn retry_work(&self) -> Result<Option<RecoveryRestoreWork>, RestoreFlowError> {
        let retry_plan = RestoreFlow::retry_plan(
            self.original_work.plan(),
            self.documents
                .values()
                .filter(|document| document.retryability == RestoreExecutionRetryability::Retryable)
                .map(|document| document.resolution.clone()),
        )?;
        Ok(retry_plan.map(|plan| {
            RecoveryRestoreWork::new(
                self.original_work.project_root().to_path_buf(),
                self.original_work.startup().clone(),
                plan,
            )
        }))
    }

    fn apply_retry_result(
        &mut self,
        retry_work: &RecoveryRestoreWork,
        result: &Result<RestoreExecutionReport, JobError>,
    ) {
        match result {
            Ok(report) => {
                for record in report.records() {
                    match record.failure() {
                        Some(failure) => {
                            self.documents.insert(
                                record.document().clone(),
                                FailedRecoveryDocument {
                                    resolution: record.resolution().clone(),
                                    retryability: failure.retryability(),
                                    detail: failure.to_string(),
                                },
                            );
                        }
                        None => {
                            self.documents.remove(record.document());
                        }
                    }
                }
            }
            Err(error) => {
                // A job-level failure has no per-document commit boundary. Repeating those
                // resolutions could duplicate a copy that was published before the worker died.
                for (document, failure) in
                    unknown_failed_documents(retry_work.plan().resolutions(), error)
                {
                    self.documents.insert(document, failure);
                }
            }
        }
    }
}

fn failed_documents_from_report(
    report: &RestoreExecutionReport,
) -> BTreeMap<AutosaveDocumentId, FailedRecoveryDocument> {
    report
        .records()
        .iter()
        .filter_map(|record| {
            record.failure().map(|failure| {
                (
                    record.document().clone(),
                    FailedRecoveryDocument {
                        resolution: record.resolution().clone(),
                        retryability: failure.retryability(),
                        detail: failure.to_string(),
                    },
                )
            })
        })
        .collect()
}

fn unknown_failed_documents(
    resolutions: &[RestoreResolution],
    error: &JobError,
) -> BTreeMap<AutosaveDocumentId, FailedRecoveryDocument> {
    let detail = format!("recovery worker ended without a per-document terminal report: {error}");
    resolutions
        .iter()
        .map(|resolution| {
            (
                resolution.document().clone(),
                FailedRecoveryDocument {
                    resolution: resolution.clone(),
                    retryability: RestoreExecutionRetryability::RequiresOperatorIntervention,
                    detail: detail.clone(),
                },
            )
        })
        .collect()
}

#[derive(Debug, Error)]
pub(super) enum ProjectRecoveryDecisionServiceError {
    #[error(transparent)]
    SessionAuthority(#[from] SessionGuardError),
    #[error("recovery actions require the current owned Ready project session")]
    SessionNotReady,
    #[error("recovery work belongs to a different physical project or Ready session")]
    SessionChanged,
    #[error("recovery action belongs to a stale terminal attempt or decision ticket")]
    StaleFailureDecision,
    #[error(transparent)]
    Coordinator(#[from] ProjectRecoveryDecisionError),
    #[error(transparent)]
    JobSubmit(#[from] JobSubmitError),
    #[error(transparent)]
    RestoreFlow(#[from] RestoreFlowError),
    #[error("project recovery execution is already active")]
    ExecutionAlreadyActive,
    #[error("project recovery has no pending validated work to submit")]
    MissingPendingWork,
    #[error("project recovery retry completed without a retained failed execution")]
    MissingFailedExecution,
    #[error("project recovery has no retained failed execution")]
    NoFailedExecution,
    #[error("project recovery has no failed document that is safe to retry")]
    NoRetryableDocuments,
}

#[cfg(test)]
#[path = "tests/service.rs"]
mod tests;
