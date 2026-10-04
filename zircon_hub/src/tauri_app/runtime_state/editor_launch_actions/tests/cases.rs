use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::engines::SourceEngineInstall;
use crate::process::editor_child_receipt::{EditorChildDisposition, EditorChildTerminalReceipt};
use crate::projects::{metadata_for_path, project_metadata_key, ProjectMetadata, RecentProject};
use crate::settings::{HubConfig, HubLanguage};
use crate::state::{HubActionKind, HubActionStatus, HubMessage, HubMessageId, ProcessMessageId};
use zircon_runtime_interface::hub_protocol::{
    HubEditorMailboxV1, HubEditorReadyReceiptV1, HubEditorStartupFailureCodeV1, HubSessionToken,
};

use super::super::{HubActionRequest, HubRuntimeSession};
use super::{
    validate_project_editor_handshake, EditorLaunchOutcome, EditorLaunchReport, PendingEditorLaunch,
};

#[test]
fn editor_handshake_accepts_a_ready_receipt_bound_to_the_supervised_child() {
    let session = HubSessionToken::new();
    assert_eq!(
        validate_project_editor_handshake(
            913,
            HubEditorMailboxV1::ready(
                session,
                HubEditorReadyReceiptV1::after_first_present(913, "913-42", 1)
                    .expect("ready receipt"),
            ),
        )
        .expect("matching ready mailbox"),
        913
    );

    let mismatch = validate_project_editor_handshake(
        913,
        HubEditorMailboxV1::ready(
            session,
            HubEditorReadyReceiptV1::after_first_present(914, "914-42", 1).expect("ready receipt"),
        ),
    )
    .expect_err("a different child process must not be reported ready");
    assert!(mismatch
        .to_string()
        .contains("not bound to the Hub-supervised child"));
}

#[test]
fn editor_handshake_surfaces_the_editor_terminal_failure() {
    let error = validate_project_editor_handshake(
        913,
        HubEditorMailboxV1::failed(
            HubSessionToken::new(),
            HubEditorStartupFailureCodeV1::ProjectActivation,
        ),
    )
    .expect_err("editor reported failure");

    assert_eq!(
        error.to_string(),
        "editor reported startup failure category through the Hub handshake: project_activation"
    );
}

#[test]
fn background_editor_launch_prepare_records_missing_executable_failure_without_spawn() {
    let temp = temp_test_dir("zircon-hub-background-editor-missing");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);

    let pending = session
        .prepare_background_editor_launch()
        .expect("missing editor should be a recoverable visible failure");

    assert!(pending.is_none());
    let record = &session.config.action_history[0];
    assert_eq!(record.action, HubActionKind::OpenEditor);
    assert_eq!(record.status, HubActionStatus::Failed);
    assert_eq!(session.task_status.label, "Open Editor failed");
    assert!(record.recovery.as_ref().unwrap().contains("editor/runtime"));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn editor_launch_missing_executable_failure_localizes_task_summary() {
    let temp = temp_test_dir("zircon-hub-background-editor-missing-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;

    let pending = session
        .prepare_background_editor_launch()
        .expect("missing editor should be a recoverable visible failure");

    assert!(pending.is_none());
    let model = session.view_model();
    assert_eq!(model.task_summary.label, "打开编辑器失败");
    assert_eq!(
        model.task_summary.detail,
        format!(
            "编辑器可执行文件不可用：{}",
            crate::process::staged_editor_executable(session.staged_engine_dir()).to_string_lossy()
        )
    );
    assert_eq!(
        model.task_summary.recovery.as_deref(),
        Some("打开项目前先构建编辑器/运行时载荷，或修复源码引擎设置")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn editor_launch_rejects_missing_bound_source_engine_before_spawn() {
    let temp = temp_test_dir("zircon-hub-editor-missing-bound-source");
    let project = create_project_root(&temp, "Game");
    let missing_source = temp.join("missing-source");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.engines.push(SourceEngineInstall {
        id: "missing-engine".to_string(),
        display_name: "Missing Engine".to_string(),
        source_dir: missing_source,
        output_dir: temp.join("out"),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    session.config.active_engine_id = Some("missing-engine".to_string());
    session.config.project_metadata.insert(
        project_metadata_key(&project),
        ProjectMetadata {
            engine_id: Some("missing-engine".to_string()),
            ..ProjectMetadata::default()
        },
    );

    let pending = session
        .prepare_background_editor_launch()
        .expect("invalid bound source engine should be a recoverable visible failure");

    assert!(pending.is_none());
    assert_eq!(session.task_status.label, "Open Editor failed");
    assert_eq!(
        session.task_status.detail,
        "Source checkout directory is missing"
    );
    assert_eq!(session.config.action_history.len(), 1);
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn project_creation_prepare_is_data_only_until_editor_admission_completes() {
    let temp = temp_test_dir("zircon-hub-editor-owned-create-prepare");
    let mut session = session_with_create_engine(&temp);
    let project_root = temp.join("projects").join("Game");
    let request = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Game",
            "location": temp.join("projects"),
            "template": "renderable-empty",
            "engineId": "test-engine",
        })),
    };

    let pending = session
        .prepare_background_project_creation(&request)
        .expect("valid creation request should prepare an Editor launch")
        .expect("valid creation request should produce pending work");

    assert_eq!(pending.project_root, project_root);
    assert!(!project_root.exists());
    assert!(session.config.recent_projects.is_empty());
    assert!(session.config.action_history.is_empty());
    let intent = serde_json::from_str::<zircon_runtime_interface::project::ProjectLaunchIntent>(
        &pending.command.args[1],
    )
    .expect("prepared command must carry a typed project launch intent");
    assert!(matches!(
        intent.target(),
        zircon_runtime_interface::project::ProjectLaunchTarget::CreateProject {
            project_name,
            location,
            template: zircon_runtime_interface::project::ProjectTemplateId::RenderableEmpty,
        } if project_name == "Game" && location == temp.join("projects").as_path()
    ));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn project_creation_rejects_a_tampered_staged_editor_before_spawn() {
    let temp = temp_test_dir("zircon-hub-editor-owned-create-tampered-buildset");
    let mut session = session_with_create_engine(&temp);
    let staged_editor =
        crate::process::staged_editor_executable(session.config.engines[0].staged_engine_dir());
    fs::write(staged_editor, b"tampered editor").unwrap();
    let request = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Game",
            "location": temp.join("projects"),
            "template": "renderable-empty",
            "engineId": "test-engine",
        })),
    };

    let pending = session
        .prepare_background_project_creation(&request)
        .expect("tampered BuildSet should be a visible recoverable failure");

    assert!(pending.is_none());
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );
    assert!(session.config.action_history[0]
        .detail
        .contains("does not match its staged SHA-256"));

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn project_creation_is_registered_only_after_editor_ready_completion() {
    let temp = temp_test_dir("zircon-hub-editor-owned-create-complete");
    let mut session = session_with_create_engine(&temp);
    let project_root = temp.join("projects").join("Game");
    let request = HubActionRequest {
        action_id: "create-project".to_string(),
        target_id: None,
        payload: Some(serde_json::json!({
            "name": "Game",
            "location": temp.join("projects"),
            "template": "renderable-empty",
            "engineId": "test-engine",
        })),
    };
    let pending = session
        .prepare_background_project_creation(&request)
        .unwrap()
        .unwrap();
    create_project_root_at(&project_root, "Game");

    session
        .complete_background_project_creation(
            pending,
            Ok(EditorLaunchReport {
                attempt_id: 1,
                process_id: 42,
                outcome: EditorLaunchOutcome::Spawned,
            }),
        )
        .expect("Ready completion should register the Editor-created project");

    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(project_root.as_path())
    );
    assert_eq!(session.config.recent_projects.len(), 1);
    let metadata = metadata_for_path(&session.config.project_metadata, &project_root).unwrap();
    assert_eq!(metadata.engine_id.as_deref(), Some("test-engine"));
    assert_eq!(
        metadata.last_selected_template.as_deref(),
        Some("renderable-empty")
    );
    let record = &session.config.action_history[0];
    assert_eq!(record.action, HubActionKind::CreateProject);
    assert_eq!(record.status, HubActionStatus::Success);
    assert_eq!(record.process_id, Some(42));
    assert!(record
        .command_line
        .iter()
        .any(|arg| arg == "--project-launch-intent"));
    assert_eq!(session.task_status.label, "Project created");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn background_editor_launch_completion_records_success_after_external_spawn() {
    let temp = temp_test_dir("zircon-hub-background-editor-complete");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    let pending = PendingEditorLaunch {
        target: "Game".to_string(),
        command: super::EditorLaunchPreparedCommand::Empty {
            executable: temp.join("zircon_editor.exe"),
        },
        project_path: Some(project.clone()),
        remember_project: true,
        recovery_on_launch_failure: HubMessage::new(HubMessageId::Process(
            ProcessMessageId::VerifyEditorExecutable,
        )),
    };

    session
        .complete_background_editor_launch(
            pending,
            Ok(EditorLaunchReport {
                attempt_id: 1,
                process_id: 42,
                outcome: EditorLaunchOutcome::Spawned,
            }),
        )
        .expect("editor launch completion should record success");

    let record = &session.config.action_history[0];
    assert_eq!(record.action, HubActionKind::OpenEditor);
    assert_eq!(record.status, HubActionStatus::Success);
    assert_eq!(record.process_id, Some(42));
    assert_eq!(session.task_status.label, "Editor launched");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn editor_exit_before_launch_completion_is_published_after_ready_once() {
    let temp = temp_test_dir("zircon-hub-editor-early-exit");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    let receipt = failed_editor_receipt(41, 913);
    assert!(!session
        .observe_editor_child_terminal(receipt.clone())
        .expect("early exit is buffered"));

    session
        .complete_background_editor_launch(
            pending_empty_editor_launch(&temp, &project),
            Ok(EditorLaunchReport {
                attempt_id: 41,
                process_id: 913,
                outcome: EditorLaunchOutcome::Spawned,
            }),
        )
        .expect("Ready completion reconciles early terminal");

    assert_eq!(session.config.action_history.len(), 2);
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );
    assert_eq!(session.config.action_history[0].process_id, Some(913));
    assert_eq!(
        session.config.action_history[1].status,
        HubActionStatus::Success
    );
    assert_eq!(session.task_status.task_id, 41);
    assert_eq!(session.task_status.label, "Open Editor failed");
    assert!(!session
        .observe_editor_child_terminal(receipt)
        .expect("duplicate exit is ignored"));
    assert_eq!(session.config.action_history.len(), 2);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn pre_ready_cancel_records_a_pid_bound_terminal_without_launch_success() {
    let temp = temp_test_dir("zircon-hub-editor-pre-ready-cancel");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session
        .start_background_action_status(&HubActionRequest {
            action_id: "open-editor".to_string(),
            target_id: None,
            payload: None,
        })
        .unwrap();
    session.finish_background_task(1);

    assert!(session
        .observe_editor_child_terminal(EditorChildTerminalReceipt::stopped_before_ready(
            1, 913, None,
        ))
        .expect("publish pre-Ready reap receipt"));

    assert_eq!(session.config.action_history.len(), 1);
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Cancelled
    );
    assert_eq!(session.config.action_history[0].process_id, Some(913));
    assert_eq!(session.config.action_history[0].target, "Game");
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn late_editor_exit_does_not_replace_newer_task_status() {
    let temp = temp_test_dir("zircon-hub-editor-late-exit");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session
        .complete_background_editor_launch(
            pending_empty_editor_launch(&temp, &project),
            Ok(EditorLaunchReport {
                attempt_id: 41,
                process_id: 913,
                outcome: EditorLaunchOutcome::Spawned,
            }),
        )
        .expect("launch completion");
    session.task_status = crate::state::TaskStatus::running_operation(
        "Building",
        HubMessage::empty(),
        crate::state::TaskOperationKind::Build,
        "Other project",
    )
    .with_task_id(42);

    assert!(session
        .observe_editor_child_terminal(failed_editor_receipt(41, 913))
        .expect("late exit is recorded"));
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );
    assert_eq!(session.task_status.task_id, 42);
    assert_eq!(session.task_status.label, "Building");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn terminal_history_retries_durable_save_without_duplicate_record() {
    let temp = temp_test_dir("zircon-hub-editor-terminal-save-retry");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session
        .complete_background_editor_launch(
            pending_empty_editor_launch(&temp, &project),
            Ok(EditorLaunchReport {
                attempt_id: 41,
                process_id: 913,
                outcome: EditorLaunchOutcome::Spawned,
            }),
        )
        .expect("launch completion");
    let config_path = session.config_path.clone();
    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, b"not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");
    let receipt = failed_editor_receipt(41, 913);

    assert!(session
        .observe_editor_child_terminal(receipt.clone())
        .is_err());
    assert_eq!(session.config.action_history.len(), 2);
    session.config_path = config_path.clone();
    assert!(session
        .observe_editor_child_terminal(receipt)
        .expect("retry terminal save after config path recovers"));

    let persisted = HubConfig::load(&config_path).unwrap();
    assert_eq!(persisted.action_history.len(), 2);
    assert_eq!(persisted.action_history[0].status, HubActionStatus::Failed);
    assert_eq!(persisted.action_history[0].process_id, Some(913));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn failed_terminal_save_survives_action_history_churn_before_retry() {
    let temp = temp_test_dir("zircon-hub-editor-terminal-save-churn");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session
        .complete_background_editor_launch(
            pending_empty_editor_launch(&temp, &project),
            Ok(EditorLaunchReport {
                attempt_id: 41,
                process_id: 913,
                outcome: EditorLaunchOutcome::Spawned,
            }),
        )
        .expect("launch completion");
    let config_path = session.config_path.clone();
    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, b"not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");
    assert!(session
        .observe_editor_child_terminal(failed_editor_receipt(41, 913))
        .is_err());
    let mut churn_record = session.config.action_history[1].clone();
    churn_record.finished_unix_ms = session.config.action_history[0].finished_unix_ms + 1;
    churn_record.process_id = None;
    churn_record.detail = HubMessage::raw_text("unrelated action history");
    for _ in 0..=crate::state::ACTION_HISTORY_LIMIT {
        assert!(session
            .record_action_and_persist(churn_record.clone())
            .is_err());
    }
    assert!(session
        .config
        .action_history
        .iter()
        .all(|record| record.process_id != Some(913)));

    session.config_path = config_path.clone();
    session
        .record_action_and_persist(churn_record.clone())
        .expect("newer unrelated action can persist before terminal retry");
    assert!(session
        .retry_pending_editor_terminal_persistence()
        .expect("recover terminal history after action churn"));
    let persisted = HubConfig::load(&config_path).unwrap();
    assert_eq!(persisted.action_history[0].detail, churn_record.detail);
    assert!(persisted.action_history.iter().any(|record| {
        record.action == HubActionKind::OpenEditor
            && record.status == HubActionStatus::Failed
            && record.process_id == Some(913)
    }));
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn failed_terminal_saves_retry_in_delivery_order_with_reused_pid() {
    let temp = temp_test_dir("zircon-hub-editor-terminal-save-order");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session
        .observe_completed_editor_launch(41, 913, "Game".to_string())
        .unwrap();
    session
        .observe_completed_editor_launch(42, 913, "Game".to_string())
        .unwrap();
    let config_path = session.config_path.clone();
    let blocked_parent = temp.join("blocked-parent");
    fs::write(&blocked_parent, b"not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");
    assert!(session
        .observe_editor_child_terminal(failed_editor_receipt(41, 913))
        .is_err());
    assert!(session
        .observe_editor_child_terminal(failed_editor_receipt(42, 913))
        .is_err());
    session.config.action_history.clear();

    session.config_path = config_path.clone();
    assert!(session
        .retry_pending_editor_terminal_persistence()
        .expect("retry two terminal records"));
    let persisted = HubConfig::load(&config_path).unwrap();
    assert_eq!(persisted.action_history[0].process_id, Some(913));
    assert_eq!(persisted.action_history[1].process_id, Some(913));
    assert_eq!(
        persisted.action_history[0].log_excerpt,
        HubMessage::with_params(
            HubMessageId::Process(ProcessMessageId::EditorTerminalAttemptId),
            ["42".to_string()],
        )
    );
    assert_eq!(
        persisted.action_history[1].log_excerpt,
        HubMessage::with_params(
            HubMessageId::Process(ProcessMessageId::EditorTerminalAttemptId),
            ["41".to_string()],
        )
    );
    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn shutdown_deadline_records_incomplete_editor_owners() {
    let temp = temp_test_dir("zircon-hub-editor-incomplete-shutdown");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);

    session
        .record_editor_supervision_incomplete(Some(41), true, &[])
        .expect("persist incomplete owner receipts");

    assert_eq!(session.config.action_history.len(), 2);
    assert!(session.config.action_history.iter().all(|record| {
        record.action == HubActionKind::OpenEditor && record.status == HubActionStatus::Failed
    }));
    assert_eq!(
        session.config.action_history[0].detail,
        HubMessage::new(HubMessageId::Process(
            ProcessMessageId::EditorChildShutdownIncomplete,
        )),
    );
    assert_eq!(
        session.config.action_history[1].detail,
        HubMessage::with_params(
            HubMessageId::Process(ProcessMessageId::EditorLaunchShutdownIncomplete),
            ["41".to_string()],
        ),
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn shutdown_records_unprojected_terminal_attempt_and_pid() {
    let temp = temp_test_dir("zircon-hub-editor-unprojected-shutdown");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session
        .record_editor_supervision_incomplete(
            None,
            false,
            &[EditorChildTerminalReceipt::stopped_before_ready(
                41, 913, None,
            )],
        )
        .expect("persist incomplete terminal projection");
    let record = &session.config.action_history[0];
    assert_eq!(record.process_id, Some(913));
    assert_eq!(
        record.detail,
        HubMessage::with_params(
            HubMessageId::Process(ProcessMessageId::EditorTerminalProjectionIncomplete),
            ["41".to_string(), "913".to_string()],
        ),
    );
    fs::remove_dir_all(temp).unwrap();
}

fn failed_editor_receipt(attempt_id: u64, process_id: u32) -> EditorChildTerminalReceipt {
    EditorChildTerminalReceipt {
        attempt_id,
        process_id,
        disposition: EditorChildDisposition::Exited {
            code: Some(7),
            signal: None,
        },
        cleanup_error: None,
    }
}

fn pending_empty_editor_launch(temp: &Path, project: &Path) -> PendingEditorLaunch {
    PendingEditorLaunch {
        target: "Game".to_string(),
        command: super::EditorLaunchPreparedCommand::Empty {
            executable: temp.join("zircon_editor.exe"),
        },
        project_path: Some(project.to_path_buf()),
        remember_project: true,
        recovery_on_launch_failure: HubMessage::new(HubMessageId::Process(
            ProcessMessageId::VerifyEditorExecutable,
        )),
    }
}

#[test]
fn editor_launch_completion_localizes_task_summary_and_history() {
    let temp = temp_test_dir("zircon-hub-background-editor-complete-localized");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);
    session.config.settings.language = HubLanguage::Chinese;
    let pending = PendingEditorLaunch {
        target: "Game".to_string(),
        command: super::EditorLaunchPreparedCommand::Empty {
            executable: temp.join("zircon_editor.exe"),
        },
        project_path: Some(project.clone()),
        remember_project: true,
        recovery_on_launch_failure: HubMessage::new(HubMessageId::Process(
            ProcessMessageId::VerifyEditorExecutable,
        )),
    };

    session
        .complete_background_editor_launch(
            pending,
            Ok(EditorLaunchReport {
                attempt_id: 1,
                process_id: 42,
                outcome: EditorLaunchOutcome::Spawned,
            }),
        )
        .expect("editor launch completion should record success");

    let model = session.view_model();
    assert_eq!(model.task_summary.label, "编辑器已启动");
    assert_eq!(model.task_summary.detail, "正在打开 Game（进程 42）");
    assert_eq!(model.action_history[0].action, "打开编辑器");
    assert_eq!(model.action_history[0].detail, "已启动进程 42");

    fs::remove_dir_all(temp).unwrap();
}

fn session_with_project(
    temp: &std::path::Path,
    name: &str,
    project: &std::path::Path,
) -> HubRuntimeSession {
    let source = temp.join("source");
    fs::create_dir_all(source.join("tools")).unwrap();
    fs::write(
        source.join("Cargo.toml"),
        "[workspace]\nmembers = [\"zircon_runtime\"]\n",
    )
    .unwrap();
    fs::write(source.join("tools").join("zircon_build.py"), "").unwrap();
    let output = temp.join("out");
    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_source_dir = source.clone();
    config.settings.default_build_output_dir = output.clone();
    config.engines.push(SourceEngineInstall {
        id: "test-engine".to_string(),
        display_name: "Test Engine".to_string(),
        source_dir: source,
        output_dir: output,
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    config.active_engine_id = Some("test-engine".to_string());
    config.recent_projects = vec![RecentProject::fixture(name, project, 1)];
    config.runtime.selected_project_path = Some(project.to_path_buf());
    config.project_metadata.insert(
        project_metadata_key(project),
        ProjectMetadata {
            engine_id: Some("test-engine".to_string()),
            ..ProjectMetadata::default()
        },
    );
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn session_with_create_engine(temp: &Path) -> HubRuntimeSession {
    let source = temp.join("source");
    fs::create_dir_all(source.join("tools")).unwrap();
    fs::write(
        source.join("Cargo.toml"),
        "[workspace]\nmembers = [\"zircon_runtime\"]\n",
    )
    .unwrap();
    fs::write(source.join("tools").join("zircon_build.py"), "").unwrap();

    let output = temp.join("out");
    let staged_engine = output.join("ZirconEngine");
    super::super::build_actions::staged_build::write_valid_test_fixture(
        &staged_engine,
        &source,
        HubConfig::default().settings.build_profile.as_mode(),
    );

    let config_path = temp.join("hub.toml");
    let shared_recent_projects_path = temp.join("recent_projects.json");
    let mut config = HubConfig::default();
    config.settings.default_project_dir = temp.join("projects");
    config.settings.default_source_dir = source.clone();
    config.settings.default_build_output_dir = output.clone();
    config.engines.push(SourceEngineInstall {
        id: "test-engine".to_string(),
        display_name: "Test Engine".to_string(),
        source_dir: source,
        output_dir: output,
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    config.active_engine_id = Some("test-engine".to_string());
    config.runtime.new_project_engine_id = Some("test-engine".to_string());
    config.save(&config_path).unwrap();
    fs::write(
        &shared_recent_projects_path,
        r#"{"protocol_version":1,"projects":[]}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, shared_recent_projects_path).unwrap()
}

fn create_project_root(temp: &std::path::Path, name: &str) -> PathBuf {
    let project = temp.join(name);
    create_project_root_at(&project, name);
    project
}

fn create_project_root_at(project: &Path, name: &str) {
    fs::create_dir_all(project.join("Assets")).unwrap();
    fs::write(
        project.join("zircon-project.toml"),
        format!(
            "name = {name:?}\nformat_version = 3\nproject_guid = \"0e62bc49-5a20-430e-9902-aa4b9b6518a4\"\ndefault_scene = \"res://scenes/main.scene.toml\"\nasset_roots = [\"assets\"]\nlibrary_version = 1\n"
        ),
    )
    .unwrap();
    fs::write(project.join("Assets").join("mesh.txt"), "mesh").unwrap();
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let target_directory = std::env::var_os("CARGO_TARGET_DIR")
        .expect("Hub editor-launch filesystem tests require coordinator-managed CARGO_TARGET_DIR");
    let path = PathBuf::from(target_directory).join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    let _ = fs::remove_dir_all(&path);
    fs::create_dir_all(&path).unwrap();
    path
}
