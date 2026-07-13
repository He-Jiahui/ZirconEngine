use std::{fs, path::PathBuf};

use crate::{
    engines::{source_engine_id, SourceEngineInstall},
    error::HubError,
    projects::{metadata_for_path, project_metadata_key, RecentProject},
    settings::{HubConfig, HubLanguage},
    state::{HubActionKind, HubActionStatus},
    tauri_app::view_model::HubTextBundle,
};

use super::super::{HubActionRequest, HubRuntimeSession};
use super::CREATE_KEPT_FOLDER_RECOVERY;

#[test]
fn create_project_action_scaffolds_project_and_selects_detail() {
    let temp = temp_test_dir("zircon-hub-create-project-action");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let mut session = session_with_source(&temp, &source);
    let engine_id = source_engine_id(&source);

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": temp.join("projects").to_string_lossy(),
                "template": "renderable-empty",
                "engineId": engine_id,
            })),
        })
        .expect("create-project should return refreshed state");

    let project = temp.join("projects").join("Game");
    assert!(project.join("zircon-project.toml").is_file());
    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(project.as_path())
    );
    assert_eq!(session.project_subpage.id(), "project-detail");
    assert_eq!(view_model.selected_project.as_ref().unwrap().name, "Game");
    let metadata = metadata_for_path(&session.config.project_metadata, &project).unwrap();
    assert_eq!(metadata.engine_id.as_deref(), Some(engine_id.as_str()));
    assert_eq!(
        metadata.last_selected_template.as_deref(),
        Some("renderable-empty")
    );
    assert_eq!(
        session.config.action_history[0].action,
        HubActionKind::CreateProject
    );
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Success
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn create_project_action_marks_disabled_templates_as_coming_soon() {
    let temp = temp_test_dir("zircon-hub-create-project-coming-soon");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let mut session = session_with_source(&temp, &source);

    session
        .apply_action(HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": temp.join("projects").to_string_lossy(),
                "template": "3d-scene",
            })),
        })
        .expect("disabled template should be a recoverable Hub error");

    assert_eq!(session.task_status.label, "Create Project failed");
    assert!(session.task_status.detail.contains("coming soon"));
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn create_project_disabled_template_failure_localizes_task_summary() {
    let temp = temp_test_dir("zircon-hub-create-project-coming-soon-localized");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let mut session = session_with_source(&temp, &source);
    session.config.settings.language = HubLanguage::Chinese;

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": temp.join("projects").to_string_lossy(),
                "template": "3d-scene",
            })),
        })
        .expect("disabled template should localize the recoverable Hub error");

    assert_eq!(view_model.task_summary.label, "创建项目失败");
    assert_eq!(view_model.task_summary.detail, "项目模板尚未开放：3d-scene");
    assert_eq!(
        view_model.task_summary.recovery.as_deref(),
        Some("v1 本地项目创建请选择“可渲染空项目”模板")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn create_project_non_empty_target_recovery_points_to_import() {
    let temp = temp_test_dir("zircon-hub-create-project-non-empty-recovery");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let target = temp.join("projects").join("Game");
    fs::create_dir_all(&target).unwrap();
    write_valid_project_manifest(&target, "Game");
    let mut session = session_with_source(&temp, &source);
    let engine_id = source_engine_id(&source);

    session
        .apply_action(HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": temp.join("projects").to_string_lossy(),
                "template": "renderable-empty",
                "engineId": engine_id,
            })),
        })
        .expect("non-empty create target should be recoverable");

    assert_eq!(session.task_status.label, "Create Project failed");
    assert_eq!(session.task_status.detail, "Target directory must be empty");
    assert_eq!(
        session
            .task_status
            .recovery
            .as_ref()
            .map(|message| message.render(HubLanguage::English))
            .as_deref(),
        Some("If the folder already contains a project, use Import Project; otherwise choose an empty target folder")
    );
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn create_project_recording_failure_keeps_folder_and_points_recovery_to_import() {
    let temp = temp_test_dir("zircon-hub-create-project-record-failure");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let mut session = session_with_source(&temp, &source);
    let blocked_parent = temp.join("blocked-config-parent");
    fs::write(&blocked_parent, "not a directory").unwrap();
    session.config_path = blocked_parent.join("hub.toml");
    let engine_id = source_engine_id(&source);

    session
        .apply_action(HubActionRequest {
            action_id: "create-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "name": "Game",
                "location": temp.join("projects").to_string_lossy(),
                "template": "renderable-empty",
                "engineId": engine_id,
            })),
        })
        .expect("recording failure should remain a recoverable Hub state");

    let project = temp.join("projects").join("Game");
    assert!(project.join("zircon-project.toml").is_file());
    assert_eq!(session.task_status.label, "Create Project failed");
    assert!(session
        .task_status
        .detail
        .contains("Project folder was created at"));
    assert_eq!(
        session
            .task_status
            .recovery
            .as_ref()
            .map(|message| message.render(HubLanguage::English))
            .as_deref(),
        Some(CREATE_KEPT_FOLDER_RECOVERY)
    );
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );
    assert_eq!(
        session.config.action_history[0].output_dir.as_deref(),
        Some(project.as_path())
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn import_project_action_validates_manifest_and_records_recent_project() {
    let temp = temp_test_dir("zircon-hub-import-project-action");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let project = temp.join("Imported");
    fs::create_dir_all(&project).unwrap();
    write_valid_project_manifest(&project, "Imported");
    let mut session = session_with_source(&temp, &source);

    session
        .apply_action(HubActionRequest {
            action_id: "import-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({ "path": project.to_string_lossy() })),
        })
        .expect("import-project should accept a valid manifest folder");

    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(project.as_path())
    );
    assert_eq!(session.config.recent_projects[0].summary.name, "Imported");
    assert_eq!(
        session.config.action_history[0].action,
        HubActionKind::ImportProject
    );
    assert_eq!(session.task_status.label, "Project imported");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn import_project_action_rejects_folder_without_manifest() {
    let temp = temp_test_dir("zircon-hub-import-project-invalid");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let project = temp.join("Imported");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_source(&temp, &source);

    session
        .apply_action(HubActionRequest {
            action_id: "import-project".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .expect("invalid import should be a recoverable Hub error");

    assert_eq!(session.task_status.label, "Import Project failed");
    assert!(session.config.recent_projects.is_empty());
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn import_project_missing_manifest_failure_localizes_task_summary() {
    let temp = temp_test_dir("zircon-hub-import-project-invalid-localized");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let project = temp.join("Imported");
    fs::create_dir_all(&project).unwrap();
    let mut session = session_with_source(&temp, &source);
    session.config.settings.language = HubLanguage::Chinese;

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "import-project".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .expect("invalid import should localize the recoverable Hub error");

    assert_eq!(view_model.task_summary.label, "导入项目失败");
    assert_eq!(
        view_model.task_summary.detail,
        format!(
            "未在 {} 找到 zircon-project.toml",
            project.to_string_lossy()
        )
    );
    assert_eq!(
        view_model.task_summary.recovery.as_deref(),
        Some("选择包含 zircon-project.toml 的文件夹")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn import_project_duplicate_path_selects_existing_entry_without_new_row() {
    let temp = temp_test_dir("zircon-hub-import-project-duplicate");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let project = temp.join("Imported");
    fs::create_dir_all(&project).unwrap();
    write_valid_project_manifest(&project, "Imported");
    let mut session = session_with_source(&temp, &source);
    session.config.recent_projects = vec![RecentProject::fixture("Existing Imported", &project, 1)];

    session
        .apply_action(HubActionRequest {
            action_id: "import-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({ "path": project.join(".") })),
        })
        .expect("duplicate import should select the existing project entry");

    assert_eq!(session.config.recent_projects.len(), 1);
    assert_eq!(
        session.config.recent_projects[0].summary.name,
        "Existing Imported"
    );
    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(project.as_path())
    );
    assert_eq!(
        session.task_status.target.as_deref(),
        Some("Existing Imported")
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn import_project_invalid_manifest_is_recoverable_failure() {
    let temp = temp_test_dir("zircon-hub-import-project-broken-manifest");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let project = temp.join("Imported");
    fs::create_dir_all(&project).unwrap();
    fs::write(project.join("zircon-project.toml"), "name = \"Imported\n").unwrap();
    let mut session = session_with_source(&temp, &source);

    session
        .apply_action(HubActionRequest {
            action_id: "import-project".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .expect("invalid manifest should be a recoverable Hub error");

    assert_eq!(session.task_status.label, "Import Project failed");
    assert!(session
        .task_status
        .detail
        .contains("zircon-project.toml could not be parsed"));
    assert!(session.config.recent_projects.is_empty());
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn import_project_picker_cancel_keeps_state_without_history() {
    let temp = temp_test_dir("zircon-hub-import-picker-cancel");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let mut session = session_with_source(&temp, &source);
    session.folder_picker = |_| Ok(None);

    session
        .apply_action(HubActionRequest {
            action_id: "import-project".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("picker cancel should refresh state without error");

    assert_eq!(session.task_status.label, "Import cancelled");
    assert!(session.config.action_history.is_empty());
    assert!(session.config.recent_projects.is_empty());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn import_project_picker_error_records_failed_history() {
    let temp = temp_test_dir("zircon-hub-import-picker-error");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let mut session = session_with_source(&temp, &source);
    session.folder_picker = |_| Err(HubError::message("picker boom"));

    session
        .apply_action(HubActionRequest {
            action_id: "import-project".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("picker error should be recorded as a recoverable failure");

    assert_eq!(session.task_status.label, "Import Project failed");
    assert_eq!(session.task_status.detail, "picker boom");
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn import_project_folder_picker_title_uses_current_language() {
    let chinese = HubTextBundle::new(HubLanguage::Chinese);
    let english = HubTextBundle::new(HubLanguage::English);

    assert_eq!(
        super::import_project_picker_title(chinese),
        "导入 Zircon 项目"
    );
    assert_eq!(
        super::import_project_picker_title(english),
        "Import Zircon Project"
    );
}

#[test]
fn pin_remove_and_delete_request_update_project_metadata_and_selection() {
    let temp = temp_test_dir("zircon-hub-project-lifecycle");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    write_valid_project_manifest(&project, "Game");
    let mut session = session_with_source(&temp, &source);
    session.config.recent_projects =
        vec![crate::projects::RecentProject::fixture("Game", &project, 1)];
    session.selected_project_path = Some(project.clone());

    session
        .apply_action(HubActionRequest {
            action_id: "pin-project".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .unwrap();
    assert!(session.config.project_metadata[&project_metadata_key(&project)].pinned);

    session
        .apply_action(HubActionRequest {
            action_id: "request-delete".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .unwrap();
    assert_eq!(
        session.pending_delete_project_path.as_deref(),
        Some(project.as_path())
    );

    session
        .apply_action(HubActionRequest {
            action_id: "cancel-delete".to_string(),
            target_id: None,
            payload: None,
        })
        .unwrap();
    assert!(session.pending_delete_project_path.is_none());

    session
        .apply_action(HubActionRequest {
            action_id: "remove-from-hub".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .unwrap();
    assert!(session.config.recent_projects.is_empty());
    assert!(session.selected_project_path.is_none());
    assert!(project.join("zircon-project.toml").is_file());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn project_management_actions_resolve_typed_project_path_before_archived_target() {
    let temp = temp_test_dir("zircon-hub-project-management-typed-target");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let fallback = temp.join("Fallback");
    let target = temp.join("Target");
    fs::create_dir_all(&fallback).unwrap();
    fs::create_dir_all(&target).unwrap();
    write_valid_project_manifest(&fallback, "Fallback");
    write_valid_project_manifest(&target, "Target");
    let mut session = session_with_source(&temp, &source);
    session.config.recent_projects = vec![
        crate::projects::RecentProject::fixture("Fallback", &fallback, 1),
        crate::projects::RecentProject::fixture("Target", &target, 2),
    ];
    session.selected_project_path = Some(fallback.clone());

    session
        .apply_action(HubActionRequest {
            action_id: "pin-project".to_string(),
            target_id: Some(fallback.to_string_lossy().into_owned()),
            payload: Some(serde_json::json!({
                "projectId": fallback,
                "projectPath": target
            })),
        })
        .expect("project management actions should accept typed project targets");

    assert!(
        metadata_for_path(&session.config.project_metadata, &target)
            .unwrap()
            .pinned
    );
    assert!(metadata_for_path(&session.config.project_metadata, &fallback).is_none());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn cancel_delete_project_target_must_match_pending_project() {
    let temp = temp_test_dir("zircon-hub-project-cancel-delete-typed-target");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let fallback = temp.join("Fallback");
    let target = temp.join("Target");
    fs::create_dir_all(&fallback).unwrap();
    fs::create_dir_all(&target).unwrap();
    write_valid_project_manifest(&fallback, "Fallback");
    write_valid_project_manifest(&target, "Target");
    let mut session = session_with_source(&temp, &source);
    session.config.recent_projects = vec![
        crate::projects::RecentProject::fixture("Fallback", &fallback, 1),
        crate::projects::RecentProject::fixture("Target", &target, 2),
    ];
    session.selected_project_path = Some(fallback.clone());
    session.pending_delete_project_path = Some(target.clone());

    session
        .apply_action(HubActionRequest {
            action_id: "cancel-delete".to_string(),
            target_id: Some(target.to_string_lossy().into_owned()),
            payload: Some(serde_json::json!({
                "projectId": "fallback",
                "projectPath": fallback
            })),
        })
        .expect_err(
            "cancel-delete should reject a typed target that does not match pending delete",
        );
    assert_eq!(
        session.pending_delete_project_path.as_deref(),
        Some(target.as_path())
    );

    session
        .apply_action(HubActionRequest {
            action_id: "cancel-delete".to_string(),
            target_id: Some(fallback.to_string_lossy().into_owned()),
            payload: Some(serde_json::json!({
                "projectId": "target",
                "projectPath": target
            })),
        })
        .expect("cancel-delete should accept the pending project typed target");

    assert!(session.pending_delete_project_path.is_none());
    assert_eq!(session.task_status.label, "Delete cancelled");

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn confirm_delete_failure_keeps_pending_state_with_recovery() {
    let temp = temp_test_dir("zircon-hub-project-confirm-delete-failure");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    write_valid_project_manifest(&project, "Game");
    let mut session = session_with_source(&temp, &source);
    session.config.recent_projects = vec![RecentProject::fixture("Game", &project, 1)];
    session.selected_project_path = Some(project.clone());
    session.recycle_delete = |_| {
        Err(HubError::message(
            "Recycle Bin deletion failed with status 1",
        ))
    };

    session
        .apply_action(HubActionRequest {
            action_id: "request-delete".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .unwrap();
    session
        .apply_action(HubActionRequest {
            action_id: "confirm-delete".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .expect("delete failure should be recorded as recoverable state");

    assert_eq!(
        session.pending_delete_project_path.as_deref(),
        Some(project.as_path())
    );
    assert_eq!(session.config.recent_projects.len(), 1);
    assert_eq!(session.task_status.label, "Delete Project failed");
    assert_eq!(
        session.config.action_history[0].status,
        HubActionStatus::Failed
    );
    assert!(session.config.action_history[0].recovery.is_some());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn confirm_delete_success_with_injected_recycler_drops_project_only_from_hub() {
    let temp = temp_test_dir("zircon-hub-project-confirm-delete-success");
    let source = temp.join("ZirconEngine");
    fs::create_dir_all(&source).unwrap();
    let project = temp.join("Game");
    fs::create_dir_all(&project).unwrap();
    write_valid_project_manifest(&project, "Game");
    let mut session = session_with_source(&temp, &source);
    session.config.recent_projects = vec![RecentProject::fixture("Game", &project, 1)];
    session.selected_project_path = Some(project.clone());
    session.recycle_delete = |_| Ok(());

    session
        .apply_action(HubActionRequest {
            action_id: "request-delete".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .unwrap();
    session
        .apply_action(HubActionRequest {
            action_id: "confirm-delete".to_string(),
            target_id: Some(project.to_string_lossy().into_owned()),
            payload: None,
        })
        .expect("injected recycler success should remove the project from Hub");

    assert!(session.pending_delete_project_path.is_none());
    assert!(session.config.recent_projects.is_empty());
    assert!(session.selected_project_path.is_none());
    assert_eq!(session.task_status.label, "Project deleted");
    assert!(
        project.join("zircon-project.toml").is_file(),
        "injected recycler must not delete files during the unit test"
    );

    fs::remove_dir_all(temp).unwrap();
}

fn session_with_source(temp: &std::path::Path, source: &std::path::Path) -> HubRuntimeSession {
    let config_path = temp.join("hub.toml");
    let editor_config_path = temp.join("editor.json");
    let mut config = HubConfig::default();
    config.settings.default_project_dir = temp.join("projects");
    config.settings.default_source_dir = source.to_path_buf();
    config.settings.default_build_output_dir = temp.join("out");
    config.engines.push(SourceEngineInstall {
        id: source_engine_id(source),
        display_name: "Local Source".to_string(),
        source_dir: source.to_path_buf(),
        output_dir: temp.join("out"),
        last_build_unix_ms: None,
        build_history: Vec::new(),
    });
    config.active_engine_id = Some(source_engine_id(source));
    config.runtime.new_project_engine_id = Some(source_engine_id(source));
    config.save(&config_path).unwrap();
    fs::write(
        &editor_config_path,
        r#"{"editor.startup.session":{"recent_projects":[]}}"#,
    )
    .unwrap();
    HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
}

fn write_valid_project_manifest(root: &std::path::Path, name: &str) {
    let document = format!(
        "name = {name:?}\nformat_version = 2\ndefault_scene = \"res://scenes/main.scene.toml\"\nasset_roots = [\"assets\"]\nlibrary_version = 1\n"
    );
    zircon_runtime_interface::project::ProjectManifestSummary::parse_toml_str(&document)
        .expect("test project manifest must satisfy the shared summary contract");
    fs::write(root.join("zircon-project.toml"), document).unwrap();
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
