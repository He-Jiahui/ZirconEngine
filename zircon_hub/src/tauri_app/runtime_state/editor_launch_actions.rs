use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::HubError;
use crate::process::{
    launch_editor, preferred_editor_executable, preferred_editor_executable_exists,
    EditorLaunchCommand, EditorLaunchRequest,
};
use crate::projects::{
    merge_recent_projects, project_paths_match, validate_project_root, ProjectValidation,
    RecentProject,
};
use crate::state::{
    HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId, ProcessMessageId,
    ProjectMessageId, TaskOperationKind, TaskStatus,
};

use super::{action_tasks::BackgroundTask, recent_project_display_name, HubRuntimeSession};

#[derive(Debug)]
pub(in crate::tauri_app) struct EditorLaunchReport {
    process_id: u32,
}

#[derive(Clone, Debug)]
pub(in crate::tauri_app) struct PendingEditorLaunch {
    target: String,
    command: EditorLaunchPreparedCommand,
    project_path: Option<PathBuf>,
    remember_project: bool,
    recovery_on_launch_failure: HubMessage,
}

#[derive(Clone, Debug)]
enum EditorLaunchPreparedCommand {
    Project(EditorLaunchCommand),
    Empty { executable: PathBuf },
}

impl BackgroundTask for PendingEditorLaunch {
    type Output = EditorLaunchReport;

    fn run(&self) -> Result<EditorLaunchReport, HubError> {
        let child = match &self.command {
            EditorLaunchPreparedCommand::Project(command) => launch_editor(command)?,
            EditorLaunchPreparedCommand::Empty { executable } => {
                Command::new(executable).spawn()?
            }
        };
        Ok(EditorLaunchReport {
            process_id: child.id(),
        })
    }
}

impl PendingEditorLaunch {
    fn command_line(&self) -> Vec<String> {
        match &self.command {
            EditorLaunchPreparedCommand::Project(command) => command.command_line(),
            EditorLaunchPreparedCommand::Empty { executable } => {
                vec![executable.to_string_lossy().into_owned()]
            }
        }
    }
}

impl HubRuntimeSession {
    pub(super) fn open_selected_project_or_editor(&mut self) -> Result<(), HubError> {
        let pending_launch = match self.prepare_editor_launch() {
            Ok(pending_launch) => pending_launch,
            Err(_) => return Ok(()),
        };
        let result = pending_launch.run();
        self.complete_editor_launch(pending_launch, result)
    }

    pub(in crate::tauri_app) fn prepare_background_editor_launch(
        &mut self,
    ) -> Result<Option<PendingEditorLaunch>, HubError> {
        match self.prepare_editor_launch() {
            Ok(pending_launch) => {
                self.mark_background_action_prepared();
                Ok(Some(pending_launch))
            }
            Err(error) if self.task_status.running => Err(error),
            Err(_) => Ok(None),
        }
    }

    pub(in crate::tauri_app) fn complete_background_editor_launch(
        &mut self,
        pending_launch: PendingEditorLaunch,
        result: Result<EditorLaunchReport, HubError>,
    ) -> Result<(), HubError> {
        self.complete_editor_launch(pending_launch, result)
    }

    fn prepare_editor_launch(&mut self) -> Result<PendingEditorLaunch, HubError> {
        let Some(project) = (match self.selected_or_latest_recent_project_for_action() {
            Ok(project) => project,
            Err(error) => {
                let (detail, _) = error.into_status_messages();
                self.record_editor_launch_failure(
                    self.action_target_for_project_failure(),
                    detail,
                    Vec::new(),
                    HubMessage::new(HubMessageId::Process(
                        ProcessMessageId::SelectProjectOrLaunchEmpty,
                    )),
                )?;
                return Err(HubError::status(
                    HubMessage::new(HubMessageId::Process(
                        ProcessMessageId::SelectProjectOrLaunchEmpty,
                    )),
                    None,
                ));
            }
        }) else {
            return self.prepare_empty_editor_launch();
        };
        self.prepare_project_editor_launch(project)
    }

    fn prepare_project_editor_launch(
        &mut self,
        project: RecentProject,
    ) -> Result<PendingEditorLaunch, HubError> {
        let project_path = project.path.clone();
        let display_name = recent_project_display_name(&project);
        if project_path.as_os_str().is_empty() {
            let detail =
                HubMessage::new(HubMessageId::Project(ProjectMessageId::ProjectPathRequired));
            let recovery = HubMessage::new(HubMessageId::Process(
                ProcessMessageId::ChooseValidProjectForEditor,
            ));
            self.record_editor_launch_failure(
                "Project".to_string(),
                detail.clone(),
                Vec::new(),
                recovery.clone(),
            )?;
            return Err(HubError::status(detail, Some(recovery)));
        }
        if validate_project_root(&project_path) != ProjectValidation::Valid {
            let detail = HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::RootInvalid),
                [project_path.to_string_lossy().into_owned()],
            );
            let recovery = HubMessage::new(HubMessageId::Project(
                ProjectMessageId::CheckProjectManifest,
            ));
            self.record_editor_launch_failure(
                display_name,
                detail.clone(),
                Vec::new(),
                recovery.clone(),
            )?;
            return Err(HubError::status(detail, Some(recovery)));
        }
        self.activate_project_engine_for_path(&project_path);
        if let Err(error) = self.ensure_editor_available() {
            let (detail, _) = error.into_status_messages();
            let recovery = HubMessage::new(HubMessageId::Process(
                ProcessMessageId::BuildPayloadBeforeOpeningProject,
            ));
            self.record_editor_launch_failure(display_name, detail, Vec::new(), recovery.clone())?;
            return Err(HubError::status(
                HubMessage::new(HubMessageId::Process(
                    ProcessMessageId::BuildPayloadBeforeOpeningProject,
                )),
                Some(recovery),
            ));
        }
        let command = EditorLaunchCommand::from_preferred_engine(
            self.staged_engine_dir(),
            EditorLaunchRequest::OpenProject {
                project_path: project_path.clone(),
            },
        );
        Ok(PendingEditorLaunch {
            target: recent_project_display_name(&project),
            command: EditorLaunchPreparedCommand::Project(command),
            project_path: Some(project_path),
            remember_project: true,
            recovery_on_launch_failure: HubMessage::new(HubMessageId::Process(
                ProcessMessageId::VerifyEditorAndProjectPath,
            )),
        })
    }

    fn prepare_empty_editor_launch(&mut self) -> Result<PendingEditorLaunch, HubError> {
        if let Err(error) = self.ensure_editor_available() {
            let (detail, _) = error.into_status_messages();
            let recovery = HubMessage::new(HubMessageId::Process(
                ProcessMessageId::BuildPayloadBeforeLaunching,
            ));
            self.record_editor_launch_failure(
                "Editor without project".to_string(),
                detail,
                Vec::new(),
                recovery.clone(),
            )?;
            return Err(HubError::status(
                HubMessage::new(HubMessageId::Process(
                    ProcessMessageId::BuildPayloadBeforeLaunching,
                )),
                Some(recovery),
            ));
        }
        Ok(PendingEditorLaunch {
            target: "Editor without project".to_string(),
            command: EditorLaunchPreparedCommand::Empty {
                executable: preferred_editor_executable(self.staged_engine_dir()),
            },
            project_path: None,
            remember_project: false,
            recovery_on_launch_failure: HubMessage::new(HubMessageId::Process(
                ProcessMessageId::VerifyEditorExecutable,
            )),
        })
    }

    fn complete_editor_launch(
        &mut self,
        pending_launch: PendingEditorLaunch,
        result: Result<EditorLaunchReport, HubError>,
    ) -> Result<(), HubError> {
        let command_line = pending_launch.command_line();
        let report = match result {
            Ok(report) => report,
            Err(error) => {
                let detail = HubMessage::raw_text(error.to_string());
                self.record_editor_launch_failure(
                    pending_launch.target,
                    detail,
                    command_line,
                    pending_launch.recovery_on_launch_failure,
                )?;
                return Ok(());
            }
        };

        if pending_launch.remember_project {
            let Some(project_path) = pending_launch.project_path else {
                return Err(HubError::message(
                    "Editor launch project state is missing the project path",
                ));
            };
            self.remember_project(RecentProject::with_now(
                pending_launch.target.clone(),
                project_path,
            ))?;
        }
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::OpenEditor,
            status: HubActionStatus::Success,
            target: pending_launch.target.clone(),
            detail: HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::StartedProcess),
                [report.process_id.to_string()],
            ),
            log_excerpt: HubMessage::empty(),
            recovery: None,
            process_id: Some(report.process_id),
            command_line,
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        let (operation, detail) = if pending_launch.remember_project {
            (
                TaskOperationKind::Project,
                HubMessage::with_params(
                    HubMessageId::Process(ProcessMessageId::OpeningTargetProcess),
                    [pending_launch.target.clone(), report.process_id.to_string()],
                ),
            )
        } else {
            (
                TaskOperationKind::Process,
                HubMessage::with_params(
                    HubMessageId::Process(ProcessMessageId::ProcessId),
                    [report.process_id.to_string()],
                ),
            )
        };
        self.task_status = TaskStatus::success("Editor launched", detail)
            .with_operation(operation, pending_launch.target);
        Ok(())
    }

    fn ensure_editor_available(&mut self) -> Result<(), HubError> {
        if preferred_editor_executable_exists(self.staged_engine_dir()) {
            return Ok(());
        }
        let executable = preferred_editor_executable(self.staged_engine_dir());
        Err(HubError::status(
            HubMessage::with_params(
                HubMessageId::Process(ProcessMessageId::EditorExecutableUnavailable),
                [executable.to_string_lossy().into_owned()],
            ),
            None,
        ))
    }

    fn selected_or_latest_recent_project(&mut self) -> Option<RecentProject> {
        let had_selected_project = self.selected_project_path.is_some();
        if let Some(project) = self.selected_recent_project() {
            return Some(project);
        }
        if had_selected_project {
            return None;
        }
        let project = self
            .config
            .recent_projects
            .iter()
            .max_by_key(|project| project.last_opened_unix_ms)
            .cloned();
        if let Some(project) = &project {
            self.selected_project_path = Some(project.path.clone());
        }
        project
    }

    fn selected_or_latest_recent_project_for_action(
        &mut self,
    ) -> Result<Option<RecentProject>, HubError> {
        let selected_before = self.selected_project_path.clone();
        let active_engine_before = self.config.active_engine_id.clone();
        let project = self.selected_or_latest_recent_project();
        if let Some(project) = &project {
            self.activate_project_engine_for_path(&project.path);
        }
        let selected_project_changed = selected_project_path_changed(
            selected_before.as_deref(),
            self.selected_project_path.as_deref(),
        );
        self.refresh_project_context_views(
            selected_project_changed,
            self.config.active_engine_id != active_engine_before,
        )?;
        Ok(project)
    }

    pub(super) fn selected_or_latest_recent_project_for_named_action(
        &mut self,
        missing_project_message: HubMessage,
        stale_project_message: HubMessage,
    ) -> Result<RecentProject, HubError> {
        let had_selected_project = self.selected_project_path.is_some();
        let Some(project) = self.selected_or_latest_recent_project_for_action()? else {
            return Err(HubError::status(
                if had_selected_project {
                    stale_project_message
                } else {
                    missing_project_message
                },
                None,
            ));
        };
        Ok(project)
    }

    fn remember_project(&mut self, project: RecentProject) -> Result<(), HubError> {
        let last_project_path = project.path.clone();
        let active_engine_before = self.config.active_engine_id.clone();
        self.selected_project_path = Some(last_project_path.clone());
        self.config.recent_projects = merge_recent_projects(
            std::iter::once(project),
            self.config.recent_projects.clone(),
        );
        self.activate_project_engine_for_path(&last_project_path);
        self.refresh_project_context_views(
            true,
            self.config.active_engine_id != active_engine_before,
        )?;
        self.persist(Some(&last_project_path))
    }

    fn record_editor_launch_failure(
        &mut self,
        target: String,
        detail: HubMessage,
        command_line: Vec<String>,
        recovery: HubMessage,
    ) -> Result<(), HubError> {
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::OpenEditor,
            status: HubActionStatus::Failed,
            target: target.clone(),
            detail: detail.clone(),
            log_excerpt: HubMessage::empty(),
            recovery: Some(recovery.clone()),
            process_id: None,
            command_line,
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        self.set_action_failure_status(HubActionKind::OpenEditor, target, detail, recovery);
        Ok(())
    }
}

fn selected_project_path_changed(before: Option<&Path>, after: Option<&Path>) -> bool {
    match (before, after) {
        (Some(before), Some(after)) => !project_paths_match(before, after),
        (None, None) => false,
        _ => true,
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::projects::RecentProject;
    use crate::settings::{HubConfig, HubLanguage};
    use crate::state::{
        HubActionKind, HubActionStatus, HubMessage, HubMessageId, ProcessMessageId,
    };

    use super::super::HubRuntimeSession;
    use super::{EditorLaunchReport, PendingEditorLaunch};

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
                super::preferred_editor_executable(session.staged_engine_dir()).to_string_lossy()
            )
        );
        assert_eq!(
            model.task_summary.recovery.as_deref(),
            Some("打开项目前先构建编辑器/运行时载荷，或修复源码引擎设置")
        );

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
            .complete_background_editor_launch(pending, Ok(EditorLaunchReport { process_id: 42 }))
            .expect("editor launch completion should record success");

        let record = &session.config.action_history[0];
        assert_eq!(record.action, HubActionKind::OpenEditor);
        assert_eq!(record.status, HubActionStatus::Success);
        assert_eq!(record.process_id, Some(42));
        assert_eq!(session.task_status.label, "Editor launched");

        fs::remove_dir_all(temp).unwrap();
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
            .complete_background_editor_launch(pending, Ok(EditorLaunchReport { process_id: 42 }))
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
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_source_dir = PathBuf::new();
        config.settings.default_build_output_dir = temp.join("out");
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
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).unwrap();
        path
    }
}
