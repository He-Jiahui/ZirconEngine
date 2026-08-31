use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use zircon_runtime_interface::project::session_lock::ProjectSessionPrincipalV1;
use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};
use zircon_runtime_interface::runtime_build_set::ZrRuntimeBuildSetId;

use super::ProjectRecoveryDecisionCoordinator;
use crate::core::notifications::{
    DecisionCenterConfig, DecisionNotification, DecisionNotificationCenter, DecisionOption,
    DecisionOptionId, NotificationId, NotificationSource,
};
use crate::core::recovery::{
    AutosaveDocumentId, RestoreAction, RestoreCandidate, RestoreFlow, RestoreFreshness,
    RestoreStartup, SessionAdmissionRequest, SessionGuard, SessionGuardAdmission,
};

#[test]
fn recovery_candidates_publish_one_at_a_time_and_emit_work_only_after_every_resolution() {
    let root = temporary_root("recovery-decision-sequence");
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::default()).unwrap();
    let coordinator = ProjectRecoveryDecisionCoordinator::default();
    coordinator
        .begin(
            &center,
            &root,
            recovery_startup(&root, &["scene_main", "scene_ui"]),
        )
        .unwrap();

    assert!(coordinator.pump(&center).unwrap().is_none());
    let first = center.pending_snapshot();
    assert_eq!(first.len(), 1);
    assert_eq!(
        first[0].notification().display_subject(),
        Some("assets/scene_main.zscene")
    );
    center
        .resolve(
            first[0].ticket(),
            &DecisionOptionId::parse("restore").unwrap(),
        )
        .unwrap();

    assert!(coordinator.pump(&center).unwrap().is_none());
    let second = center.pending_snapshot();
    assert_eq!(second.len(), 1);
    assert_eq!(
        second[0].notification().display_subject(),
        Some("assets/scene_ui.zscene")
    );
    center
        .resolve(
            second[0].ticket(),
            &DecisionOptionId::parse("compare").unwrap(),
        )
        .unwrap();

    let work = coordinator
        .pump(&center)
        .unwrap()
        .expect("all explicit choices should create one recovery work item");
    assert_eq!(work.plan().resolutions().len(), 2);
    assert_eq!(
        work.plan().resolutions()[0].action(),
        RestoreAction::RestoreAutosave
    );
    assert_eq!(
        work.plan().resolutions()[1].action(),
        RestoreAction::OpenComparison
    );
    assert!(!coordinator.is_active());

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn expired_receipt_history_republishes_the_same_candidate_without_guessing_a_choice() {
    let root = temporary_root("recovery-decision-expired-receipt");
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::new(4, 1).unwrap()).unwrap();
    let coordinator = ProjectRecoveryDecisionCoordinator::default();
    coordinator
        .begin(&center, &root, recovery_startup(&root, &["scene_main"]))
        .unwrap();
    coordinator.pump(&center).unwrap();
    let first_ticket = center.pending_snapshot().pop().unwrap().ticket().clone();
    center
        .resolve(&first_ticket, &DecisionOptionId::parse("discard").unwrap())
        .unwrap();

    resolve_foreign_decision(&center, "editor.recovery.foreign_one");
    resolve_foreign_decision(&center, "editor.recovery.foreign_two");

    assert!(coordinator.pump(&center).unwrap().is_none());
    let republished = center.pending_snapshot();
    assert_eq!(republished.len(), 1);
    assert_ne!(republished[0].ticket(), &first_ticket);
    assert_eq!(
        republished[0].notification().display_subject(),
        Some("assets/scene_main.zscene")
    );

    fs::remove_dir_all(root).unwrap();
}

#[test]
fn full_shared_decision_center_defers_recovery_publication_until_capacity_changes() {
    let root = temporary_root("recovery-decision-capacity");
    let center = DecisionNotificationCenter::new(DecisionCenterConfig::new(1, 4).unwrap()).unwrap();
    let coordinator = ProjectRecoveryDecisionCoordinator::default();
    let foreign_ticket = publish_foreign_decision(&center, "editor.recovery.foreign_blocker");
    coordinator
        .begin(&center, &root, recovery_startup(&root, &["scene_main"]))
        .unwrap();

    coordinator.pump(&center).unwrap();
    coordinator.pump(&center).unwrap();
    assert_eq!(center.pending_count(), 1);
    assert_eq!(
        center.pending_snapshot()[0].notification().id().as_str(),
        "editor.recovery.foreign_blocker"
    );

    center
        .resolve(&foreign_ticket, &DecisionOptionId::parse("apply").unwrap())
        .unwrap();
    coordinator.pump(&center).unwrap();
    assert_eq!(center.pending_count(), 1);
    assert_eq!(
        center.pending_snapshot()[0]
            .notification()
            .display_subject(),
        Some("assets/scene_main.zscene")
    );

    fs::remove_dir_all(root).unwrap();
}

fn recovery_startup(root: &Path, documents: &[&str]) -> RestoreStartup {
    let candidates = documents.iter().map(|document| {
        RestoreCandidate::new(
            AutosaveDocumentId::parse(document).unwrap(),
            root.join("assets").join(format!("{document}.zscene")),
            root.join(".zircon")
                .join("autosave")
                .join(document)
                .join("1.zscene"),
            RestoreFreshness::SnapshotAheadOfSource,
        )
    });
    RestoreFlow::detect(residual_lock(root), candidates).unwrap()
}

fn resolve_foreign_decision(center: &DecisionNotificationCenter, id: &str) {
    let ticket = publish_foreign_decision(center, id);
    center
        .resolve(&ticket, &DecisionOptionId::parse("apply").unwrap())
        .unwrap();
}

fn publish_foreign_decision(
    center: &DecisionNotificationCenter,
    id: &str,
) -> crate::core::notifications::DecisionTicket {
    center
        .publish(
            DecisionNotification::new(
                NotificationId::parse(id).unwrap(),
                NotificationSource::builtin("editor.test").unwrap(),
                "editor.play.pending_edits.title",
                "editor.play.pending_edits.message",
                vec![
                    DecisionOption::new(
                        DecisionOptionId::parse("apply").unwrap(),
                        "editor.play.pending_edits.apply",
                    )
                    .unwrap(),
                    DecisionOption::new(
                        DecisionOptionId::parse("discard").unwrap(),
                        "editor.play.pending_edits.discard",
                    )
                    .unwrap(),
                ],
            )
            .unwrap(),
        )
        .unwrap()
}

fn residual_lock(root: &Path) -> crate::core::recovery::SessionLockInspection {
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
