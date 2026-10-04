use std::fs;
use std::time::SystemTime;

use zircon_runtime_interface::project::session_lock::ProjectSessionPrincipalV1;
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use super::{
    RestoreDocumentExecutionError, RestoreExecutionOutcome, RestoreExecutionRetryability,
    RestoreExecutor,
};
use crate::core::recovery::{
    AutosaveDocumentId, RestoreAction, RestoreCandidate, RestoreFlow, RestoreFreshness,
    RestoreResolution, SessionAdmissionRequest, SessionGuard, SessionGuardAdmission,
};

#[test]
fn restore_materializes_a_copy_without_changing_the_authoritative_source() {
    let root = temporary_root("restore-copy");
    let source_path = root.join("assets").join("scene.zscene");
    let autosave_path = root
        .join(".zircon")
        .join("autosave")
        .join("scene_main")
        .join("1.zscene");
    fs::create_dir_all(source_path.parent().unwrap()).unwrap();
    fs::create_dir_all(autosave_path.parent().unwrap()).unwrap();
    fs::write(&source_path, "authoritative source").unwrap();
    fs::write(&autosave_path, "recovered autosave").unwrap();

    let lock = residual_lock(&root);
    let document = AutosaveDocumentId::parse("scene_main").unwrap();
    let candidate = RestoreCandidate::new(
        document.clone(),
        source_path.clone(),
        autosave_path,
        RestoreFreshness::SnapshotAheadOfSource,
    );
    let startup = RestoreFlow::detect(lock, [candidate]).unwrap();
    let plan = RestoreFlow::plan(
        &startup,
        [RestoreResolution::new(
            document,
            RestoreAction::RestoreAutosave,
        )],
    )
    .unwrap();

    let report = RestoreExecutor::new(&root)
        .execute(&startup, &plan)
        .unwrap();

    assert_eq!(
        fs::read_to_string(&source_path).unwrap(),
        "authoritative source"
    );
    let RestoreExecutionOutcome::RecoveredCopy(copy) = report.records()[0]
        .outcome()
        .expect("restore should succeed")
    else {
        panic!("expected recovered copy outcome");
    };
    assert_eq!(
        fs::read_to_string(copy.recovered_path()).unwrap(),
        "recovered autosave"
    );
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn discard_removes_only_the_selected_document_recovery_directory() {
    let root = temporary_root("restore-discard");
    let document = AutosaveDocumentId::parse("scene_main").unwrap();
    let autosave_path = root
        .join(".zircon")
        .join("autosave")
        .join(document.as_str())
        .join("1.zscene");
    fs::create_dir_all(autosave_path.parent().unwrap()).unwrap();
    fs::write(&autosave_path, "discarded autosave").unwrap();

    let lock = residual_lock(&root);
    let candidate = RestoreCandidate::new(
        document.clone(),
        root.join("assets").join("scene.zscene"),
        autosave_path.clone(),
        RestoreFreshness::SourceMissing,
    );
    let startup = RestoreFlow::detect(lock, [candidate]).unwrap();
    let plan = RestoreFlow::plan(
        &startup,
        [RestoreResolution::new(
            document,
            RestoreAction::DiscardAutosave,
        )],
    )
    .unwrap();

    let report = RestoreExecutor::new(&root)
        .execute(&startup, &plan)
        .unwrap();

    assert!(matches!(
        report.records()[0].outcome(),
        Some(RestoreExecutionOutcome::Discarded { .. })
    ));
    assert!(!autosave_path.parent().unwrap().exists());
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn execution_reports_every_document_after_retryable_and_permanent_failures() {
    let root = temporary_root("restore-partial-report");
    let retry_document = AutosaveDocumentId::parse("a_retry").unwrap();
    let rejected_document = AutosaveDocumentId::parse("b_rejected").unwrap();
    let restored_document = AutosaveDocumentId::parse("c_restored").unwrap();
    let retry_path = root
        .join(".zircon")
        .join("autosave")
        .join(retry_document.as_str())
        .join("missing.zscene");
    let rejected_path = root.join("outside-autosave").join("snapshot.zscene");
    let restored_path = root
        .join(".zircon")
        .join("autosave")
        .join(restored_document.as_str())
        .join("1.zscene");
    fs::create_dir_all(restored_path.parent().unwrap()).unwrap();
    fs::write(&restored_path, "recover me").unwrap();

    let startup = RestoreFlow::detect(
        residual_lock(&root),
        [
            RestoreCandidate::new(
                retry_document.clone(),
                root.join("assets/a_retry.zscene"),
                retry_path,
                RestoreFreshness::SnapshotAheadOfSource,
            ),
            RestoreCandidate::new(
                rejected_document.clone(),
                root.join("assets/b_rejected.zscene"),
                rejected_path,
                RestoreFreshness::SnapshotAheadOfSource,
            ),
            RestoreCandidate::new(
                restored_document.clone(),
                root.join("assets/c_restored.zscene"),
                restored_path,
                RestoreFreshness::SnapshotAheadOfSource,
            ),
        ],
    )
    .unwrap();
    let plan = RestoreFlow::plan(
        &startup,
        [
            RestoreResolution::new(retry_document.clone(), RestoreAction::RestoreAutosave),
            RestoreResolution::new(rejected_document.clone(), RestoreAction::RestoreAutosave),
            RestoreResolution::new(restored_document.clone(), RestoreAction::RestoreAutosave),
        ],
    )
    .unwrap();

    let report = RestoreExecutor::new(&root)
        .execute(&startup, &plan)
        .unwrap();

    assert_eq!(report.records().len(), 3);
    assert_eq!(report.success_count(), 1);
    assert_eq!(report.failure_count(), 2);
    assert!(report.has_failures());
    assert_eq!(report.records()[0].document(), &retry_document);
    assert!(matches!(
        report.records()[0].failure(),
        Some(RestoreDocumentExecutionError::Io { .. })
    ));
    assert_eq!(
        report.records()[0].retryability(),
        Some(RestoreExecutionRetryability::Retryable)
    );
    assert_eq!(report.records()[1].document(), &rejected_document);
    assert!(matches!(
        report.records()[1].failure(),
        Some(RestoreDocumentExecutionError::InvalidCandidatePath { .. })
    ));
    assert_eq!(
        report.records()[1].retryability(),
        Some(RestoreExecutionRetryability::RequiresOperatorIntervention)
    );
    assert_eq!(report.records()[2].document(), &restored_document);
    assert!(report.records()[2].outcome().is_some());

    let retryable = report.retryable_resolutions();
    assert_eq!(retryable.len(), 1);
    assert_eq!(retryable[0].document(), &retry_document);
    assert_eq!(retryable[0].action(), RestoreAction::RestoreAutosave);
    let retry_plan = RestoreFlow::retry_plan(&plan, retryable)
        .unwrap()
        .expect("one retryable failure should produce a retry plan");
    assert_eq!(retry_plan.resolutions().len(), 1);
    assert_eq!(retry_plan.resolutions()[0].document(), &retry_document);
    assert!(matches!(
        RestoreFlow::retry_plan(
            &plan,
            [RestoreResolution::new(
                retry_document,
                RestoreAction::DiscardAutosave,
            )],
        ),
        Err(crate::core::recovery::RestoreFlowError::ChangedRetryAction { .. })
    ));

    fs::remove_dir_all(root).unwrap();
}

fn temporary_root(label: &str) -> std::path::PathBuf {
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

fn residual_lock(project_root: &std::path::Path) -> crate::core::recovery::SessionLockInspection {
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
    let guard = match SessionGuard::claim(project_root, &admission).expect("fixture session claim")
    {
        SessionGuardAdmission::Acquired(guard) => guard,
        SessionGuardAdmission::Active { .. } | SessionGuardAdmission::Residual(_) => {
            panic!("fresh fixture root must acquire a session guard")
        }
    };
    let inspection = SessionGuard::inspect(project_root).expect("inspect residual fixture lock");
    drop(guard);
    inspection
}
