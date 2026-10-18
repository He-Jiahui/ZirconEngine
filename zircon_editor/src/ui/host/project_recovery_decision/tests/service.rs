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
    // The native validation owner must preregister this physical scratch tree and bind TEMP/TMP.
    // Read-only canonicalization rejects unprepared roots before any fixture is created.
    let approved = std::fs::canonicalize("E:/cargo-targets/zircon-local")
        .expect("native owner must prepare the approved physical artifact root");
    let physical = approved
        .to_str()
        .expect("approved physical fixture root must be Unicode")
        .replace('\\', "/");
    let physical = physical.strip_prefix("//?/").unwrap_or(&physical);
    assert!(
        physical.eq_ignore_ascii_case("E:/cargo-targets/zircon-local"),
        "approved fixture root must physically resolve below the E drive cargo-targets root"
    );
    let scratch = std::fs::canonicalize(std::env::temp_dir())
        .expect("native owner must prepare its registered TEMP/TMP directory");
    assert!(
        scratch.starts_with(&approved),
        "recovery fixtures require registered TEMP/TMP below E:/cargo-targets/zircon-local"
    );
    scratch.join(format!(
        "zircon-editor-{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

use super::{
    ProjectRecoveryDecisionService, ProjectRecoveryDecisionServiceError,
    RecoveryCompletionDisposition, RecoveryExecutionCompletion, RecoveryOwnerAdmission,
};
use crate::core::jobs::{test_job_system, EditorJobSystem};
use crate::core::notifications::{
    DecisionCenterConfig, DecisionNotification, DecisionNotificationCenter,
    DecisionNotificationError, DecisionOption, DecisionOptionId, DecisionTicket, NotificationId,
    NotificationSource,
};
use crate::ui::host::project_recovery_decision::coordinator::{
    RecoveryFailureChoice, RecoveryFailureChoiceReceipt,
};
use std::time::{Duration, Instant};

struct RecoveryDecisionFixture {
    root: PathBuf,
    guard: Option<SessionGuard>,
    service: ProjectRecoveryDecisionService,
    center: DecisionNotificationCenter,
    jobs: EditorJobSystem,
    retry_document: AutosaveDocumentId,
    operator_document: AutosaveDocumentId,
    retry_snapshot: PathBuf,
}

impl RecoveryDecisionFixture {
    fn new(label: &str) -> Self {
        Self::with_center(
            label,
            DecisionNotificationCenter::new(Default::default()).unwrap(),
        )
    }

    fn with_center(label: &str, center: DecisionNotificationCenter) -> Self {
        let requested_root = temporary_root(label);
        let (guard, inspection) = ready_recovery_test_guard(&requested_root);
        let root = guard.project_root().to_path_buf();
        fs::create_dir_all(root.join("assets")).unwrap();
        fs::write(
            root.join("assets/retry_scene.zscene"),
            b"authoritative retry",
        )
        .unwrap();
        fs::write(
            root.join("assets/operator_scene.zscene"),
            b"authoritative operator",
        )
        .unwrap();
        fs::write(root.join("outside.zscene"), b"operator recovery evidence").unwrap();
        let retry_document = AutosaveDocumentId::parse("retry_scene").unwrap();
        let operator_document = AutosaveDocumentId::parse("operator_scene").unwrap();
        let retry_snapshot = root.join(".zircon/autosave/retry_scene/1.zscene");
        let startup = RestoreFlow::detect(
            inspection,
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
        let service = ProjectRecoveryDecisionService::default();
        let admission = RecoveryOwnerAdmission::from_guard(&guard).unwrap();
        service.begin(&center, &root, startup, &admission).unwrap();
        Self {
            root,
            guard: Some(guard),
            service,
            center,
            jobs: test_job_system(),
            retry_document,
            operator_document,
            retry_snapshot,
        }
    }

    fn admission(&self) -> RecoveryOwnerAdmission<'_> {
        RecoveryOwnerAdmission::from_guard(self.guard.as_ref().unwrap()).unwrap()
    }

    fn pump(
        &self,
    ) -> Result<Option<RecoveryExecutionCompletion>, ProjectRecoveryDecisionServiceError> {
        self.service
            .pump(&self.center, &self.jobs, &self.admission())
    }

    fn run_initial_failure(&self) -> RecoveryExecutionCompletion {
        assert!(self.pump().unwrap().is_none());
        for _ in 0..2 {
            let pending = self.center.pending_snapshot();
            assert_eq!(pending.len(), 1);
            let ticket = pending[0].ticket().clone();
            self.resolve(&ticket, "restore");
            assert!(self.pump().unwrap().is_none());
        }
        let terminal = self.await_terminal();
        assert_eq!(
            terminal.result().unwrap().as_ref().unwrap().failure_count(),
            2
        );
        assert_eq!(terminal.unresolved().total, 2);
        assert!(self.service.is_active());
        terminal
    }

    fn await_terminal(&self) -> RecoveryExecutionCompletion {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if let Some(terminal) = self.service.poll_completion().unwrap() {
                return terminal;
            }
            assert!(
                Instant::now() < deadline,
                "real recovery worker must reach a terminal"
            );
            std::thread::yield_now();
        }
    }

    fn failure_ticket(&self) -> DecisionTicket {
        assert!(self.pump().unwrap().is_none());
        let pending = self.center.pending_snapshot();
        assert_eq!(pending.len(), 1);
        pending[0].ticket().clone()
    }

    fn resolve(&self, ticket: &DecisionTicket, option: &str) {
        self.try_resolve(ticket, option).unwrap();
    }

    fn try_resolve(
        &self,
        ticket: &DecisionTicket,
        option: &str,
    ) -> Result<(), DecisionNotificationError> {
        self.center
            .resolve(ticket, &DecisionOptionId::parse(option).unwrap())
            .map(|_| ())
    }

    fn cancel(&self, ticket: &DecisionTicket) {
        self.center.cancel(ticket).unwrap();
    }

    fn write_retry_snapshot(&self) {
        fs::create_dir_all(self.retry_snapshot.parent().unwrap()).unwrap();
        fs::write(&self.retry_snapshot, b"retry recovery evidence").unwrap();
    }

    fn evidence_bytes(&self) -> Vec<(PathBuf, Vec<u8>)> {
        [
            "assets/retry_scene.zscene",
            "assets/operator_scene.zscene",
            "outside.zscene",
            ".zircon/autosave/retry_scene/1.zscene",
        ]
        .iter()
        .map(|path| {
            let path = self.root.join(path);
            let bytes = fs::read(&path).unwrap();
            (path, bytes)
        })
        .collect()
    }
}

impl Drop for RecoveryDecisionFixture {
    fn drop(&mut self) {
        let running = { self.service.lock_execution().in_flight.take() };
        if let Some(running) = running {
            // Cleanup consumes only a remaining real ticket; no result is used as a fake report.
            let _ = running.ticket.wait();
        }
        // Shutdown happens after terminal collection, never as a successful worker barrier.
        let unfinished = self.jobs.shutdown(Instant::now() + Duration::from_secs(10));
        if !std::thread::panicking() {
            assert!(
                unfinished.is_empty(),
                "fixture must not retain its own recovery job"
            );
        }
        drop(self.guard.take());
        let _ = fs::remove_dir_all(&self.root);
    }
}

fn test_unrelated_decision(id: &str) -> DecisionNotification {
    DecisionNotification::new(
        NotificationId::parse(id).unwrap(),
        NotificationSource::builtin("editor.recovery.test").unwrap(),
        "editor.recovery.review.title",
        "editor.recovery.review.message",
        vec![
            DecisionOption::new(
                DecisionOptionId::parse("keep_files").unwrap(),
                "editor.recovery.review.keep_files",
            )
            .unwrap(),
            DecisionOption::new(
                DecisionOptionId::parse("defer").unwrap(),
                "editor.recovery.review.defer",
            )
            .unwrap(),
        ],
    )
    .unwrap()
}

fn ready_recovery_test_guard(root: &Path) -> (SessionGuard, SessionLockInspection) {
    let operation = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("fixture operation");
    let request = SessionAdmissionRequest::new(
        operation,
        ProjectSessionPrincipalV1::Welcome,
        ZrRuntimeBuildSetId::parse(
            "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .unwrap(),
    );
    let mut guard = match SessionGuard::claim(root, &request).unwrap() {
        SessionGuardAdmission::Acquired(guard) => guard,
        _ => panic!("the test must acquire the actual physical owner"),
    };
    let inspection = SessionGuard::inspect(root).unwrap();
    guard.mark_preflight_approved().unwrap();
    guard.begin_activation().unwrap();
    guard.commit_ready().unwrap();
    (guard, inspection)
}

#[test]
fn retry_decision_runs_only_safe_documents_and_preserves_unresolved_operator_failures() {
    let fixture = RecoveryDecisionFixture::new("recovery-retry-through-worker");
    let first = fixture.run_initial_failure();
    assert_eq!(
        first.disposition(),
        RecoveryCompletionDisposition::Incomplete
    );
    assert_eq!(first.unresolved().total, 2);
    assert_eq!(first.unresolved().retryable, 1);
    fixture.write_retry_snapshot();
    let ticket = fixture.failure_ticket();
    fixture.resolve(&ticket, "retry");
    fixture.pump().unwrap();
    let retried = fixture.await_terminal();
    let report = retried.result().unwrap().as_ref().unwrap();
    assert_eq!(report.records().len(), 1);
    assert_eq!(report.records()[0].document(), &fixture.retry_document);
    assert_eq!(report.success_count(), 1);
    assert!(!report.has_failures());
    // This worker succeeded, but the flow still contains the earlier permanent failure.
    assert_eq!(
        retried.disposition(),
        RecoveryCompletionDisposition::Incomplete
    );
    assert_eq!(retried.unresolved().total, 1);
    assert_eq!(retried.unresolved().retryable, 0);
    assert_eq!(retried.unresolved().operator_required, 1);
    assert!(fixture.service.is_active());
    let failed = fixture.service.lock_execution();
    let failed = failed.failed.as_ref().unwrap();
    assert!(!failed.documents.contains_key(&fixture.retry_document));
    assert!(failed.documents.contains_key(&fixture.operator_document));
}

#[test]
fn keep_files_decision_settles_only_the_attempt_without_deleting_recovery_evidence() {
    let fixture = RecoveryDecisionFixture::new("recovery-keep-files");
    fixture.run_initial_failure();
    fixture.write_retry_snapshot();
    let evidence = fixture.evidence_bytes();
    let ticket = fixture.failure_ticket();
    fixture.resolve(&ticket, "keep_files");
    let kept = fixture
        .pump()
        .unwrap()
        .expect("explicit settlement completion");
    assert_eq!(
        kept.disposition(),
        RecoveryCompletionDisposition::EvidenceKept
    );
    assert!(kept.result().is_none());
    assert_eq!(kept.unresolved().total, 2);
    assert_eq!(fixture.evidence_bytes(), evidence);
    assert!(!fixture.service.is_active());
    assert!(!fixture.service.request_review_if_active());
    assert!(fixture.center.pending_snapshot().is_empty());
    assert_eq!(
        fixture
            .service
            .lock_execution()
            .failed
            .as_ref()
            .unwrap()
            .documents
            .len(),
        2
    );
    assert!(fixture.pump().unwrap().is_none());
    assert_eq!(fixture.evidence_bytes(), evidence);
}

#[test]
fn deferred_failure_is_republished_only_after_a_close_review_request() {
    let fixture = RecoveryDecisionFixture::new("recovery-deferred-close-review");
    fixture.run_initial_failure();
    let old = fixture.failure_ticket();
    fixture.cancel(&old);
    fixture.pump().unwrap();
    for _ in 0..4 {
        assert!(fixture.pump().unwrap().is_none());
        assert!(fixture.center.pending_snapshot().is_empty());
        assert!(fixture.service.is_active());
    }
    // This is the production service entry used by the real Close preflight.
    assert!(fixture.service.request_review_if_active());
    let new = fixture.failure_ticket();
    assert_ne!(new, old);
    assert!(fixture.try_resolve(&old, "keep_files").is_err());
    fixture.resolve(&new, "keep_files");
    fixture
        .pump()
        .unwrap()
        .expect("the new ticket settles the same attempt");
    assert!(!fixture.service.is_active());
}

#[test]
fn duplicate_or_old_tickets_cannot_repeat_retry_or_acknowledge_a_new_attempt() {
    let fixture = RecoveryDecisionFixture::new("recovery-stale-decision-ticket");
    let initial = fixture.run_initial_failure();
    let old_attempt = initial.job();
    let old = fixture.failure_ticket();
    fixture.resolve(&old, "retry");
    fixture.pump().unwrap();
    let retried = fixture.await_terminal();
    assert_ne!(retried.job(), old_attempt);
    assert_eq!(retried.unresolved().total, 2);
    let current = fixture.failure_ticket();
    assert_ne!(old, current);
    // The center deliberately accepts the same option idempotently; it must not rerun work.
    assert!(fixture.try_resolve(&old, "retry").is_ok());
    assert!(fixture.pump().unwrap().is_none());
    {
        let state = fixture.service.lock_execution();
        assert!(state.pending.is_none() && state.in_flight.is_none());
        assert_eq!(state.failed_attempt, Some(retried.job()));
    }
    assert!(fixture.try_resolve(&old, "keep_files").is_err());
    let stale = RecoveryFailureChoiceReceipt {
        attempt: old_attempt,
        ticket: old,
        choice: RecoveryFailureChoice::KeepFiles,
    };
    let admission = fixture.admission();
    let operation = fixture.service.lock_operation_gate();
    let mut state = fixture.service.lock_execution();
    assert!(matches!(
        ProjectRecoveryDecisionService::apply_failure_choice_locked(&mut state, &admission, &stale,),
        Err(ProjectRecoveryDecisionServiceError::StaleFailureDecision)
    ));
    assert!(!state.failed_acknowledged);
    assert!(state.pending.is_none() && state.in_flight.is_none());
    assert_eq!(state.failed_attempt, Some(retried.job()));
    drop(state);
    drop(operation);
    assert!(fixture.service.is_active());
    fixture.resolve(&current, "keep_files");
    fixture.pump().unwrap();
    assert!(!fixture.service.is_active());
}

#[test]
fn lost_or_replaced_ready_ownership_rejects_failure_actions_without_consuming_evidence() {
    let mut fixture = RecoveryDecisionFixture::new("recovery-ready-owner-recheck");
    fixture.run_initial_failure();
    fixture.write_retry_snapshot();
    let evidence = fixture.evidence_bytes();
    let ticket = fixture.failure_ticket();
    fixture.resolve(&ticket, "keep_files");
    let guard = fixture.guard.as_ref().unwrap();
    let record_path = guard.path().to_path_buf();
    let record = fs::read(&record_path).unwrap();
    fs::write(&record_path, b"not the current session record").unwrap();
    assert!(matches!(
        RecoveryOwnerAdmission::from_guard(fixture.guard.as_ref().unwrap()),
        Err(ProjectRecoveryDecisionServiceError::SessionAuthority(_))
    ));
    assert!(fixture.service.is_active());
    assert_eq!(fixture.evidence_bytes(), evidence);
    fs::write(&record_path, &record).unwrap();
    // A different, genuinely owned Ready project cannot authorize this flow.
    let other = RecoveryDecisionFixture::new("recovery-unrelated-ready-owner");
    let admission = other.admission();
    assert!(matches!(
        fixture
            .service
            .pump(&fixture.center, &fixture.jobs, &admission),
        Err(ProjectRecoveryDecisionServiceError::SessionChanged)
    ));
    assert!(fixture.service.is_active());
    assert_eq!(fixture.evidence_bytes(), evidence);
    let old_stamp = fixture.admission().stamp.clone();
    // Normal release removes only this session record, not its recovery evidence.
    fixture.guard.take().unwrap().release().unwrap();
    let (new_guard, _) = ready_recovery_test_guard(&fixture.root);
    fixture.guard = Some(new_guard);
    let current = fixture.admission();
    assert_eq!(current.stamp.project_root, old_stamp.project_root);
    assert_ne!(current.stamp.generation, old_stamp.generation);
    assert!(matches!(
        fixture
            .service
            .pump(&fixture.center, &fixture.jobs, &current),
        Err(ProjectRecoveryDecisionServiceError::SessionChanged)
    ));
    assert!(fixture.service.is_active());
    assert!(!fixture.service.lock_execution().failed_acknowledged);
    assert_eq!(fixture.evidence_bytes(), evidence);
}

#[test]
fn failure_actions_are_rejected_while_retry_is_queued_or_running() {
    let fixture = RecoveryDecisionFixture::new("recovery-pending-failure-actions");
    let initial = fixture.run_initial_failure();
    fixture.write_retry_snapshot();
    let ticket = fixture.failure_ticket();
    fixture.resolve(&ticket, "retry");
    let admission = fixture.admission();
    let receipt = {
        let _operation = fixture.service.lock_operation_gate();
        let mut state = fixture.service.lock_execution();
        let summary = state.failed.as_ref().unwrap().unresolved_summary();
        let receipt = fixture
            .service
            .coordinator
            .pump_failure_review(
                &mut state.review,
                &fixture.center,
                initial.job(),
                summary.total,
                summary.retryable,
            )
            .unwrap()
            .expect("real center receipt");
        assert!(ProjectRecoveryDecisionService::apply_failure_choice_locked(
            &mut state, &admission, &receipt,
        )
        .unwrap()
        .is_none());
        assert!(state.pending.is_some() && state.in_flight.is_none());
        assert!(matches!(
            ProjectRecoveryDecisionService::apply_failure_choice_locked(
                &mut state, &admission, &receipt,
            ),
            Err(ProjectRecoveryDecisionServiceError::ExecutionAlreadyActive)
        ));
        receipt
    };
    {
        let _operation = fixture.service.lock_operation_gate();
        fixture.service.submit_pending_work(&fixture.jobs).unwrap();
        let mut state = fixture.service.lock_execution();
        assert!(state.pending.is_none() && state.in_flight.is_some());
        // In-flight means submitted but not yet collected; the worker may already be terminal.
        assert!(matches!(
            ProjectRecoveryDecisionService::apply_failure_choice_locked(
                &mut state, &admission, &receipt,
            ),
            Err(ProjectRecoveryDecisionServiceError::ExecutionAlreadyActive)
        ));
    }
    let retried = fixture.await_terminal();
    assert_eq!(retried.unresolved().operator_required, 1);
    assert!(fixture.service.is_active());
}

#[test]
fn expired_or_capacity_blocked_failure_review_never_releases_the_close_fence() {
    let fixture = RecoveryDecisionFixture::with_center(
        "recovery-review-receipt-window",
        DecisionNotificationCenter::new(DecisionCenterConfig::new(1, 1).unwrap()).unwrap(),
    );
    fixture.run_initial_failure();
    let blocker = fixture
        .center
        .publish(test_unrelated_decision("recovery.review.blocker"))
        .unwrap();
    assert!(fixture.pump().unwrap().is_none());
    assert!(fixture.service.is_active());
    assert_eq!(fixture.center.pending_snapshot().len(), 1);
    assert_eq!(fixture.center.pending_snapshot()[0].ticket(), &blocker);
    fixture.resolve(&blocker, "defer");
    let old = fixture.failure_ticket();
    fixture.resolve(&old, "keep_files");
    let eviction = fixture
        .center
        .publish(test_unrelated_decision("recovery.review.eviction"))
        .unwrap();
    fixture.resolve(&eviction, "defer");
    // The exact Keep receipt is now outside the retained window. Never guess its outcome.
    assert!(fixture.pump().unwrap().is_none());
    assert!(fixture.service.is_active());
    let new = fixture.failure_ticket();
    assert_ne!(new, old);
    assert!(!fixture.service.lock_execution().failed_acknowledged);
    fixture.resolve(&new, "keep_files");
    let kept = fixture.pump().unwrap().expect("fresh explicit receipt");
    assert_eq!(
        kept.disposition(),
        RecoveryCompletionDisposition::EvidenceKept
    );
    assert!(!fixture.service.is_active());
}
