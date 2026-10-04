use std::path::{Path, PathBuf};
use std::process::Command;

use zircon_runtime_interface::hub_protocol::{
    HubEditorLaunchOutcomeV1, HubEditorReadyReceiptV1, HubEditorStartupFailureCodeV1,
    HubSessionToken,
};

use crate::engines::{
    active_source_engine, validate_source_engine, SourceEngineInstall, SourceEngineValidation,
};
use crate::error::HubError;
use crate::process::editor_focus::{
    probe_project_editor_session, publish_project_editor_focus_signal,
    wait_for_project_editor_focus_ack, ProjectEditorSessionProbe,
};
use crate::process::{
    editor_handshake::wait_for_editor_handshake, launch_editor, staged_editor_executable,
    staged_editor_executable_exists, EditorLaunchCommand, EditorLaunchRequest, SupervisedChild,
};
use crate::projects::{
    enabled_project_template_id, merge_recent_project_entries, project_paths_match,
    validate_project_root, CreateProjectRequest, ProjectTemplateId, ProjectValidation,
    RecentProject,
};
use crate::state::{
    HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId, ProcessMessageId,
    ProjectMessageId, TaskExecutionOutcome, TaskOperationKind, TaskStatus,
};

use super::{
    action_tasks::{BackgroundTask, BackgroundTaskContext},
    recent_project_display_name, source_engine_validation_detail,
    source_engine_validation_recovery, HubRuntimeSession,
};
use crate::tauri_app::action_request::{HubAction, HubActionRequest};

#[derive(Debug)]
pub(in crate::tauri_app) struct EditorLaunchReport {
    attempt_id: u64,
    process_id: u32,
    outcome: EditorLaunchOutcome,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum EditorLaunchOutcome {
    Spawned,
    FocusedExisting,
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
pub(in crate::tauri_app) struct PendingProjectCreation {
    project_name: String,
    project_root: PathBuf,
    engine_id: String,
    template: ProjectTemplateId,
    command: EditorLaunchCommand,
    handshake_session: HubSessionToken,
}

#[derive(Clone, Debug)]
enum EditorLaunchPreparedCommand {
    FocusExistingProject {
        project_path: PathBuf,
        record: zircon_runtime_interface::project::session_lock::ProjectSessionAdmissionRecordV1,
        focus_session: HubSessionToken,
    },
    Project {
        command: EditorLaunchCommand,
        handshake_session: HubSessionToken,
    },
    Empty {
        executable: PathBuf,
    },
}

impl BackgroundTask for PendingEditorLaunch {
    type Output = EditorLaunchReport;

    fn run(
        &self,
        context: &BackgroundTaskContext,
    ) -> Result<TaskExecutionOutcome<EditorLaunchReport>, HubError> {
        if context.cancellation().is_cancellation_requested() {
            return Ok(TaskExecutionOutcome::Cancelled);
        }
        let outcome = match &self.command {
            EditorLaunchPreparedCommand::FocusExistingProject {
                project_path,
                record,
                focus_session,
            } => {
                let request =
                    publish_project_editor_focus_signal(project_path, record, *focus_session)?;
                match wait_for_project_editor_focus_ack(
                    project_path,
                    &request,
                    context.cancellation(),
                )? {
                    TaskExecutionOutcome::Completed(()) => {
                        TaskExecutionOutcome::Completed(EditorLaunchReport {
                            attempt_id: context.cancellation().task_id(),
                            process_id: record.process_id(),
                            outcome: EditorLaunchOutcome::FocusedExisting,
                        })
                    }
                    TaskExecutionOutcome::Cancelled => TaskExecutionOutcome::Cancelled,
                }
            }
            EditorLaunchPreparedCommand::Project {
                command,
                handshake_session,
            } => {
                let child = launch_editor(command)?;
                let Some(project_path) = self.project_path.as_deref() else {
                    let error = HubError::message(
                        "project editor launch is missing its handshake project path",
                    );
                    return match context
                        .editor_child_reaper()
                        .fail_before_ready(context.cancellation().task_id(), child)
                    {
                        Ok(()) => Err(error),
                        Err(cleanup_error) => Err(HubError::message(format!(
                            "{error}; failed to terminate Editor process tree: {cleanup_error}"
                        ))),
                    };
                };
                match wait_for_project_editor_ready(
                    project_path,
                    *handshake_session,
                    child,
                    context,
                )? {
                    TaskExecutionOutcome::Completed(process_id) => {
                        TaskExecutionOutcome::Completed(EditorLaunchReport {
                            attempt_id: context.cancellation().task_id(),
                            process_id,
                            outcome: EditorLaunchOutcome::Spawned,
                        })
                    }
                    TaskExecutionOutcome::Cancelled => TaskExecutionOutcome::Cancelled,
                }
            }
            EditorLaunchPreparedCommand::Empty { executable } => {
                let mut process = Command::new(executable);
                let mut child = SupervisedChild::spawn(&mut process, "Editor")?;
                if context.cancellation().is_cancellation_requested() {
                    context
                        .editor_child_reaper()
                        .cancel_before_ready(context.cancellation().task_id(), child)?;
                    TaskExecutionOutcome::Cancelled
                } else {
                    let process_id = child.id();
                    context
                        .editor_child_reaper()
                        .register(context.cancellation().task_id(), child)?;
                    TaskExecutionOutcome::Completed(EditorLaunchReport {
                        attempt_id: context.cancellation().task_id(),
                        process_id,
                        outcome: EditorLaunchOutcome::Spawned,
                    })
                }
            }
        };
        Ok(outcome)
    }
}

impl BackgroundTask for PendingProjectCreation {
    type Output = EditorLaunchReport;

    fn run(
        &self,
        context: &BackgroundTaskContext,
    ) -> Result<TaskExecutionOutcome<EditorLaunchReport>, HubError> {
        if context.cancellation().is_cancellation_requested() {
            return Ok(TaskExecutionOutcome::Cancelled);
        }
        let child = launch_editor(&self.command)?;
        match wait_for_project_editor_ready(
            &self.project_root,
            self.handshake_session,
            child,
            context,
        )? {
            TaskExecutionOutcome::Completed(process_id) => {
                Ok(TaskExecutionOutcome::Completed(EditorLaunchReport {
                    attempt_id: context.cancellation().task_id(),
                    process_id,
                    outcome: EditorLaunchOutcome::Spawned,
                }))
            }
            TaskExecutionOutcome::Cancelled => Ok(TaskExecutionOutcome::Cancelled),
        }
    }
}

impl PendingEditorLaunch {
    fn command_line(&self) -> Vec<String> {
        match &self.command {
            EditorLaunchPreparedCommand::FocusExistingProject { .. } => Vec::new(),
            EditorLaunchPreparedCommand::Project { command, .. } => command.command_line(),
            EditorLaunchPreparedCommand::Empty { executable } => {
                vec![executable.to_string_lossy().into_owned()]
            }
        }
    }
}

impl HubRuntimeSession {
    pub(in crate::tauri_app) fn prepare_background_project_creation(
        &mut self,
        request: &HubActionRequest,
    ) -> Result<Option<PendingProjectCreation>, HubError> {
        let HubAction::CreateProject { payload } = request.parse_as(request.action()?)? else {
            return Err(HubError::message(
                "project creation background task received a non-create action",
            ));
        };
        self.remember_create_project_payload(&payload);

        let template = match enabled_project_template_id(&payload.template) {
            Some(template) => template,
            None => {
                self.record_lifecycle_failure(
                    HubActionKind::CreateProject,
                    payload.name,
                    HubMessage::with_params(
                        HubMessageId::Project(ProjectMessageId::TemplateComingSoon),
                        [payload.template],
                    ),
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::ChooseRenderableTemplate,
                    )),
                    None,
                )?;
                return Ok(None);
            }
        };
        let engine_id = match self.resolve_project_engine_id(payload.engine_id) {
            Ok(Some(engine_id)) => engine_id,
            Ok(None) => {
                let target = payload.name;
                self.record_lifecycle_failure(
                    HubActionKind::CreateProject,
                    target.clone(),
                    HubMessage::with_params(
                        HubMessageId::Project(ProjectMessageId::NoBoundSourceEngine),
                        [target],
                    ),
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::RegisterEngineBeforeCreate,
                    )),
                    None,
                )?;
                return Ok(None);
            }
            Err(error) => {
                let (detail, _) = error.into_status_messages();
                self.record_lifecycle_failure(
                    HubActionKind::CreateProject,
                    payload.name,
                    detail,
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::RegisterEngineBeforeCreate,
                    )),
                    None,
                )?;
                return Ok(None);
            }
        };
        let engine = self
            .config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
            .cloned()
            .expect("resolved project engine id must remain registered");
        let source_validation = validate_source_engine(&engine.source_dir);
        let staged_engine_dir = engine.staged_engine_dir();
        if source_validation != SourceEngineValidation::Valid {
            self.record_lifecycle_failure(
                HubActionKind::CreateProject,
                payload.name,
                source_engine_validation_detail(source_validation),
                source_engine_validation_recovery(source_validation),
                None,
            )?;
            return Ok(None);
        }
        let build_profile = self.config.settings.build_profile.as_mode().to_string();
        if let Err(error) = self.ensure_editor_available_at(&engine, &build_profile) {
            let (detail, _) = error.into_status_messages();
            self.record_lifecycle_failure(
                HubActionKind::CreateProject,
                payload.name,
                detail,
                HubMessage::new(HubMessageId::Process(
                    ProcessMessageId::BuildPayloadBeforeOpeningProject,
                )),
                None,
            )?;
            return Ok(None);
        }

        let create_request =
            CreateProjectRequest::new(payload.name.clone(), payload.location, template);
        let project_root = create_request.target_root();
        let handshake_session = HubSessionToken::new();
        let command = EditorLaunchCommand::from_staged_engine(
            staged_engine_dir,
            EditorLaunchRequest::create_project(create_request)?,
        )?
        .with_hub_handshake(handshake_session);
        self.mark_background_action_prepared();
        Ok(Some(PendingProjectCreation {
            project_name: payload.name,
            project_root,
            engine_id,
            template,
            command,
            handshake_session,
        }))
    }

    pub(in crate::tauri_app) fn complete_background_project_creation(
        &mut self,
        pending: PendingProjectCreation,
        result: Result<EditorLaunchReport, HubError>,
    ) -> Result<(), HubError> {
        let command_line = pending.command.command_line();
        let report = match result {
            Ok(report) => report,
            Err(error) => {
                let recovery = if validate_project_root(&pending.project_root)
                    == ProjectValidation::Valid
                {
                    HubMessage::new(HubMessageId::Project(ProjectMessageId::KeptFolderUseImport))
                } else {
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::ChooseEmptyTargetFolder,
                    ))
                };
                return self.record_lifecycle_failure(
                    HubActionKind::CreateProject,
                    pending.project_name,
                    HubMessage::raw_text(error.to_string()),
                    recovery,
                    pending
                        .project_root
                        .exists()
                        .then_some(pending.project_root),
                );
            }
        };

        let attempt_id = report.attempt_id;
        let process_id = report.process_id;
        let target = pending.project_name.clone();
        let completion = self.accept_editor_owned_project_creation(
            pending.project_name,
            pending.project_root,
            pending.engine_id,
            pending.template,
            process_id,
            command_line,
        );
        self.task_status.task_id = attempt_id;
        let terminal = self.observe_completed_editor_launch(attempt_id, process_id, target);
        completion.and(terminal)
    }

    pub(in crate::tauri_app) fn prepare_background_editor_launch(
        &mut self,
    ) -> Result<Option<PendingEditorLaunch>, HubError> {
        let history_len = self.config.action_history.len();
        match self.prepare_editor_launch() {
            Ok(pending_launch) => {
                self.mark_background_action_prepared();
                Ok(Some(pending_launch))
            }
            Err(error) if self.config.action_history.len() == history_len => Err(error),
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
        let active_session = match probe_project_editor_session(&project_path) {
            Ok(active_session) => active_session,
            Err(error) => {
                let detail = HubMessage::raw_text(error.to_string());
                let recovery = HubMessage::new(HubMessageId::Process(
                    ProcessMessageId::VerifyEditorAndProjectPath,
                ));
                self.record_editor_launch_failure(
                    display_name,
                    detail.clone(),
                    Vec::new(),
                    recovery.clone(),
                )?;
                return Err(HubError::status(detail, Some(recovery)));
            }
        };
        match active_session {
            ProjectEditorSessionProbe::Ready(record) => {
                return Ok(PendingEditorLaunch {
                    target: recent_project_display_name(&project),
                    command: EditorLaunchPreparedCommand::FocusExistingProject {
                        project_path: project_path.clone(),
                        record,
                        focus_session: HubSessionToken::new(),
                    },
                    project_path: Some(project_path),
                    remember_project: true,
                    recovery_on_launch_failure: HubMessage::new(HubMessageId::Process(
                        ProcessMessageId::VerifyEditorAndProjectPath,
                    )),
                });
            }
            ProjectEditorSessionProbe::Pending(record) => {
                return Err(HubError::message(format!(
                    "project editor instance {} is still completing admission ({})",
                    record.process_id(),
                    record.lifecycle().as_str(),
                )));
            }
            ProjectEditorSessionProbe::RecoveryRequired(record) => {
                return Err(HubError::message(format!(
                    "project editor instance {} requires recovery before another editor can open the project",
                    record.process_id(),
                )));
            }
            ProjectEditorSessionProbe::Inactive => {}
        }
        self.activate_project_engine_for_path(&project_path);
        let engine = match self.project_bound_engine(&project) {
            Ok(engine) => engine,
            Err(error) => {
                let (detail, _) = error.into_status_messages();
                let recovery = HubMessage::new(HubMessageId::Process(
                    ProcessMessageId::ChooseValidProjectForEditor,
                ));
                self.record_editor_launch_failure(
                    display_name,
                    detail.clone(),
                    Vec::new(),
                    recovery.clone(),
                )?;
                return Err(HubError::status(detail, Some(recovery)));
            }
        };
        self.validate_source_engine_for_editor_launch(&engine, &display_name)?;
        let build_profile = self.config.settings.build_profile.as_mode().to_string();
        if let Err(error) = self.ensure_editor_available_at(&engine, &build_profile) {
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
        let handshake_session = HubSessionToken::new();
        let command = EditorLaunchCommand::from_staged_engine(
            engine.staged_engine_dir(),
            EditorLaunchRequest::open_project(project_path.clone())?,
        )?
        .with_hub_handshake(handshake_session);
        Ok(PendingEditorLaunch {
            target: recent_project_display_name(&project),
            command: EditorLaunchPreparedCommand::Project {
                command,
                handshake_session,
            },
            project_path: Some(project_path),
            remember_project: true,
            recovery_on_launch_failure: HubMessage::new(HubMessageId::Process(
                ProcessMessageId::VerifyEditorAndProjectPath,
            )),
        })
    }

    fn validate_source_engine_for_editor_launch(
        &mut self,
        engine: &SourceEngineInstall,
        target: &str,
    ) -> Result<(), HubError> {
        let validation = validate_source_engine(&engine.source_dir);
        if validation == SourceEngineValidation::Valid {
            return Ok(());
        }

        let detail = source_engine_validation_detail(validation);
        let recovery = source_engine_validation_recovery(validation);
        self.record_editor_launch_failure(
            target.to_string(),
            detail.clone(),
            Vec::new(),
            recovery.clone(),
        )?;
        Err(HubError::status(detail, Some(recovery)))
    }

    fn prepare_empty_editor_launch(&mut self) -> Result<PendingEditorLaunch, HubError> {
        let engine = active_source_engine(
            &self.config.engines,
            self.config.active_engine_id.as_deref(),
        )
        .cloned()
        .ok_or_else(|| HubError::message("Editor launch requires an active Source Engine"))?;
        let build_profile = self.config.settings.build_profile.as_mode().to_string();
        if let Err(error) = self.ensure_editor_available_at(&engine, &build_profile) {
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
                executable: staged_editor_executable(engine.staged_engine_dir()),
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

        let attempt_id = report.attempt_id;
        let process_id = report.process_id;
        let target = pending_launch.target.clone();
        let completion = (|| {
            if pending_launch.remember_project {
                let Some(project_path) = pending_launch.project_path else {
                    return Err(HubError::message(
                        "Editor launch project state is missing the project path",
                    ));
                };
                self.remember_project(RecentProject::with_now(project_path)?)?;
            }
            let detail = match report.outcome {
                EditorLaunchOutcome::Spawned => HubMessage::with_params(
                    HubMessageId::Process(ProcessMessageId::StartedProcess),
                    [report.process_id.to_string()],
                ),
                EditorLaunchOutcome::FocusedExisting => HubMessage::with_params(
                    HubMessageId::Process(ProcessMessageId::FocusedExistingEditor),
                    [report.process_id.to_string()],
                ),
            };
            self.record_action_and_persist(HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action: HubActionKind::OpenEditor,
                status: HubActionStatus::Success,
                target: pending_launch.target.clone(),
                detail,
                log_excerpt: HubMessage::empty(),
                recovery: None,
                process_id: Some(report.process_id),
                command_line,
                output_dir: Some(self.config.settings.default_build_output_dir.clone()),
            })?;
            let (operation, detail) = if pending_launch.remember_project {
                (
                    TaskOperationKind::Project,
                    match report.outcome {
                        EditorLaunchOutcome::Spawned => HubMessage::with_params(
                            HubMessageId::Process(ProcessMessageId::OpeningTargetProcess),
                            [pending_launch.target.clone(), report.process_id.to_string()],
                        ),
                        EditorLaunchOutcome::FocusedExisting => HubMessage::with_params(
                            HubMessageId::Process(ProcessMessageId::FocusedExistingEditor),
                            [report.process_id.to_string()],
                        ),
                    },
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
            self.task_status = TaskStatus::success(
                match report.outcome {
                    EditorLaunchOutcome::Spawned => "Editor launched",
                    EditorLaunchOutcome::FocusedExisting => "Existing editor focused",
                },
                detail,
            )
            .with_operation(operation, pending_launch.target)
            .with_task_id(attempt_id);
            Ok(())
        })();
        let terminal = if report.outcome == EditorLaunchOutcome::Spawned {
            self.observe_completed_editor_launch(attempt_id, process_id, target)
        } else {
            Ok(())
        };
        completion.and(terminal)
    }

    fn ensure_editor_available_at(
        &self,
        engine: &SourceEngineInstall,
        expected_profile: &str,
    ) -> Result<(), HubError> {
        let staged_engine_dir = engine.staged_engine_dir();
        if staged_editor_executable_exists(&staged_engine_dir) {
            return super::build_actions::staged_build::validate_staged_editor_runtime_build(
                &staged_engine_dir,
                &engine.source_dir,
                expected_profile,
            )
            .map(|_| ());
        }
        let executable = staged_editor_executable(&staged_engine_dir);
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
        self.config.recent_projects = merge_recent_project_entries(
            std::iter::once(project),
            self.config.recent_projects.clone(),
        )
        .map_err(|error| {
            HubError::message(format!("merge shared recent-project entries: {error}"))
        })?;
        self.activate_project_engine_for_path(&last_project_path);
        self.refresh_project_context_views(
            true,
            self.config.active_engine_id != active_engine_before,
        )?;
        self.persist()
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

fn wait_for_project_editor_ready(
    project_path: &Path,
    handshake_session: HubSessionToken,
    mut child: SupervisedChild,
    context: &BackgroundTaskContext,
) -> Result<TaskExecutionOutcome<u32>, HubError> {
    let child_process_id = child.id();
    let handshake = match wait_for_editor_handshake(
        project_path,
        handshake_session,
        &mut child,
        context.cancellation(),
    ) {
        Ok(handshake) => handshake,
        Err(error) => {
            return match context
                .editor_child_reaper()
                .fail_before_ready(context.cancellation().task_id(), child)
            {
                Ok(()) => Err(error),
                Err(termination_error) => Err(HubError::message(format!(
                    "{error}; failed to terminate Editor process tree: {termination_error}"
                ))),
            };
        }
    };
    let mailbox = match handshake {
        TaskExecutionOutcome::Completed(mailbox) => mailbox,
        TaskExecutionOutcome::Cancelled => {
            context
                .editor_child_reaper()
                .cancel_before_ready(context.cancellation().task_id(), child)?;
            return Ok(TaskExecutionOutcome::Cancelled);
        }
    };
    match validate_project_editor_handshake(child_process_id, mailbox) {
        Ok(process_id) => {
            context
                .editor_child_reaper()
                .register(context.cancellation().task_id(), child)?;
            Ok(TaskExecutionOutcome::Completed(process_id))
        }
        Err(error) => match context
            .editor_child_reaper()
            .fail_before_ready(context.cancellation().task_id(), child)
        {
            Ok(()) => Err(error),
            Err(termination_error) => Err(HubError::message(format!(
                "{error}; failed to terminate Editor process tree: {termination_error}"
            ))),
        },
    }
}

fn validate_project_editor_handshake(
    child_process_id: u32,
    mailbox: zircon_runtime_interface::hub_protocol::HubEditorMailboxV1,
) -> Result<u32, HubError> {
    match mailbox.outcome {
        HubEditorLaunchOutcomeV1::Ready { receipt } => {
            validate_ready_child_receipt(child_process_id, &receipt)?;
            Ok(receipt.editor_process_id())
        }
        HubEditorLaunchOutcomeV1::Failed { code } => Err(HubError::message(format!(
            "editor reported startup failure category through the Hub handshake: {}",
            startup_failure_code_label(code)
        ))),
    }
}

fn validate_ready_child_receipt(
    child_process_id: u32,
    receipt: &HubEditorReadyReceiptV1,
) -> Result<(), HubError> {
    if receipt.editor_process_id() == child_process_id {
        Ok(())
    } else {
        Err(HubError::message(
            "editor Hub Ready receipt is not bound to the Hub-supervised child process",
        ))
    }
}

fn startup_failure_code_label(code: HubEditorStartupFailureCodeV1) -> &'static str {
    match code {
        HubEditorStartupFailureCodeV1::Startup => "startup",
        HubEditorStartupFailureCodeV1::ProjectActivation => "project_activation",
        HubEditorStartupFailureCodeV1::FocusInboxBinding => "focus_inbox_binding",
        HubEditorStartupFailureCodeV1::NativeWindow => "native_window",
        HubEditorStartupFailureCodeV1::FirstPresent => "first_present",
        HubEditorStartupFailureCodeV1::HostWindow => "host_window",
        HubEditorStartupFailureCodeV1::MailboxPublish => "mailbox_publish",
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
#[path = "editor_launch_actions/tests/cases.rs"]
mod tests;
