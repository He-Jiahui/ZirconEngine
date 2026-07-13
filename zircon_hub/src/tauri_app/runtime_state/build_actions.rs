use std::path::PathBuf;

use crate::build::{run_build_command, BuildCommand, BuildCommandOptions, BuildExecutionReport};
use crate::engines::{
    active_source_engine, active_source_engine_mut, validate_source_engine, SourceBuildRecord,
};
use crate::error::HubError;
use crate::projects::{metadata_for_path, RecentProject};
use crate::state::{
    EngineMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    ProjectMessageId, TaskOperationKind, TaskStatus,
};

use super::{
    action_tasks::BackgroundTask, recent_project_display_name, source_engine_validation_detail,
    source_engine_validation_recovery, HubRuntimeSession,
};

#[derive(Clone, Debug)]
pub(in crate::tauri_app) struct PendingEditorRuntimeBuild {
    command: BuildCommand,
    command_line: Vec<String>,
    output_dir: PathBuf,
    engine_target: String,
    staged_engine_dir: PathBuf,
}

impl PendingEditorRuntimeBuild {
    pub(in crate::tauri_app) fn command(&self) -> &BuildCommand {
        &self.command
    }
}

impl BackgroundTask for PendingEditorRuntimeBuild {
    type Output = BuildExecutionReport;

    fn run(&self) -> Result<BuildExecutionReport, HubError> {
        run_build_command(self.command())
    }
}

impl HubRuntimeSession {
    pub(super) fn build_selected_project_engine(&mut self) -> Result<(), HubError> {
        let pending_build = match self.prepare_editor_runtime_build() {
            Ok(pending_build) => pending_build,
            Err(_) => return Ok(()),
        };
        let result = pending_build.run();
        self.complete_editor_runtime_build(pending_build, result)
    }

    pub(in crate::tauri_app) fn prepare_background_editor_runtime_build(
        &mut self,
    ) -> Result<Option<PendingEditorRuntimeBuild>, HubError> {
        match self.prepare_editor_runtime_build() {
            Ok(pending_build) => Ok(Some(pending_build)),
            Err(error) if self.task_status.running => Err(error),
            Err(_) => Ok(None),
        }
    }

    pub(in crate::tauri_app) fn complete_background_editor_runtime_build(
        &mut self,
        pending_build: PendingEditorRuntimeBuild,
        result: Result<BuildExecutionReport, HubError>,
    ) -> Result<(), HubError> {
        self.complete_editor_runtime_build(pending_build, result)
    }

    fn prepare_editor_runtime_build(&mut self) -> Result<PendingEditorRuntimeBuild, HubError> {
        if let Err(error) = self.selected_or_latest_recent_project_with_engine_for_action() {
            let (detail, _) = error.into_status_messages();
            let recovery = HubMessage::new(HubMessageId::Engine(
                EngineMessageId::SelectValidProjectWithEngine,
            ));
            self.record_build_action_failure(
                self.action_target_for_project_failure(),
                detail.clone(),
                Vec::new(),
                Some(self.config.settings.default_build_output_dir.clone()),
                recovery.clone(),
            )?;
            return Err(HubError::status(detail, Some(recovery)));
        }

        let _ = self.register_source_engine_from_settings();
        self.refresh_source_scoped_views()?;
        let command = BuildCommand::for_editor_runtime(&BuildCommandOptions::new(
            self.config.settings.python_path.clone(),
            self.config.settings.cargo_path.clone(),
            self.config.settings.default_source_dir.clone(),
            self.config.settings.default_build_output_dir.clone(),
            self.config.settings.build_profile,
            Some(self.config.settings.jobs),
        ));
        let command_line = command.command_line();
        self.validate_active_source_engine_for_build(command_line.clone())?;
        let task_id = self.task_status.task_id;
        self.task_status = TaskStatus::running_operation(
            "Building",
            HubMessage::new(HubMessageId::Engine(EngineMessageId::RunningBuildScript)),
            TaskOperationKind::Build,
            self.action_engine_target(),
        )
        .with_task_id(task_id);
        self.mark_background_action_prepared();
        let output_dir = self.config.settings.default_build_output_dir.clone();
        Ok(PendingEditorRuntimeBuild {
            command,
            command_line,
            output_dir,
            engine_target: self.action_engine_target(),
            staged_engine_dir: self.staged_engine_dir(),
        })
    }

    fn complete_editor_runtime_build(
        &mut self,
        pending_build: PendingEditorRuntimeBuild,
        result: Result<BuildExecutionReport, HubError>,
    ) -> Result<(), HubError> {
        let PendingEditorRuntimeBuild {
            command_line,
            output_dir,
            engine_target,
            staged_engine_dir,
            ..
        } = pending_build;
        let report = match result {
            Ok(report) => report,
            Err(error) => {
                let detail = error.to_string();
                let detail_message = HubMessage::raw_text(detail.clone());
                let log_message = HubMessage::raw_text(detail);
                let recovery = HubMessage::new(HubMessageId::Engine(
                    EngineMessageId::CheckToolchainSettings,
                ));
                self.record_active_build(
                    false,
                    detail_message.clone(),
                    log_message.clone(),
                    command_line.clone(),
                );
                self.record_action_and_persist(HubActionRecord {
                    finished_unix_ms: crate::projects::now_unix_ms(),
                    action: HubActionKind::BuildEditorRuntime,
                    status: HubActionStatus::Failed,
                    target: engine_target.clone(),
                    detail: detail_message.clone(),
                    log_excerpt: log_message,
                    recovery: Some(recovery.clone()),
                    process_id: None,
                    command_line,
                    output_dir: Some(output_dir),
                })?;
                self.task_status = TaskStatus::error("Build failed", detail_message, recovery)
                    .with_operation(TaskOperationKind::Build, engine_target);
                return Ok(());
            }
        };
        if !report.succeeded() {
            let detail = report.summary_line();
            let detail_message = HubMessage::raw_text(detail);
            let log_message = HubMessage::raw_text(report.log_excerpt());
            let history_recovery = HubMessage::raw_text(report.recovery_hint());
            let status_recovery =
                HubMessage::new(HubMessageId::Engine(EngineMessageId::FixFirstBuildError));
            self.record_active_build(
                false,
                detail_message.clone(),
                log_message.clone(),
                command_line.clone(),
            );
            self.record_action_and_persist(HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action: HubActionKind::BuildEditorRuntime,
                status: HubActionStatus::Failed,
                target: engine_target.clone(),
                detail: detail_message.clone(),
                log_excerpt: log_message,
                recovery: Some(history_recovery),
                process_id: None,
                command_line,
                output_dir: Some(output_dir),
            })?;
            self.task_status = TaskStatus::error("Build failed", detail_message, status_recovery)
                .with_operation(TaskOperationKind::Build, engine_target);
            return Ok(());
        }
        self.record_active_build(
            true,
            HubMessage::new(HubMessageId::Engine(
                EngineMessageId::StagedEditorRuntimePayload,
            )),
            HubMessage::raw_text(report.log_excerpt()),
            command_line.clone(),
        );
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::BuildEditorRuntime,
            status: HubActionStatus::Success,
            target: engine_target.clone(),
            detail: HubMessage::new(HubMessageId::Engine(
                EngineMessageId::StagedEditorRuntimePayload,
            )),
            log_excerpt: HubMessage::raw_text(report.log_excerpt()),
            recovery: None,
            process_id: None,
            command_line,
            output_dir: Some(output_dir),
        })?;
        self.task_status = TaskStatus::success(
            "Build complete",
            HubMessage::raw_text(staged_engine_dir.to_string_lossy().into_owned()),
        )
        .with_operation(TaskOperationKind::Build, engine_target);
        Ok(())
    }

    fn validate_active_source_engine_for_build(
        &mut self,
        command_line: Vec<String>,
    ) -> Result<(), HubError> {
        let validation = validate_source_engine(&self.config.settings.default_source_dir);
        if validation == crate::engines::SourceEngineValidation::Valid {
            return Ok(());
        }
        let detail = source_engine_validation_detail(validation);
        let recovery = source_engine_validation_recovery(validation);
        let target = self.action_engine_target();
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::BuildEditorRuntime,
            status: HubActionStatus::Failed,
            target: target.clone(),
            detail: detail.clone(),
            log_excerpt: detail.clone(),
            recovery: Some(recovery.clone()),
            process_id: None,
            command_line,
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        self.task_status = TaskStatus::error("Source Engine invalid", detail, recovery)
            .with_operation(TaskOperationKind::SourceEngine, target);
        Err(HubError::status(
            self.task_status.detail.clone(),
            self.task_status.recovery.clone(),
        ))
    }

    fn selected_or_latest_recent_project_with_engine_for_action(
        &mut self,
    ) -> Result<RecentProject, HubError> {
        let project = self.selected_or_latest_recent_project_for_named_action(
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::NoRecentProjectToBuild,
            )),
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::SelectedProjectStaleForBuild,
            )),
        )?;
        self.require_project_bound_engine(&project)?;
        Ok(project)
    }

    fn require_project_bound_engine(&self, project: &RecentProject) -> Result<(), HubError> {
        let Some(engine_id) = metadata_for_path(&self.config.project_metadata, &project.path)
            .and_then(|metadata| metadata.engine_id.as_deref())
        else {
            return Err(HubError::status(
                HubMessage::with_params(
                    HubMessageId::Project(ProjectMessageId::NoBoundSourceEngine),
                    [recent_project_display_name(project)],
                ),
                None,
            ));
        };
        if self
            .config
            .engines
            .iter()
            .any(|engine| engine.id == engine_id)
        {
            return Ok(());
        }
        Err(HubError::status(
            HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::BoundSourceEngineUnavailable),
                [format!(
                    "{} -> {}",
                    recent_project_display_name(project),
                    engine_id
                )],
            ),
            None,
        ))
    }

    fn record_build_action_failure(
        &mut self,
        target: String,
        detail: HubMessage,
        command_line: Vec<String>,
        output_dir: Option<PathBuf>,
        recovery: HubMessage,
    ) -> Result<(), HubError> {
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::BuildEditorRuntime,
            status: HubActionStatus::Failed,
            target: target.clone(),
            detail: detail.clone(),
            log_excerpt: detail.clone(),
            recovery: Some(recovery.clone()),
            process_id: None,
            command_line,
            output_dir,
        })?;
        self.task_status = TaskStatus::error("Build editor/runtime failed", detail, recovery)
            .with_operation(TaskOperationKind::Build, target);
        Ok(())
    }

    fn action_engine_target(&self) -> String {
        active_source_engine(
            &self.config.engines,
            self.config.active_engine_id.as_deref(),
        )
        .map(|engine| engine.display_name.clone())
        .unwrap_or_else(|| {
            self.config
                .settings
                .default_source_dir
                .to_string_lossy()
                .into_owned()
        })
    }

    fn record_active_build(
        &mut self,
        success: bool,
        detail: HubMessage,
        log_excerpt: HubMessage,
        command_line: Vec<String>,
    ) {
        if let Some(engine) = active_source_engine_mut(
            &mut self.config.engines,
            self.config.active_engine_id.as_deref(),
        ) {
            engine.record_build(SourceBuildRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                status: if success { "success" } else { "failed" }.to_string(),
                profile: self.config.settings.build_profile.as_mode().to_string(),
                jobs: Some(self.config.settings.jobs),
                output_dir: self.config.settings.default_build_output_dir.clone(),
                detail,
                log_excerpt,
                command_line,
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use crate::build::BuildExecutionReport;
    use crate::engines::source_engine_id;
    use crate::projects::{project_metadata_key, ProjectMetadata, RecentProject};
    use crate::settings::{HubConfig, HubLanguage};
    use crate::state::{
        HubActionKind, HubActionStatus, HubMessage, HubMessageId, ProjectMessageId,
    };

    use super::super::HubRuntimeSession;

    #[test]
    fn background_build_prepares_command_without_running_or_recording_history() {
        let temp = temp_test_dir("zircon-hub-background-build-prepare");
        let source = create_source_engine_root(&temp);
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project_and_engine(&temp, "Game", &project, &source);

        let pending = session
            .prepare_background_editor_runtime_build()
            .expect("background build preparation should not fail hard")
            .expect("valid project and engine should produce a pending build");

        assert!(session.task_status.running);
        assert_eq!(session.task_status.label, "Building");
        assert!(pending
            .command()
            .command_line()
            .iter()
            .any(|part| part.contains("zircon_build.py")));
        assert_eq!(session.config.action_history.len(), 0);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn background_build_completion_records_success_after_external_result() {
        let temp = temp_test_dir("zircon-hub-background-build-complete");
        let source = create_source_engine_root(&temp);
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project_and_engine(&temp, "Game", &project, &source);
        let pending = session
            .prepare_background_editor_runtime_build()
            .unwrap()
            .unwrap();

        session
            .complete_background_editor_runtime_build(
                pending,
                Ok(BuildExecutionReport {
                    status_code: Some(0),
                    stdout: "staged editor/runtime\n".to_string(),
                    stderr: String::new(),
                }),
            )
            .expect("successful external result should complete build state");

        let record = &session.config.action_history[0];
        assert_eq!(record.action, HubActionKind::BuildEditorRuntime);
        assert_eq!(record.status, HubActionStatus::Success);
        assert_eq!(session.task_status.label, "Build complete");
        let active_engine = session
            .config
            .engines
            .iter()
            .find(|engine| session.config.active_engine_id.as_deref() == Some(engine.id.as_str()))
            .expect("build should keep active Source Engine");
        assert_eq!(active_engine.build_history.len(), 1);
        assert_eq!(active_engine.build_history[0].status, "success");

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn build_completion_localizes_success_history_detail() {
        let temp = temp_test_dir("zircon-hub-background-build-complete-localized");
        let source = create_source_engine_root(&temp);
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project_and_engine(&temp, "Game", &project, &source);
        session.config.settings.language = HubLanguage::Chinese;
        let pending = session
            .prepare_background_editor_runtime_build()
            .unwrap()
            .unwrap();

        session
            .complete_background_editor_runtime_build(
                pending,
                Ok(BuildExecutionReport {
                    status_code: Some(0),
                    stdout: "staged editor/runtime\n".to_string(),
                    stderr: String::new(),
                }),
            )
            .expect("successful external result should complete localized build state");

        let model = session.view_model();
        assert_eq!(model.task_summary.label, "构建完成");
        assert_eq!(model.action_history[0].action, "构建编辑器/运行时");
        assert_eq!(model.action_history[0].detail, "已暂存编辑器/运行时载荷");

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn build_precondition_failure_localizes_unbound_source_engine_detail() {
        let temp = temp_test_dir("zircon-hub-build-unbound-engine-localized");
        let project = create_project_root(&temp, "Game");
        let mut session = session_with_project_without_engine(&temp, "Game", &project);
        session.config.settings.language = HubLanguage::Chinese;

        let pending = session
            .prepare_background_editor_runtime_build()
            .expect("missing project engine binding should be a recoverable task status");

        assert!(pending.is_none());
        assert_eq!(
            session.config.action_history[0].detail,
            HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::NoBoundSourceEngine),
                ["Game"]
            )
        );
        let model = session.view_model();
        assert_eq!(model.task_summary.label, "构建编辑器/运行时失败");
        assert_eq!(model.task_summary.detail, "项目未绑定源码引擎：Game");
        assert_eq!(model.action_history[0].detail, "项目未绑定源码引擎：Game");
        assert_eq!(
            model.task_summary.recovery.as_deref(),
            Some("构建前先选择一个已绑定源码引擎的有效项目")
        );

        fs::remove_dir_all(temp).unwrap();
    }

    fn session_with_project_and_engine(
        temp: &std::path::Path,
        name: &str,
        project: &std::path::Path,
        source: &std::path::Path,
    ) -> HubRuntimeSession {
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let engine_id = source_engine_id(source);
        let mut config = HubConfig::default();
        config.settings.default_source_dir = source.to_path_buf();
        config.settings.default_build_output_dir = temp.join("out");
        config.recent_projects = vec![RecentProject::fixture(name, project, 1)];
        config.runtime.selected_project_path = Some(project.to_path_buf());
        config.project_metadata.insert(
            project_metadata_key(project),
            ProjectMetadata {
                engine_id: Some(engine_id),
                ..ProjectMetadata::default()
            },
        );
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
    }

    fn session_with_project_without_engine(
        temp: &std::path::Path,
        name: &str,
        project: &std::path::Path,
    ) -> HubRuntimeSession {
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.default_build_output_dir = temp.join("out");
        config.recent_projects = vec![RecentProject::fixture(name, project, 1)];
        config.runtime.selected_project_path = Some(project.to_path_buf());
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        HubRuntimeSession::load_from_paths(config_path, editor_config_path).unwrap()
    }

    fn create_source_engine_root(temp: &std::path::Path) -> PathBuf {
        let source = temp.join("ZirconEngine");
        fs::create_dir_all(source.join("tools")).unwrap();
        fs::create_dir_all(source.join("zircon_runtime")).unwrap();
        fs::write(
            source.join("Cargo.toml"),
            "[workspace]\nmembers = [\"zircon_runtime\"]\n",
        )
        .unwrap();
        fs::write(source.join("tools").join("zircon_build.py"), "").unwrap();
        source
    }

    fn create_project_root(temp: &std::path::Path, name: &str) -> PathBuf {
        let project = temp.join(name);
        fs::create_dir_all(&project).unwrap();
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
}
