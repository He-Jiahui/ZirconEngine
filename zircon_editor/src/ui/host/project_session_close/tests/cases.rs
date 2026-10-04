use std::path::PathBuf;

use zircon_runtime_interface::project::{
    ProjectActivationOperationIdGenerator, ProjectLaunchInstanceId,
};

use super::{
    ProjectCloseCoordinator, ProjectCloseCoordinatorPhase, ProjectCloseOperation,
    ProjectCloseTransitionError,
};

fn operation(root: &str) -> ProjectCloseOperation {
    let operation_id = ProjectActivationOperationIdGenerator::new(ProjectLaunchInstanceId::new())
        .allocate()
        .expect("fixture operation id");
    ProjectCloseOperation::new(PathBuf::from(root), operation_id)
}

#[test]
fn close_coordinator_has_one_forward_only_state_machine() {
    let operation = operation("project-a");
    let mut coordinator = ProjectCloseCoordinator::default();

    coordinator
        .begin_quiescing(operation.clone())
        .expect("quiescing");
    coordinator
        .begin_committing(&operation)
        .expect("committing");
    coordinator.finish_closed(&operation).expect("closed");

    assert_eq!(coordinator.phase(), ProjectCloseCoordinatorPhase::Closed);
    assert!(coordinator.begin_quiescing(operation).is_err());
}

#[test]
fn close_coordinator_rejects_a_different_operation() {
    let first = operation("project-a");
    let second = operation("project-a");
    let mut coordinator = ProjectCloseCoordinator::default();
    coordinator.begin_quiescing(first).expect("first operation");

    assert!(matches!(
        coordinator.begin_committing(&second),
        Err(ProjectCloseTransitionError::OperationMismatch { .. })
    ));
}

#[test]
fn recovery_required_is_terminal_for_in_process_close_retries() {
    let operation = operation("project-a");
    let mut coordinator = ProjectCloseCoordinator::default();
    coordinator
        .begin_quiescing(operation.clone())
        .expect("quiescing");
    coordinator
        .require_recovery(&operation)
        .expect("recovery state");

    assert_eq!(
        coordinator.phase(),
        ProjectCloseCoordinatorPhase::RecoveryRequired
    );
    assert!(coordinator.begin_committing(&operation).is_err());
}
