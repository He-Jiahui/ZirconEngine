use std::path::PathBuf;

use crate::build::{run_build_command, BuildCommand, BuildCommandOptions, BuildExecutionReport};
use crate::engines::{active_source_engine, validate_source_engine, SourceBuildRecord};
use crate::error::HubError;
use crate::projects::{metadata_for_path, RecentProject};
use crate::state::{
    EngineMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    ProjectMessageId, TaskExecutionOutcome, TaskOperationKind, TaskStatus,
};

use super::{
    action_tasks::{BackgroundTask, BackgroundTaskContext},
    recent_project_display_name, source_engine_validation_detail,
    source_engine_validation_recovery, HubRuntimeSession,
};

pub(super) mod staged_build;

#[derive(Debug)]
pub(in crate::tauri_app) struct PendingEditorRuntimeBuild {
    command: BuildCommand,
    command_line: Vec<String>,
    output_dir: PathBuf,
    engine_id: String,
    engine_target: String,
    profile: String,
    jobs: u16,
    source_dir: PathBuf,
    staged_engine_dir: PathBuf,
    active_staged_engine_dir: PathBuf,
    staging_output: StagingOutputLease,
}

impl PendingEditorRuntimeBuild {
    pub(in crate::tauri_app) fn command(&self) -> &BuildCommand {
        &self.command
    }

    #[cfg(test)]
    fn build_output_dir(&self) -> &std::path::Path {
        self.staging_output.path()
    }
}

#[derive(Debug)]
struct StagingOutputLease {
    path: PathBuf,
}

impl StagingOutputLease {
    fn new(path: PathBuf) -> Self {
        Self { path }
    }

    fn path(&self) -> &std::path::Path {
        &self.path
    }
}

impl Drop for StagingOutputLease {
    fn drop(&mut self) {
        let _ = staged_build::discard_build_output(&self.path);
    }
}

impl BackgroundTask for PendingEditorRuntimeBuild {
    type Output = BuildExecutionReport;

    fn run(
        &self,
        context: &BackgroundTaskContext,
    ) -> Result<TaskExecutionOutcome<BuildExecutionReport>, HubError> {
        run_build_command(self.command(), context.cancellation())
    }
}

impl HubRuntimeSession {
    pub(in crate::tauri_app) fn prepare_background_editor_runtime_build(
        &mut self,
    ) -> Result<Option<PendingEditorRuntimeBuild>, HubError> {
        let history_len = self.config.action_history.len();
        match self.prepare_editor_runtime_build() {
            Ok(pending_build) => Ok(Some(pending_build)),
            Err(error) if self.config.action_history.len() == history_len => Err(error),
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
        let project = match self.selected_or_latest_recent_project_with_engine_for_action() {
            Ok(project) => project,
            Err(error) => {
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
        };

        self.refresh_source_scoped_views()?;
        let engine = self.project_bound_engine(&project)?;
        let profile = self.config.settings.build_profile;
        let jobs = self.config.settings.jobs;
        let command = BuildCommand::for_editor_runtime(&BuildCommandOptions::new(
            self.config.settings.python_path.clone(),
            self.config.settings.cargo_path.clone(),
            engine.source_dir.clone(),
            engine.output_dir.clone(),
            profile,
            Some(jobs),
        ));
        let capture_dir = command.capture_dir.clone();
        self.validate_source_engine_for_build(&engine, command.command_line())?;
        let build_output_dir = staged_build::allocate_build_output_dir(&engine.output_dir)?;
        let mut command = BuildCommand::for_editor_runtime(&BuildCommandOptions::new(
            self.config.settings.python_path.clone(),
            self.config.settings.cargo_path.clone(),
            engine.source_dir.clone(),
            build_output_dir.clone(),
            profile,
            Some(jobs),
        ));
        command.capture_dir = capture_dir;
        let command_line = command.command_line();
        self.mark_background_action_prepared();
        let output_dir = engine.output_dir.clone();
        Ok(PendingEditorRuntimeBuild {
            command,
            command_line,
            output_dir,
            engine_id: engine.id.clone(),
            engine_target: engine.display_name.clone(),
            profile: profile.as_mode().to_string(),
            jobs,
            source_dir: engine.source_dir.clone(),
            staged_engine_dir: build_output_dir.join("ZirconEngine"),
            active_staged_engine_dir: engine.staged_engine_dir(),
            staging_output: StagingOutputLease::new(build_output_dir),
        })
    }

    fn complete_editor_runtime_build(
        &mut self,
        pending_build: PendingEditorRuntimeBuild,
        result: Result<BuildExecutionReport, HubError>,
    ) -> Result<(), HubError> {
        let build_output_dir = pending_build.staging_output.path().to_path_buf();
        let PendingEditorRuntimeBuild {
            command_line,
            output_dir,
            engine_id,
            engine_target,
            profile,
            jobs,
            source_dir,
            staged_engine_dir,
            active_staged_engine_dir,
            staging_output: _staging_output,
            ..
        } = pending_build;
        let report = match result {
            Ok(report) => report,
            Err(error) => {
                staged_build::discard_build_output(&build_output_dir)?;
                let detail = error.to_string();
                let detail_message = HubMessage::raw_text(detail.clone());
                let log_message = HubMessage::raw_text(detail);
                let recovery = HubMessage::new(HubMessageId::Engine(
                    EngineMessageId::CheckToolchainSettings,
                ));
                self.record_build_for_engine(
                    &engine_id,
                    &profile,
                    jobs,
                    &output_dir,
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
        if !report.process_exited_successfully() {
            staged_build::discard_build_output(&build_output_dir)?;
            let detail = report.summary_line();
            let detail_message = HubMessage::raw_text(detail);
            let log_message = HubMessage::raw_text(report.log_excerpt());
            let history_recovery = HubMessage::raw_text(report.recovery_hint());
            let status_recovery =
                HubMessage::new(HubMessageId::Engine(EngineMessageId::FixFirstBuildError));
            self.record_build_for_engine(
                &engine_id,
                &profile,
                jobs,
                &output_dir,
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
        let qualified_build = match staged_build::validate_staged_editor_runtime_build(
            &staged_engine_dir,
            &source_dir,
            &profile,
        ) {
            Ok(qualified_build) => qualified_build,
            Err(error) => {
                staged_build::discard_build_output(&build_output_dir)?;
                self.record_artifact_qualification_failure(
                    &engine_id,
                    &engine_target,
                    &profile,
                    jobs,
                    &output_dir,
                    command_line,
                    &report,
                    error,
                )?;
                return Ok(());
            }
        };
        let activation_warning = match staged_build::activate_staged_engine(
            &staged_engine_dir,
            &active_staged_engine_dir,
        ) {
            Ok(warning) => warning,
            Err(error) => {
                let error = match staged_build::discard_build_output(&build_output_dir) {
                    Ok(()) => error,
                    Err(cleanup_error) => HubError::message(format!(
                        "{error}; failed to clean up staging output: {cleanup_error}"
                    )),
                };
                self.record_artifact_qualification_failure(
                    &engine_id,
                    &engine_target,
                    &profile,
                    jobs,
                    &output_dir,
                    command_line,
                    &report,
                    error,
                )?;
                return Ok(());
            }
        };
        let cleanup_warning = staged_build::discard_build_output(&build_output_dir)
            .err()
            .map(|error| format!("staging cleanup pending: {error}"));
        let qualified_log =
            staged_build::qualified_build_log(&qualified_build, &report.log_excerpt());
        let qualified_log = [activation_warning, cleanup_warning]
            .into_iter()
            .flatten()
            .fold(qualified_log, |log, warning| format!("{log}\n{warning}"));
        self.record_build_for_engine(
            &engine_id,
            &profile,
            jobs,
            &output_dir,
            true,
            HubMessage::new(HubMessageId::Engine(
                EngineMessageId::StagedEditorRuntimePayload,
            )),
            HubMessage::raw_text(qualified_log.clone()),
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
            log_excerpt: HubMessage::raw_text(qualified_log),
            recovery: None,
            process_id: None,
            command_line,
            output_dir: Some(output_dir),
        })?;
        self.task_status = TaskStatus::success(
            "Build complete",
            HubMessage::raw_text(active_staged_engine_dir.to_string_lossy().into_owned()),
        )
        .with_operation(TaskOperationKind::Build, engine_target);
        Ok(())
    }

    fn validate_source_engine_for_build(
        &mut self,
        engine: &crate::engines::SourceEngineInstall,
        command_line: Vec<String>,
    ) -> Result<(), HubError> {
        let validation = validate_source_engine(&engine.source_dir);
        if validation == crate::engines::SourceEngineValidation::Valid {
            return Ok(());
        }
        let detail = source_engine_validation_detail(validation);
        let recovery = source_engine_validation_recovery(validation);
        let target = engine.display_name.clone();
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
            output_dir: Some(engine.output_dir.clone()),
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
        self.project_bound_engine(&project)?;
        Ok(project)
    }

    pub(super) fn project_bound_engine(
        &self,
        project: &RecentProject,
    ) -> Result<crate::engines::SourceEngineInstall, HubError> {
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
        self.config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
            .cloned()
            .ok_or_else(|| {
                HubError::status(
                    HubMessage::with_params(
                        HubMessageId::Project(ProjectMessageId::BoundSourceEngineUnavailable),
                        [format!(
                            "{} -> {}",
                            recent_project_display_name(project),
                            engine_id
                        )],
                    ),
                    None,
                )
            })
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

    #[allow(clippy::too_many_arguments)]
    fn record_artifact_qualification_failure(
        &mut self,
        engine_id: &str,
        engine_target: &str,
        profile: &str,
        jobs: u16,
        output_dir: &PathBuf,
        command_line: Vec<String>,
        report: &BuildExecutionReport,
        error: HubError,
    ) -> Result<(), HubError> {
        let detail = HubMessage::raw_text(error.to_string());
        let build_log = report.log_excerpt();
        let log_excerpt = HubMessage::raw_text(if build_log.is_empty() {
            detail.to_string()
        } else {
            format!("{build_log}\n{detail}")
        });
        let recovery = HubMessage::raw_text(
            "Rebuild the bound Source Engine and inspect the staged BuildSet manifest before retrying",
        );
        self.record_build_for_engine(
            engine_id,
            profile,
            jobs,
            output_dir,
            false,
            detail.clone(),
            log_excerpt.clone(),
            command_line.clone(),
        );
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::BuildEditorRuntime,
            status: HubActionStatus::Failed,
            target: engine_target.to_string(),
            detail: detail.clone(),
            log_excerpt,
            recovery: Some(recovery.clone()),
            process_id: None,
            command_line,
            output_dir: Some(output_dir.clone()),
        })?;
        self.task_status = TaskStatus::error("Build artifacts invalid", detail, recovery)
            .with_operation(TaskOperationKind::Build, engine_target);
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

    fn record_build_for_engine(
        &mut self,
        engine_id: &str,
        profile: &str,
        jobs: u16,
        output_dir: &PathBuf,
        success: bool,
        detail: HubMessage,
        log_excerpt: HubMessage,
        command_line: Vec<String>,
    ) {
        if let Some(engine) = self
            .config
            .engines
            .iter_mut()
            .find(|engine| engine.id == engine_id)
        {
            engine.record_build(SourceBuildRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                status: if success { "success" } else { "failed" }.to_string(),
                profile: profile.to_string(),
                jobs: Some(jobs),
                output_dir: output_dir.clone(),
                detail,
                log_excerpt,
                command_line,
            });
        }
    }
}

#[cfg(test)]
#[path = "tests/build_actions.rs"]
mod tests;
