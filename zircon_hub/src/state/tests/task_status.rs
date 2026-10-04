use super::*;

#[test]
fn task_status_operation_summary_names_scope_and_target() {
    let status = TaskStatus::success("Project selected", HubMessage::raw_text("Game"))
        .with_operation(TaskOperationKind::Project, "Game");

    assert_eq!(status.operation_summary(), "Project: Game");
}

#[test]
fn task_status_progress_tracks_lifecycle_checkpoints() {
    assert_eq!(
        TaskStatus::idle().progress_percent,
        TASK_PROGRESS_IDLE_PERCENT
    );
    assert_eq!(
        TaskStatus::running(
            "Building",
            HubMessage::raw_text("Running tools/zircon_build.py")
        )
        .progress_percent,
        TASK_PROGRESS_STARTED_PERCENT
    );
    assert_eq!(
        TaskStatus::success("Build complete", HubMessage::raw_text("out")).progress_percent,
        TASK_PROGRESS_COMPLETE_PERCENT
    );
    assert_eq!(
        TaskStatus::error(
            "Build failed",
            HubMessage::raw_text("failed"),
            HubMessage::raw_text("retry"),
        )
        .progress_percent,
        TASK_PROGRESS_IDLE_PERCENT
    );

    let clamped = TaskStatus::running(
        "Building",
        HubMessage::raw_text("Running tools/zircon_build.py"),
    )
    .with_progress_percent(TASK_PROGRESS_COMPLETE_PERCENT + 1);

    assert_eq!(clamped.progress_percent, TASK_PROGRESS_COMPLETE_PERCENT);
}
