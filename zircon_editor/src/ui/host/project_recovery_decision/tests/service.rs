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
