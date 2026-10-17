use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicBool, Ordering},
    sync::{Arc, Mutex},
};

use crate::{
    error::HubError,
    process::EditorChildReaper,
    projects::RecentProject,
    settings::HubConfig,
    state::{
        HubActionKind, HubActionStatus, HubMessage, HubMessageId, ShellMessageId,
        TaskExecutionOutcome, TaskOperationKind, TaskSeverity, TaskStatus,
        TASK_PROGRESS_PREPARED_PERCENT, TASK_PROGRESS_STARTED_PERCENT,
    },
};

use super::super::HubRuntimeSession;
use super::{
    execute_background_task, run_background_worker_loop, BackgroundTask, BackgroundTaskContext,
};
use crate::tauri_app::HubActionRequest;

#[test]
fn editor_shutdown_cancels_active_launch_and_removes_only_queued_editor_work() {
    let temp = temp_test_dir("zircon-hub-editor-shutdown-admission");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    let editor_request = HubActionRequest {
        action_id: "open-editor".to_string(),
        target_id: None,
        payload: None,
    };
    session
        .start_background_action_status(&editor_request)
        .unwrap();
    assert_eq!(session.editor_launch_owner().request_shutdown(), Some(1));
    session.background_action_queue.push_back(editor_request);
    session.background_action_queue.push_back(HubActionRequest {
        action_id: "package-project".to_string(),
        target_id: None,
        payload: None,
    });

    assert_eq!(session.request_editor_launch_shutdown(), Some(1));
    assert!(session.editor_launch_task_is_active(1));
    assert!(session
        .background_cancellation_for_task(1)
        .expect("active Editor cancellation token")
        .is_cancellation_requested());
    assert_eq!(session.background_action_queue.len(), 1);
    assert_eq!(
        session.background_action_queue[0].action_id,
        "package-project"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn editor_shutdown_leaves_unrelated_running_task_intact() {
    let temp = temp_test_dir("zircon-hub-editor-shutdown-other-task");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    session
        .start_background_action_status(&HubActionRequest {
            action_id: "build-project".to_string(),
            target_id: None,
            payload: None,
        })
        .unwrap();

    assert_eq!(session.request_editor_launch_shutdown(), None);
    assert!(!session
        .background_cancellation_for_task(1)
        .expect("active build cancellation token")
        .is_cancellation_requested());
    assert!(!session.editor_launch_task_is_active(1));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn queued_editor_is_refused_when_shutdown_closes_admission_at_build_handoff() {
    let temp = temp_test_dir("zircon-hub-editor-queued-shutdown-handoff");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.engines[0].source_dir = temp.join("missing-source");
    let build = HubActionRequest {
        action_id: "build-project".to_string(),
        target_id: None,
        payload: None,
    };
    session.start_background_action_status(&build).unwrap();
    session.background_worker_active = true;
    session.background_action_queue.push_back(HubActionRequest {
        action_id: "open-editor".to_string(),
        target_id: None,
        payload: None,
    });
    let shared = Arc::new(Mutex::new(session));
    let admission_closed = AtomicBool::new(false);
    let reaper = EditorChildReaper::start().expect("start fixture reaper");

    run_background_worker_loop(
        build,
        &shared,
        &|_| admission_closed.store(true, Ordering::Release),
        &reaper,
        &admission_closed,
    );

    let session = shared.lock().unwrap();
    let editor_records = session
        .config
        .action_history
        .iter()
        .filter(|record| record.action == HubActionKind::OpenEditor)
        .collect::<Vec<_>>();
    assert_eq!(editor_records.len(), 1);
    assert_eq!(editor_records[0].status, HubActionStatus::Cancelled);
    assert_eq!(editor_records[0].process_id, None);
    drop(session);
    reaper
        .shutdown_and_join(std::time::Duration::from_secs(1))
        .unwrap();
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_action_status_marks_build_running_without_executing_it() {
    let temp = temp_test_dir("zircon-hub-background-build-status");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);

    session
        .start_background_action_status(&HubActionRequest {
            action_id: "build-project".to_string(),
            target_id: None,
            payload: None,
        })
        .unwrap();

    assert!(session.task_status.running);
    assert_eq!(session.task_status.label, "Building");
    assert_eq!(
        session.task_status.operation,
        Some(TaskOperationKind::Build)
    );
    assert_eq!(
        session.task_status.progress_percent,
        TASK_PROGRESS_STARTED_PERCENT
    );
    assert_eq!(session.task_status.task_id, 1);
    assert_eq!(session.config.action_history.len(), 0);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_action_progress_advances_after_preparation() {
    let temp = temp_test_dir("zircon-hub-background-progress-prepare");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);

    session
        .start_background_action_status(&HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: None,
            payload: None,
        })
        .unwrap();
    session.mark_background_action_prepared();

    assert!(session.task_status.running);
    assert_eq!(
        session.task_status.progress_percent,
        TASK_PROGRESS_PREPARED_PERCENT
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_action_error_is_recoverable_visible_status() {
    let temp = temp_test_dir("zircon-hub-background-error-status");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);

    session
        .record_background_action_error(
            &HubActionRequest {
                action_id: "package-project".to_string(),
                target_id: None,
                payload: None,
            },
            HubError::message("worker failed"),
        )
        .unwrap();

    assert!(!session.task_status.running);
    assert_eq!(session.task_status.severity, TaskSeverity::Error);
    assert_eq!(session.task_status.label, "Packaging failed");
    assert_eq!(session.task_status.target.as_deref(), Some("Game"));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_action_status_uses_explicit_project_target_label() {
    let temp = temp_test_dir("zircon-hub-background-target-status");
    let selected = temp.join("Selected");
    let target = temp.join("Target");
    fs::create_dir_all(&selected).unwrap();
    fs::create_dir_all(&target).unwrap();
    let mut session = session_with_projects(
        &temp,
        &[("Selected", selected.clone()), ("Target", target.clone())],
        &selected,
    );
    let request = HubActionRequest {
        action_id: "package-project".to_string(),
        target_id: Some(target.to_string_lossy().into_owned()),
        payload: None,
    };

    session.apply_request_project_target(&request).unwrap();
    session.start_background_action_status(&request).unwrap();

    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(target.as_path())
    );
    assert_eq!(session.task_status.target.as_deref(), Some("Target"));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_action_start_records_invalid_project_target_without_spawning() {
    let temp = temp_test_dir("zircon-hub-background-invalid-target");
    let selected = temp.join("Selected");
    fs::create_dir_all(&selected).unwrap();
    let mut session = session_with_project(&temp, "Selected", &selected);
    let request = HubActionRequest {
        action_id: "package-project".to_string(),
        target_id: Some("missing-project".to_string()),
        payload: None,
    };

    let should_spawn = session
        .start_background_action_or_record_error(&request)
        .expect("invalid background action target should become visible Hub state");

    assert!(!should_spawn);
    assert!(!session.background_worker_active);
    assert!(session.background_action_queue.is_empty());
    assert!(!session.task_status.running);
    assert_eq!(session.task_status.severity, TaskSeverity::Error);
    assert_eq!(session.task_status.label, "Packaging failed");
    assert_eq!(
        session.task_status.detail,
        "Unknown recent project target for package-project: missing-project"
    );
    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(selected.as_path())
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn opening_an_editor_is_always_dispatched_to_the_background_worker() {
    let request = HubActionRequest {
        action_id: "open-editor".to_string(),
        target_id: None,
        payload: None,
    };

    assert!(HubRuntimeSession::should_run_action_in_background(&request));
}

#[test]
fn creating_a_project_is_dispatched_to_the_editor_owned_background_workflow() {
    let request = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Game",
            "location": "E:/Projects",
            "template": "renderable-empty"
        })),
    };

    assert!(HubRuntimeSession::should_run_action_in_background(&request));
    let projects_module = include_str!("../../../../projects/mod.rs");
    assert!(!projects_module.contains("mod create_project;"));
    assert!(!projects_module.contains("pub use create_project::"));
}

#[test]
fn background_actions_queue_while_worker_is_active_and_dequeue_fifo() {
    let temp = temp_test_dir("zircon-hub-background-action-queue");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    let build_request = background_request("build-project");
    let package_request = background_request("package-project");
    let install_request = background_request("install-device");

    let should_spawn_first = session
        .start_background_action_or_record_error(&build_request)
        .expect("first background action should start the worker");
    let running_status = session.task_status.clone();
    let should_spawn_second = session
        .start_background_action_or_record_error(&package_request)
        .expect("second background action should be queued");
    let should_spawn_third = session
        .start_background_action_or_record_error(&install_request)
        .expect("third background action should be queued");

    assert!(should_spawn_first);
    assert!(!should_spawn_second);
    assert!(!should_spawn_third);
    assert!(session.background_worker_active);
    assert_eq!(session.background_action_queue.len(), 2);
    assert_eq!(
        session.task_status, running_status,
        "queued actions must not overwrite the currently running status"
    );
    let model = session.view_model();
    assert_eq!(model.task_summary.task_id, running_status.task_id);
    assert_eq!(model.task_summary.queued, 2);

    let next_request = session
        .take_next_background_action()
        .expect("package action should be next");
    assert_eq!(next_request.action_id, "package-project");
    assert!(session.background_worker_active);

    let next_request = session
        .take_next_background_action()
        .expect("install action should follow package action");
    assert_eq!(next_request.action_id, "install-device");
    assert!(session.background_worker_active);

    assert!(session.take_next_background_action().is_none());
    assert!(!session.background_worker_active);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_action_lifecycle_dequeued_action_starts_new_running_status() {
    let temp = temp_test_dir("zircon-hub-background-dequeued-status");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    let build_request = background_request("build-project");
    let package_request = background_request("package-project");

    assert!(session
        .start_background_action_or_record_error(&build_request)
        .unwrap());
    let first_task_id = session.task_status.task_id;
    assert!(!session
        .start_background_action_or_record_error(&package_request)
        .unwrap());

    let next_request = session
        .take_next_background_action()
        .expect("queued package action should become active");

    assert_eq!(next_request.action_id, "package-project");
    assert!(session.task_status.running);
    assert_eq!(session.task_status.label, "Packaging");
    assert!(session.task_status.task_id > first_task_id);
    assert_ne!(session.task_status.task_id, 0);
    let second_task_id = session.task_status.task_id;

    session.mark_background_action_prepared();

    assert_eq!(session.task_status.task_id, second_task_id);
    assert_eq!(
        session.task_status.progress_percent,
        TASK_PROGRESS_PREPARED_PERCENT
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn cancellation_is_bound_to_the_exact_running_task_and_not_reused_by_the_queue() {
    let temp = temp_test_dir("zircon-hub-background-cancellation-identity");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    let build_request = background_request("build-project");
    let package_request = background_request("package-project");

    assert!(session
        .start_background_action_or_record_error(&build_request)
        .unwrap());
    assert!(!session
        .start_background_action_or_record_error(&package_request)
        .unwrap());
    let first_task_id = session.task_status.task_id;

    session.request_background_task_cancellation(first_task_id + 1);
    assert!(!session
        .background_cancellation_for_task(first_task_id)
        .unwrap()
        .is_cancellation_requested());

    session.request_background_task_cancellation(first_task_id);
    assert!(session
        .background_cancellation_for_task(first_task_id)
        .unwrap()
        .is_cancellation_requested());
    let task_summary = session.view_model().task_summary;
    assert!(!task_summary.cancellable);
    assert_eq!(
        task_summary.detail,
        "Cancellation requested; waiting for the current operation to stop"
    );

    session
        .take_next_background_action()
        .expect("queued task should become active");
    let second_task_id = session.task_status.task_id;
    assert_ne!(second_task_id, first_task_id);
    assert!(session.task_status.cancellable);
    assert!(!session
        .background_cancellation_for_task(second_task_id)
        .unwrap()
        .is_cancellation_requested());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn foreground_status_changes_do_not_detach_the_running_background_task() {
    let temp = temp_test_dir("zircon-hub-background-cancellation-foreground-status");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    let request = background_request("build-project");

    assert!(session
        .start_background_action_or_record_error(&request)
        .unwrap());
    let task_id = session
        .active_background_task_id()
        .expect("running background task should own an identity");

    session.discard_settings_draft();

    assert_eq!(session.task_status.label, "Settings draft discarded");
    let model = session.view_model();
    assert!(model.task_summary.running);
    assert_eq!(model.task_summary.task_id, task_id);
    assert_eq!(model.task_summary.label, "Building");

    session.request_background_task_cancellation(task_id);

    assert!(session
        .background_cancellation_for_task(task_id)
        .expect("foreground status must not detach background cancellation")
        .is_cancellation_requested());
    assert!(!session.view_model().task_summary.cancellable);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn preparation_progress_does_not_reenable_a_cancelled_background_task() {
    let temp = temp_test_dir("zircon-hub-background-cancellation-prepare");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    let request = background_request("build-project");

    assert!(session
        .start_background_action_or_record_error(&request)
        .unwrap());
    let task_id = session
        .active_background_task_id()
        .expect("running background task should own an identity");
    session.request_background_task_cancellation(task_id);

    session.mark_background_action_prepared();

    let task_summary = session.view_model().task_summary;
    assert_eq!(task_summary.task_id, task_id);
    assert_eq!(
        task_summary.progress_percent,
        TASK_PROGRESS_PREPARED_PERCENT
    );
    assert!(!task_summary.cancellable);
    assert_eq!(
        task_summary.detail,
        "Cancellation requested; waiting for the current operation to stop"
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn cancelled_execution_records_a_distinct_terminal_status_without_running_completion() {
    let temp = temp_test_dir("zircon-hub-background-cancelled-terminal");
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    let request = background_request("package-project");
    session.start_background_action_status(&request).unwrap();
    let task_id = session.task_status.task_id;
    let session_handle = Arc::new(Mutex::new(session));
    let editor_child_reaper = crate::process::EditorChildReaper::start().unwrap();

    execute_background_task(
        &request,
        &session_handle,
        &|_| {},
        &editor_child_reaper,
        prepare_cancelled_background_task,
        complete_cancelled_background_task,
    );

    let session = session_handle.lock().unwrap();
    assert_eq!(session.task_status.task_id, task_id);
    assert_eq!(session.task_status.label, "Task cancelled");
    assert_eq!(session.task_status.severity, TaskSeverity::Warning);
    assert_eq!(session.config.action_history.len(), 1);
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Cancelled
    );
    drop(session);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_action_lifecycle_success_preserves_running_task_id() {
    assert_background_completion_preserves_task_id(
        "zircon-hub-background-success-task-id",
        complete_immediate_success,
        TaskSeverity::Success,
    );
}

#[test]
fn background_action_lifecycle_failure_preserves_running_task_id() {
    assert_background_completion_preserves_task_id(
        "zircon-hub-background-failure-task-id",
        complete_immediate_failure,
        TaskSeverity::Error,
    );
}

#[test]
fn background_action_start_localizes_invalid_project_target_failure() {
    let temp = temp_test_dir("zircon-hub-background-invalid-target-localized");
    let selected = temp.join("Selected");
    fs::create_dir_all(&selected).unwrap();
    let mut session = session_with_project(&temp, "Selected", &selected);
    session.config.settings.language = crate::settings::HubLanguage::Chinese;
    let request = HubActionRequest {
        action_id: "package-project".to_string(),
        target_id: Some("missing-project".to_string()),
        payload: None,
    };

    let should_spawn = session
        .start_background_action_or_record_error(&request)
        .expect("invalid background action target should return a localized ViewModel");

    let model = session.view_model();
    assert!(!should_spawn);
    assert_eq!(model.task_summary.label, "打包失败");
    assert_eq!(
        model.task_summary.detail,
        "未知最近项目目标（package-project）：missing-project"
    );
    assert_eq!(
        model.task_summary.recovery.as_deref(),
        Some("检查操作目标后重试")
    );

    fs::remove_dir_all(temp).unwrap();
}

struct ImmediateBackgroundTask;

struct CancelledBackgroundTask;

impl BackgroundTask for CancelledBackgroundTask {
    type Output = ();

    fn run(
        &self,
        _context: &BackgroundTaskContext,
    ) -> Result<TaskExecutionOutcome<Self::Output>, HubError> {
        Ok(TaskExecutionOutcome::Cancelled)
    }
}

impl BackgroundTask for ImmediateBackgroundTask {
    type Output = ();

    fn run(
        &self,
        _context: &BackgroundTaskContext,
    ) -> Result<TaskExecutionOutcome<Self::Output>, HubError> {
        Ok(TaskExecutionOutcome::Completed(()))
    }
}

fn prepare_immediate_background_task(
    session: &mut HubRuntimeSession,
) -> Result<Option<ImmediateBackgroundTask>, HubError> {
    session.mark_background_action_prepared();
    Ok(Some(ImmediateBackgroundTask))
}

fn prepare_cancelled_background_task(
    session: &mut HubRuntimeSession,
) -> Result<Option<CancelledBackgroundTask>, HubError> {
    session.mark_background_action_prepared();
    Ok(Some(CancelledBackgroundTask))
}

fn complete_cancelled_background_task(
    _session: &mut HubRuntimeSession,
    _pending: CancelledBackgroundTask,
    _result: Result<(), HubError>,
) -> Result<(), HubError> {
    panic!("cancelled work must bypass action-specific success/failure completion")
}

fn complete_immediate_success(
    session: &mut HubRuntimeSession,
    _pending: ImmediateBackgroundTask,
    _result: Result<(), HubError>,
) -> Result<(), HubError> {
    session.task_status = TaskStatus::success("Completed", HubMessage::raw_text("completed"));
    Ok(())
}

fn complete_immediate_failure(
    _session: &mut HubRuntimeSession,
    _pending: ImmediateBackgroundTask,
    _result: Result<(), HubError>,
) -> Result<(), HubError> {
    Err(HubError::message("failed"))
}

fn assert_background_completion_preserves_task_id(
    temp_prefix: &str,
    complete: fn(
        &mut HubRuntimeSession,
        ImmediateBackgroundTask,
        Result<(), HubError>,
    ) -> Result<(), HubError>,
    expected_severity: TaskSeverity,
) {
    let temp = temp_test_dir(temp_prefix);
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_project(&temp, "Game", &project);
    let request = background_request("package-project");
    session.start_background_action_status(&request).unwrap();
    let task_id = session.task_status.task_id;
    let session_handle = Arc::new(Mutex::new(session));
    let editor_child_reaper = crate::process::EditorChildReaper::start().unwrap();

    execute_background_task(
        &request,
        &session_handle,
        &|_| {},
        &editor_child_reaper,
        prepare_immediate_background_task,
        complete,
    );

    let session = session_handle.lock().unwrap();
    assert!(!session.task_status.running);
    assert_eq!(session.task_status.severity, expected_severity);
    assert_eq!(session.task_status.task_id, task_id);
    assert_ne!(session.task_status.task_id, 0);
    drop(session);

    fs::remove_dir_all(temp).unwrap();
}

fn session_with_project(
    temp: &std::path::Path,
    name: &str,
    project: &std::path::Path,
) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_build_output_dir = temp.join("out");
    config.recent_projects = vec![RecentProject::fixture(name, project, 1)];
    config.runtime.selected_project_path = Some(project.to_path_buf());
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn session_with_projects(
    temp: &std::path::Path,
    projects: &[(&str, PathBuf)],
    selected_project: &std::path::Path,
) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_build_output_dir = temp.join("out");
    config.recent_projects = projects
        .iter()
        .map(|(name, path)| RecentProject::fixture(*name, path, 1))
        .collect();
    config.runtime.selected_project_path = Some(selected_project.to_path_buf());
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn background_request(action_id: &str) -> HubActionRequest {
    HubActionRequest {
        action_id: action_id.to_string(),
        target_id: None,
        payload: None,
    }
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
