use std::fs;
use std::path::PathBuf;

use crate::projects::RecentProject;
use crate::settings::HubConfig;
use crate::state::{HubActionKind, HubActionStatus};

use super::super::{HubActionRequest, HubRuntimeSession};

#[test]
fn package_action_creates_project_package_and_records_success_history() {
    let temp = temp_test_dir("zircon-hub-tauri-package-action");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);

    session
        .apply_action(HubActionRequest {
            action_id: "package-project".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("package action should return refreshed state");

    let record = &session.config.action_history[0];
    assert_eq!(record.action, HubActionKind::PackageProject);
    assert_eq!(record.status, HubActionStatus::Success);
    assert_eq!(record.target, "Game");
    assert_eq!(session.task_status.label, "Package created");
    let package_dir = record
        .output_dir
        .as_ref()
        .expect("package action should record package output dir");
    assert!(package_dir.join("zircon-package.toml").is_file());
    assert!(package_dir
        .join("project")
        .join("zircon-project.toml")
        .is_file());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn install_action_packages_project_then_copies_package_to_device_root() {
    let temp = temp_test_dir("zircon-hub-tauri-install-action");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);

    session
        .apply_action(HubActionRequest {
            action_id: "install-device".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("install action should return refreshed state");

    let install = &session.config.action_history[0];
    assert_eq!(install.action, HubActionKind::InstallProject);
    assert_eq!(install.status, HubActionStatus::Success);
    assert_eq!(
        session.config.action_history[1].action,
        HubActionKind::PackageProject
    );
    assert_eq!(session.task_status.label, "Installed to device");
    let install_dir = install
        .output_dir
        .as_ref()
        .expect("install action should record install dir");
    assert!(install_dir.join("zircon-package.toml").is_file());
    assert!(install_dir
        .join("project")
        .join("zircon-project.toml")
        .is_file());

    fs::remove_dir_all(temp).unwrap();
}

#[test]
fn open_editor_action_records_recoverable_failure_without_falling_back_to_demo_state() {
    let temp = temp_test_dir("zircon-hub-tauri-open-editor-missing");
    let project = create_project_root(&temp, "Game");
    let mut session = session_with_project(&temp, "Game", &project);

    let view_model = session
        .apply_action(HubActionRequest {
            action_id: "open-editor".to_string(),
            target_id: None,
            payload: None,
        })
        .expect("open editor action should return refreshed state even when launch fails");

    let record = &session.config.action_history[0];
    assert_eq!(record.action, HubActionKind::OpenEditor);
    assert_eq!(record.status, HubActionStatus::Failed);
    assert_eq!(session.task_status.label, "Open Editor failed");
    assert!(record.recovery.as_ref().unwrap().contains("editor/runtime"));
    assert_eq!(view_model.active_page, "projects");

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
    config.settings.default_source_dir = PathBuf::new();
    config.settings.default_build_output_dir = temp.join("out");
    config.settings.default_device_install_dir = temp.join("device");
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

fn create_project_root(temp: &std::path::Path, name: &str) -> PathBuf {
    let project = temp.join(name);
    fs::create_dir_all(project.join("Assets")).unwrap();
    fs::write(
        project.join("zircon-project.toml"),
        format!("name = \"{name}\"\n"),
    )
    .unwrap();
    fs::write(project.join("Assets").join("mesh.txt"), "mesh").unwrap();
    project
}

fn temp_test_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}-{}-{}",
        std::process::id(),
        crate::projects::now_unix_ms()
    ));
    fs::create_dir_all(&path).unwrap();
    path
}
