use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use thiserror::Error;

use crate::core::jobs::{EditorJobSystem, JobError, JobId, JobSubmitError, JobTicket};
use crate::core::notifications::DecisionNotificationCenter;
use crate::core::recovery::{
    AutosaveDocumentId, RestoreExecutionReport, RestoreExecutionRetryability, RestoreFlow,
    RestoreFlowError, RestoreResolution, RestoreStartup,
};

use super::coordinator::{ProjectRecoveryDecisionCoordinator, ProjectRecoveryDecisionError};
use super::execution::RecoveryRestoreJob;
use super::model::RecoveryRestoreWork;

/// Manager-owned recovery lifecycle. It separates receipt collection from job admission so the
/// retained host never performs recovery filesystem work at frame cadence.
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
    pending: Option<QueuedRecoveryWork>,
    in_flight: Option<InFlightRecoveryWork>,
    failed: Option<FailedRecoveryExecution>,
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

pub(crate) struct RecoveryExecutionCompletion {
    job: JobId,
    result: Result<RestoreExecutionReport, JobError>,
}

impl RecoveryExecutionCompletion {
    pub(super) const fn job(&self) -> JobId {
        self.job
    }

    pub(super) fn result(&self) -> &Result<RestoreExecutionReport, JobError> {
        &self.result
    }
}

impl ProjectRecoveryDecisionService {
    pub(super) fn begin(
        &self,
        center: &DecisionNotificationCenter,
        project_root: &std::path::Path,
        startup: RestoreStartup,
    ) -> Result<bool, ProjectRecoveryDecisionServiceError> {
        let _operation = self.lock_operation_gate();
        if self.execution_is_active() {
            return Err(ProjectRecoveryDecisionServiceError::ExecutionAlreadyActive);
        }
        Ok(self.coordinator.begin(center, project_root, startup)?)
    }

    /// Collects receipts, admits a background worker only after a complete plan exists, and
    /// returns one terminal worker result for the manager's notification/log projection.
    pub(super) fn pump(
        &self,
        center: &DecisionNotificationCenter,
        jobs: &EditorJobSystem,
    ) -> Result<Option<RecoveryExecutionCompletion>, ProjectRecoveryDecisionServiceError> {
        let _operation = self.lock_operation_gate();
        if let Some(completion) = self.poll_execution()? {
            return Ok(Some(completion));
        }
        if self.execution_has_ticket() {
            return Ok(None);
        }

        if self.execution_has_pending_work() {
            self.submit_pending_work(jobs)?;
            return Ok(None);
        }
        if let Some(work) = self.coordinator.pump(center)? {
            self.store_pending_work(work);
            self.submit_pending_work(jobs)?;
        }
        Ok(None)
    }

    pub(super) fn is_active(&self) -> bool {
        let _operation = self.lock_operation_gate();
        self.coordinator.is_active() || self.execution_is_active()
    }

    /// Requeues only the documents whose latest terminal record is explicitly retryable.
    ///
    /// The failed audit remains active while the retry is pending or running, so project close
    /// cannot remove the residual session marker between attempts.
    pub(super) fn retry_failed(&self) -> Result<(), ProjectRecoveryDecisionServiceError> {
        let _operation = self.lock_operation_gate();
        let mut execution = self.lock_execution();
        if execution.pending.is_some() || execution.in_flight.is_some() {
            return Err(ProjectRecoveryDecisionServiceError::ExecutionAlreadyActive);
        }
        let retry_work = execution
            .failed
            .as_ref()
            .ok_or(ProjectRecoveryDecisionServiceError::NoFailedExecution)?
            .retry_work()?
            .ok_or(ProjectRecoveryDecisionServiceError::NoRetryableDocuments)?;
        execution.pending = Some(QueuedRecoveryWork {
            work: Arc::new(retry_work),
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
        let job = in_flight.ticket.id();
        match in_flight.ticket.try_take() {
            Some(result) => {
                match in_flight.attempt {
                    RecoveryExecutionAttempt::Initial => {
                        execution.failed = FailedRecoveryExecution::from_initial_result(
                            Arc::clone(&in_flight.work),
                            &result,
                        )?;
                    }
                    RecoveryExecutionAttempt::Retry => {
                        let failed = execution
                            .failed
                            .as_mut()
                            .ok_or(ProjectRecoveryDecisionServiceError::MissingFailedExecution)?;
                        failed.apply_retry_result(&in_flight.work, &result);
                        if failed.documents.is_empty() {
                            execution.failed = None;
                        }
                    }
                }
                Ok(Some(RecoveryExecutionCompletion { job, result }))
            }
            None => {
                execution.in_flight = Some(in_flight);
                Ok(None)
            }
        }
    }

    fn submit_pending_work(
        &self,
        jobs: &EditorJobSystem,
    ) -> Result<(), ProjectRecoveryDecisionServiceError> {
        let queued = self
            .lock_execution()
            .pending
            .take()
            .ok_or(ProjectRecoveryDecisionServiceError::MissingPendingWork)?;
        let spec = RecoveryRestoreJob::spec(queued.work.as_ref());
        match jobs.submit(spec, RecoveryRestoreJob::new(Arc::clone(&queued.work))) {
            Ok(ticket) => {
                self.lock_execution().in_flight = Some(InFlightRecoveryWork {
                    work: queued.work,
                    attempt: queued.attempt,
                    ticket,
                });
            }
            Err(error) => {
                self.lock_execution().pending = Some(queued);
                return Err(error.into());
            }
        }
        Ok(())
    }

    fn store_pending_work(&self, work: RecoveryRestoreWork) {
        self.lock_execution().pending = Some(QueuedRecoveryWork {
            work: Arc::new(work),
            attempt: RecoveryExecutionAttempt::Initial,
        });
    }

    fn execution_is_active(&self) -> bool {
        let execution = self.lock_execution();
        if let Some(failed) = execution.failed.as_ref() {
            debug_assert!(failed
                .documents
                .values()
                .all(|document| !document.detail.is_empty()));
            return true;
        }
        execution.pending.is_some() || execution.in_flight.is_some()
    }

    fn execution_has_pending_work(&self) -> bool {
        self.lock_execution().pending.is_some()
    }

    fn execution_has_ticket(&self) -> bool {
        self.lock_execution().in_flight.is_some()
    }

    fn lock_operation_gate(&self) -> std::sync::MutexGuard<'_, ()> {
        self.operation_gate
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn lock_execution(&self) -> std::sync::MutexGuard<'_, RecoveryExecutionState> {
        self.execution
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
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
