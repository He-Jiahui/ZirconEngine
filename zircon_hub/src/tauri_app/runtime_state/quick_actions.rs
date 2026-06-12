use std::path::PathBuf;

use crate::engines::{active_source_engine, SourceEngineInstall};
use crate::error::HubError;
use crate::state::{HubActionKind, HubActionRecord, HubMessage, TaskOperationKind, TaskStatus};

use super::HubRuntimeSession;

impl HubRuntimeSession {
    pub(super) fn record_action_and_persist(
        &mut self,
        record: HubActionRecord,
    ) -> Result<(), HubError> {
        self.config.action_history.insert(0, record);
        self.config
            .action_history
            .truncate(crate::state::ACTION_HISTORY_LIMIT);
        self.persist(None)
    }

    pub(super) fn set_action_failure_status(
        &mut self,
        action: HubActionKind,
        target: String,
        detail: HubMessage,
        recovery: HubMessage,
    ) {
        self.task_status =
            TaskStatus::error(format!("{} failed", action.label()), detail, recovery)
                .with_operation(action_operation_kind(action), target);
    }

    pub(super) fn action_target_for_project_failure(&self) -> String {
        self.selected_project_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Project".to_string())
    }

    pub(super) fn staged_engine_dir(&self) -> PathBuf {
        active_source_engine(
            &self.config.engines,
            self.config.active_engine_id.as_deref(),
        )
        .map(SourceEngineInstall::staged_engine_dir)
        .unwrap_or_else(|| {
            self.config
                .settings
                .default_build_output_dir
                .join("ZirconEngine")
        })
    }
}

fn action_operation_kind(action: HubActionKind) -> TaskOperationKind {
    match action {
        HubActionKind::BuildEditorRuntime => TaskOperationKind::Build,
        HubActionKind::OpenEditor | HubActionKind::OpenOutput => TaskOperationKind::Process,
        HubActionKind::OpenResource => TaskOperationKind::Hub,
        HubActionKind::CreateProject
        | HubActionKind::ImportProject
        | HubActionKind::RemoveProject
        | HubActionKind::DeleteProject
        | HubActionKind::PackageProject
        | HubActionKind::InstallProject => TaskOperationKind::Project,
    }
}

#[cfg(test)]
mod tests {
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
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_source_dir = PathBuf::new();
        config.settings.default_build_output_dir = temp.join("out");
        config.settings.default_device_install_dir = temp.join("device");
        config.recent_projects = vec![RecentProject::new(name, project, 1)];
        config.runtime.selected_project_path = Some(project.to_path_buf());
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
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
}
