use std::{fs, path::PathBuf};

use crate::{
    projects::RecentProject, settings::HubConfig, state::TaskOperationKind,
    tauri_app::HubActionRequest,
};

use super::super::HubRuntimeSession;

#[test]
fn explicit_project_target_updates_selected_project_without_overwriting_task_status() {
    let temp = temp_test_dir("zircon-hub-action-targets-explicit-project");
    let selected = create_project_root(&temp, "Selected");
    let target = create_project_root(&temp, "Target");
    let mut session = session_with_projects(
        &temp,
        &[("Selected", selected.clone()), ("Target", target.clone())],
        &selected,
    );
    session.task_status = crate::state::TaskStatus::running_operation(
        "Packaging",
        crate::state::HubMessage::new(crate::state::HubMessageId::Delivery(
            crate::state::DeliveryMessageId::CopyingProjectToPackage,
        )),
        TaskOperationKind::Project,
        "Target",
    );

    session
        .apply_request_project_target(&HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: Some(target.to_string_lossy().into_owned()),
            payload: None,
        })
        .expect("explicit target should resolve to a recent project");

    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(target.as_path())
    );
    assert_eq!(session.task_status.label, "Packaging");
    assert!(session.task_status.running);

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn project_target_payload_takes_precedence_over_target_id_for_background_actions() {
    let temp = temp_test_dir("zircon-hub-action-targets-payload-project");
    let fallback = create_project_root(&temp, "Fallback");
    let target = create_project_root(&temp, "Target");
    let mut session = session_with_projects(
        &temp,
        &[("Fallback", fallback.clone()), ("Target", target.clone())],
        &fallback,
    );

    session
        .apply_request_project_target(&HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: Some(fallback.to_string_lossy().into_owned()),
            payload: Some(serde_json::json!({
                "projectPath": target
            })),
        })
        .expect("typed project payload should select the payload project before target_id");

    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(target.as_path())
    );

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn project_target_payload_path_takes_precedence_over_project_id() {
    let temp = temp_test_dir("zircon-hub-action-targets-payload-path");
    let fallback = create_project_root(&temp, "Fallback");
    let target = create_project_root(&temp, "Target");
    let mut session = session_with_projects(
        &temp,
        &[("Fallback", fallback.clone()), ("Target", target.clone())],
        &fallback,
    );

    session
        .apply_request_project_target(&HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: None,
            payload: Some(serde_json::json!({
                "projectId": fallback,
                "projectPath": target
            })),
        })
        .expect("projectPath should resolve before projectId when both are present");

    assert_eq!(
        session.selected_project_path.as_deref(),
        Some(target.as_path())
    );

    fs::remove_dir_all(temp).unwrap();
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

fn create_project_root(temp: &std::path::Path, name: &str) -> PathBuf {
    let project = temp.join(name);
    fs::create_dir_all(project.join("Assets")).unwrap();
    fs::write(
        project.join("zircon-project.toml"),
        format!("name = \"{name}\"\n"),
    )
    .unwrap();
    project
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
