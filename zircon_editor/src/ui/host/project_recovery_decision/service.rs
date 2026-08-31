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
pub(super) struct ProjectRecoveryDecisionService {
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

pub(super) struct RecoveryExecutionCompletion {
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
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::time::SystemTime;

    use zircon_runtime_interface::project::session_lock::ProjectSessionPrincipalV1;
    use zircon_runtime_interface::project::{
        ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
    };
    use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

    use super::FailedRecoveryExecution;
    use crate::core::jobs::JobError;
    use crate::core::recovery::{
        AutosaveDocumentId, RestoreAction, RestoreCandidate, RestoreExecutionReport,
        RestoreExecutionRetryability, RestoreExecutor, RestoreFlow, RestoreFreshness,
        RestoreResolution, SessionAdmissionRequest, SessionGuard, SessionGuardAdmission,
        SessionLockInspection,
    };
    use crate::ui::host::project_recovery_decision::model::RecoveryRestoreWork;

    #[test]
    fn partial_failure_retries_only_safe_documents_and_retains_operator_work() {
        let root = temporary_root("recovery-service-partial-retry");
        let retry_document = AutosaveDocumentId::parse("retry_scene").unwrap();
        let operator_document = AutosaveDocumentId::parse("operator_scene").unwrap();
        let retry_snapshot = root
            .join(".zircon")
            .join("autosave")
            .join(retry_document.as_str())
            .join("1.zscene");
        let startup = RestoreFlow::detect(
            residual_lock(&root),
            [
                RestoreCandidate::new(
                    retry_document.clone(),
                    root.join("assets/retry_scene.zscene"),
                    retry_snapshot.clone(),
                    RestoreFreshness::SnapshotAheadOfSource,
                ),
                RestoreCandidate::new(
                    operator_document.clone(),
                    root.join("assets/operator_scene.zscene"),
                    root.join("outside.zscene"),
                    RestoreFreshness::SnapshotAheadOfSource,
                ),
            ],
        )
        .unwrap();
        let plan = RestoreFlow::plan(
            &startup,
            [
                RestoreResolution::new(retry_document.clone(), RestoreAction::RestoreAutosave),
                RestoreResolution::new(operator_document.clone(), RestoreAction::RestoreAutosave),
            ],
        )
        .unwrap();
        let original = Arc::new(RecoveryRestoreWork::new(root.clone(), startup, plan));
        let initial = RestoreExecutor::new(&root)
            .execute(original.startup(), original.plan())
            .unwrap();
        let mut failed =
            FailedRecoveryExecution::from_initial_result(Arc::clone(&original), &Ok(initial))
                .unwrap()
                .expect("both document failures must be retained");

        let retry = failed
            .retry_work()
            .unwrap()
            .expect("the I/O failure is explicitly retryable");
        assert_eq!(retry.plan().resolutions().len(), 1);
        assert_eq!(retry.plan().resolutions()[0].document(), &retry_document);
        assert_eq!(
            failed.documents[&operator_document].retryability,
            RestoreExecutionRetryability::RequiresOperatorIntervention
        );

        fs::create_dir_all(retry_snapshot.parent().unwrap()).unwrap();
        fs::write(&retry_snapshot, b"recover me").unwrap();
        let retry_result = RestoreExecutor::new(&root)
            .execute(retry.startup(), retry.plan())
            .unwrap();
        failed.apply_retry_result(&retry, &Ok(retry_result));

        assert!(!failed.documents.contains_key(&retry_document));
        assert!(failed.documents.contains_key(&operator_document));
        assert!(failed.retry_work().unwrap().is_none());
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn job_failure_without_document_terminals_requires_operator_intervention() {
        let root = temporary_root("recovery-service-job-failure");
        let document = AutosaveDocumentId::parse("scene_main").unwrap();
        let startup = RestoreFlow::detect(
            residual_lock(&root),
            [RestoreCandidate::new(
                document.clone(),
                root.join("assets/scene_main.zscene"),
                root.join(".zircon/autosave/scene_main/1.zscene"),
                RestoreFreshness::SnapshotAheadOfSource,
            )],
        )
        .unwrap();
        let plan = RestoreFlow::plan(
            &startup,
            [RestoreResolution::new(
                document.clone(),
                RestoreAction::RestoreAutosave,
            )],
        )
        .unwrap();
        let work = Arc::new(RecoveryRestoreWork::new(root.clone(), startup, plan));
        let result: Result<RestoreExecutionReport, JobError> = Err(JobError::ResultChannelClosed);

        let failed = FailedRecoveryExecution::from_initial_result(work, &result)
            .unwrap()
            .expect("unknown worker completion must retain the recovery fence");

        assert_eq!(
            failed.documents[&document].retryability,
            RestoreExecutionRetryability::RequiresOperatorIntervention
        );
        assert!(failed.retry_work().unwrap().is_none());
        assert!(failed.documents[&document]
            .detail
            .contains("without a per-document terminal report"));
        fs::remove_dir_all(root).unwrap();
    }

    fn residual_lock(root: &Path) -> SessionLockInspection {
        let operation = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
            .allocate()
            .expect("fixture operation id");
        let admission = SessionAdmissionRequest::new(
            operation,
            ProjectSessionPrincipalV1::Welcome,
            ZrRuntimeBuildSetId::parse(
                "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            )
            .expect("fixture BuildSet"),
        );
        let guard = match SessionGuard::claim(root, &admission).expect("fixture session claim") {
            SessionGuardAdmission::Acquired(guard) => guard,
            SessionGuardAdmission::Active { .. } | SessionGuardAdmission::Residual(_) => {
                panic!("fresh fixture root must acquire a session guard")
            }
        };
        let inspection = SessionGuard::inspect(root).expect("inspect residual fixture lock");
        drop(guard);
        inspection
    }

    fn temporary_root(label: &str) -> PathBuf {
        std::env::current_dir()
            .unwrap()
            .join("target")
            .join(format!(
                "zircon-editor-{label}-{}-{}",
                std::process::id(),
                SystemTime::now()
                    .duration_since(SystemTime::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ))
    }
}
