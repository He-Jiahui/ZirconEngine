use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::process::FolderPickerRequest;
use crate::projects::{
    create_project, merge_recent_projects, metadata_for_path_mut, normalize_project_root,
    project_metadata_key, project_paths_match, prune_empty_metadata, validate_project_root,
    CreateProjectRequest, ProjectTemplate, ProjectValidation, RecentProject,
};
use crate::state::{
    EngineMessageId, HubActionKind, HubActionRecord, HubActionStatus, HubMessage, HubMessageId,
    HubPage, ProjectMessageId, ProjectSubpage, ProjectViewMode, TaskOperationKind, TaskStatus,
};
use crate::tauri_app::action_request::{
    CreateProjectActionPayload, ImportProjectActionPayload, ProjectTargetActionPayload,
};
use crate::tauri_app::view_model::HubTextBundle;

use super::{recent_project_display_name, HubRuntimeSession};

#[cfg(test)]
const CREATE_KEPT_FOLDER_RECOVERY: &str =
    "The project folder was kept on disk; use Import Project to add it to Hub";

impl HubRuntimeSession {
    pub(super) fn create_project_from_payload(
        &mut self,
        payload: CreateProjectActionPayload,
    ) -> Result<(), HubError> {
        self.remember_create_project_payload(&payload);
        let template = match ProjectTemplate::from_enabled_id(&payload.template) {
            Some(template) => template,
            None => {
                self.record_lifecycle_failure(
                    HubActionKind::CreateProject,
                    payload.name.clone(),
                    HubMessage::with_params(
                        HubMessageId::Project(ProjectMessageId::TemplateComingSoon),
                        [payload.template],
                    ),
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::ChooseRenderableTemplate,
                    )),
                    None,
                )?;
                return Ok(());
            }
        };
        let engine_id = match self.resolve_project_engine_id(payload.engine_id) {
            Ok(engine_id) => engine_id,
            Err(error) => {
                let (detail, _) = error.into_status_messages();
                self.record_lifecycle_failure(
                    HubActionKind::CreateProject,
                    payload.name.clone(),
                    detail,
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::RegisterEngineBeforeCreate,
                    )),
                    None,
                )?;
                return Ok(());
            }
        };
        let request = CreateProjectRequest::new(payload.name.clone(), payload.location, template);
        let report = match create_project(&request) {
            Ok(report) => report,
            Err(error) => {
                let detail = error.to_string();
                let detail_message = create_project_error_message(&detail);
                let recovery = if detail == "Target directory must be empty" {
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::ExistingFolderUseImport,
                    ))
                } else {
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::ChooseEmptyTargetFolder,
                    ))
                };
                self.record_lifecycle_failure(
                    HubActionKind::CreateProject,
                    payload.name.clone(),
                    detail_message,
                    recovery,
                    None,
                )?;
                return Ok(());
            }
        };

        let project_root = report.project_root.clone();
        if let Err(error) = self.remember_lifecycle_project(
            payload.name.clone(),
            project_root.clone(),
            engine_id,
            Some(template.id().to_string()),
        ) {
            return self.record_create_project_kept_folder_failure(
                payload.name,
                &project_root,
                error,
            );
        }
        self.push_lifecycle_record(
            HubActionKind::CreateProject,
            HubActionStatus::Success,
            payload.name.clone(),
            HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::CreatedPath),
                [project_root.to_string_lossy().into_owned()],
            ),
            None,
            Some(project_root.clone()),
        );
        self.task_status = TaskStatus::success(
            "Project created",
            HubMessage::legacy(project_root.to_string_lossy().into_owned()),
        )
        .with_operation(TaskOperationKind::Project, payload.name);
        self.new_project_name.clear();
        if let Err(error) = self.persist(Some(&project_root)) {
            self.config.action_history.retain(|record| {
                record.status != HubActionStatus::Success
                    || record.action != HubActionKind::CreateProject
                    || record.output_dir.as_deref() != Some(project_root.as_path())
            });
            return self.record_create_project_kept_folder_failure(
                request.project_name,
                &project_root,
                error,
            );
        }
        Ok(())
    }

    pub(super) fn import_project_from_action(
        &mut self,
        target_id: Option<&str>,
        payload: Option<ImportProjectActionPayload>,
    ) -> Result<(), HubError> {
        let mut project_root = payload
            .as_ref()
            .and_then(|payload| payload.path.clone().or_else(|| payload.folder.clone()))
            .or_else(|| target_id.map(PathBuf::from));

        if project_root.is_none() {
            let text = HubTextBundle::new(self.config.settings.language);
            project_root = match (self.folder_picker)(&FolderPickerRequest::new(
                import_project_picker_title(text),
                Some(self.config.settings.default_project_dir.clone()),
            )) {
                Ok(path) => path,
                Err(error) => {
                    let detail = HubMessage::legacy(error.to_string());
                    self.record_lifecycle_failure(
                        HubActionKind::ImportProject,
                        "Import Project".to_string(),
                        detail,
                        HubMessage::new(HubMessageId::Project(
                            ProjectMessageId::ChooseFolderWithManifest,
                        )),
                        None,
                    )?;
                    return Ok(());
                }
            };
        }

        let Some(project_root) = project_root else {
            self.task_status = TaskStatus::warning(
                "Import cancelled",
                HubMessage::new(HubMessageId::Project(
                    ProjectMessageId::NoProjectFolderSelected,
                )),
                HubMessage::new(HubMessageId::Project(ProjectMessageId::RunImportAgain)),
            )
            .with_operation(TaskOperationKind::Project, "Import Project");
            return Ok(());
        };

        let project_root = normalize_project_root(&project_root);
        if let Some(error) = project_validation_error(&project_root) {
            self.record_lifecycle_failure(
                HubActionKind::ImportProject,
                project_root.to_string_lossy().into_owned(),
                error,
                HubMessage::new(HubMessageId::Project(
                    ProjectMessageId::ChooseFolderWithManifest,
                )),
                Some(project_root),
            )?;
            return Ok(());
        }

        let engine_id =
            match self.resolve_project_engine_id(payload.and_then(|payload| payload.engine_id)) {
                Ok(engine_id) => engine_id,
                Err(error) => {
                    let (detail, _) = error.into_status_messages();
                    self.record_lifecycle_failure(
                        HubActionKind::ImportProject,
                        project_root.to_string_lossy().into_owned(),
                        detail,
                        HubMessage::new(HubMessageId::Project(
                            ProjectMessageId::RegisterEngineBeforeImport,
                        )),
                        Some(project_root),
                    )?;
                    return Ok(());
                }
            };
        let (display_name, project_root) =
            match self.find_recent_project_by_filesystem_key(&project_root) {
                Some(existing) => (recent_project_display_name(&existing), existing.path),
                None => (project_display_name_from_path(&project_root), project_root),
            };
        self.remember_lifecycle_project(
            display_name.clone(),
            project_root.clone(),
            engine_id,
            None,
        )?;
        self.push_lifecycle_record(
            HubActionKind::ImportProject,
            HubActionStatus::Success,
            display_name.clone(),
            HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::ImportedPath),
                [project_root.to_string_lossy().into_owned()],
            ),
            None,
            Some(project_root.clone()),
        );
        self.task_status = TaskStatus::success(
            "Project imported",
            HubMessage::legacy(project_root.to_string_lossy().into_owned()),
        )
        .with_operation(TaskOperationKind::Project, display_name);
        self.persist(Some(&project_root))
    }

    pub(super) fn set_project_pinned(
        &mut self,
        target_id: Option<&str>,
        payload: Option<&ProjectTargetActionPayload>,
        pinned: bool,
    ) -> Result<(), HubError> {
        let project = self.resolve_action_project(target_id, payload)?;
        metadata_for_path_mut(&mut self.config.project_metadata, &project.path).pinned = pinned;
        prune_empty_metadata(&mut self.config.project_metadata);
        self.pending_delete_project_path = None;
        self.task_status = TaskStatus::success(
            if pinned {
                "Project pinned"
            } else {
                "Project unpinned"
            },
            HubMessage::legacy(recent_project_display_name(&project)),
        )
        .with_operation(
            TaskOperationKind::Project,
            recent_project_display_name(&project),
        );
        self.persist(None)
    }

    pub(super) fn remove_project_from_hub(
        &mut self,
        target_id: Option<&str>,
        payload: Option<&ProjectTargetActionPayload>,
    ) -> Result<(), HubError> {
        let project = self.resolve_action_project(target_id, payload)?;
        let display_name = recent_project_display_name(&project);
        self.drop_project_from_hub(&project.path)?;
        self.push_lifecycle_record(
            HubActionKind::RemoveProject,
            HubActionStatus::Success,
            display_name.clone(),
            HubMessage::new(HubMessageId::Project(ProjectMessageId::RemovedFromHub)),
            None,
            Some(project.path.clone()),
        );
        self.task_status = TaskStatus::success(
            "Project removed from Hub",
            HubMessage::new(HubMessageId::Project(ProjectMessageId::FilesLeftOnDisk)),
        )
        .with_operation(TaskOperationKind::Project, display_name);
        self.persist(None)
    }

    pub(super) fn request_project_delete(
        &mut self,
        target_id: Option<&str>,
        payload: Option<&ProjectTargetActionPayload>,
    ) -> Result<(), HubError> {
        let project = self.resolve_action_project(target_id, payload)?;
        self.pending_delete_project_path = Some(project.path.clone());
        self.task_status = TaskStatus::warning(
            "Delete requested",
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::ConfirmDeleteRecycleBin,
            )),
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::CancelDeleteUnchanged,
            )),
        )
        .with_operation(
            TaskOperationKind::Project,
            recent_project_display_name(&project),
        );
        self.persist(None)
    }

    pub(super) fn cancel_project_delete(
        &mut self,
        target_id: Option<&str>,
        payload: Option<&ProjectTargetActionPayload>,
    ) -> Result<(), HubError> {
        let operation_target =
            if super::action_targets::project_target_candidates(target_id, payload).is_empty() {
                self.pending_delete_project_path
                    .as_ref()
                    .map(|path| project_display_name_from_path(path))
                    .unwrap_or_else(|| "Project".to_string())
            } else {
                recent_project_display_name(
                    &self.resolve_pending_delete_project(target_id, payload)?,
                )
            };
        self.pending_delete_project_path = None;
        self.task_status = TaskStatus::success(
            "Delete cancelled",
            HubMessage::new(HubMessageId::Project(ProjectMessageId::ProjectUnchanged)),
        )
        .with_operation(TaskOperationKind::Project, operation_target);
        self.persist(None)
    }

    pub(super) fn confirm_project_delete(
        &mut self,
        target_id: Option<&str>,
        payload: Option<&ProjectTargetActionPayload>,
    ) -> Result<(), HubError> {
        let project = self.resolve_pending_delete_project(target_id, payload)?;
        let display_name = recent_project_display_name(&project);
        match (self.recycle_delete)(project.path.clone()) {
            Ok(()) => {
                self.drop_project_from_hub(&project.path)?;
                self.push_lifecycle_record(
                    HubActionKind::DeleteProject,
                    HubActionStatus::Success,
                    display_name.clone(),
                    HubMessage::new(HubMessageId::Project(ProjectMessageId::MovedToRecycleBin)),
                    None,
                    Some(project.path.clone()),
                );
                self.task_status = TaskStatus::success(
                    "Project deleted",
                    HubMessage::new(HubMessageId::Project(ProjectMessageId::MovedToRecycleBin)),
                )
                .with_operation(TaskOperationKind::Project, display_name);
                self.persist(None)
            }
            Err(error) => {
                self.pending_delete_project_path = Some(project.path.clone());
                let recovery = if cfg!(target_os = "windows") {
                    HubMessage::new(HubMessageId::Project(
                        ProjectMessageId::DeletionFailureRecovery,
                    ))
                } else {
                    HubMessage::new(HubMessageId::Project(ProjectMessageId::RecycleUnsupported))
                };
                self.record_lifecycle_failure(
                    HubActionKind::DeleteProject,
                    display_name,
                    HubMessage::legacy(error.to_string()),
                    recovery,
                    Some(project.path),
                )
            }
        }
    }

    fn remember_lifecycle_project(
        &mut self,
        display_name: String,
        project_root: PathBuf,
        engine_id: Option<String>,
        template_id: Option<String>,
    ) -> Result<(), HubError> {
        let active_engine_before = self.config.active_engine_id.clone();
        self.selected_page = HubPage::Projects;
        self.project_subpage = ProjectSubpage::ProjectDetail;
        self.project_view_mode = ProjectViewMode::List;
        self.selected_project_path = Some(project_root.clone());
        self.pending_delete_project_path = None;
        self.config.recent_projects = merge_recent_projects(
            std::iter::once(RecentProject::with_now(display_name, project_root.clone())),
            self.config.recent_projects.clone(),
        );
        let metadata = metadata_for_path_mut(&mut self.config.project_metadata, &project_root);
        if engine_id.is_some() {
            metadata.engine_id = engine_id;
        }
        if template_id.is_some() {
            metadata.last_selected_template = template_id.clone();
        }
        if let Some(template_id) = template_id {
            self.selected_template_id = template_id;
        }
        self.activate_project_engine_for_path(&project_root);
        self.refresh_project_context_views(
            true,
            self.config.active_engine_id != active_engine_before,
        )
    }

    fn resolve_project_engine_id(
        &self,
        requested_engine_id: Option<String>,
    ) -> Result<Option<String>, HubError> {
        let candidate = requested_engine_id
            .filter(|id| !id.trim().is_empty())
            .or_else(|| self.new_project_engine_id.clone())
            .or_else(|| self.config.active_engine_id.clone());
        let Some(engine_id) = candidate else {
            return Ok(None);
        };
        if self
            .config
            .engines
            .iter()
            .any(|engine| engine.id == engine_id)
        {
            Ok(Some(engine_id))
        } else {
            Err(HubError::status(
                HubMessage::with_params(
                    HubMessageId::Engine(EngineMessageId::UnknownSourceEngine),
                    [engine_id],
                ),
                None,
            ))
        }
    }

    fn resolve_action_project(
        &mut self,
        target_id: Option<&str>,
        payload: Option<&ProjectTargetActionPayload>,
    ) -> Result<RecentProject, HubError> {
        let targets = super::action_targets::project_target_candidates(target_id, payload);
        if !targets.is_empty() {
            if let Some(project) = targets
                .iter()
                .find_map(|target| self.find_recent_project(target))
            {
                return Ok(project);
            }
            if let Some(selected_path) = self.selected_project_path.clone() {
                if let Some(target) = targets
                    .iter()
                    .find(|target| project_paths_match(&selected_path, target))
                {
                    return Ok(RecentProject::with_now(
                        project_display_name_from_path(Path::new(target)),
                        PathBuf::from(target),
                    ));
                }
            }
            if let Some(target) = targets.first() {
                return Err(HubError::message(format!(
                    "Unknown recent project: {target}"
                )));
            }
        }

        if let Some(project) = self.selected_recent_project() {
            return Ok(project);
        }
        let Some(path) = self.selected_project_path.clone() else {
            return Err(HubError::message("Select a project first"));
        };
        Ok(RecentProject::with_now(
            project_display_name_from_path(&path),
            path,
        ))
    }

    fn resolve_pending_delete_project(
        &mut self,
        target_id: Option<&str>,
        payload: Option<&ProjectTargetActionPayload>,
    ) -> Result<RecentProject, HubError> {
        let targets = super::action_targets::project_target_candidates(target_id, payload);
        let project = if targets.is_empty() {
            if let Some(path) = self.pending_delete_project_path.clone() {
                self.find_recent_project(&path.to_string_lossy())
                    .unwrap_or_else(|| {
                        RecentProject::with_now(project_display_name_from_path(&path), path)
                    })
            } else {
                self.resolve_action_project(None, None)?
            }
        } else {
            self.resolve_action_project(target_id, payload)?
        };

        let Some(pending_path) = self.pending_delete_project_path.as_ref() else {
            return Err(HubError::message(
                "Request delete before confirming project deletion",
            ));
        };
        if !project_paths_match(&project.path, pending_path) {
            return Err(HubError::message(
                "Confirm delete target does not match the pending project",
            ));
        }
        Ok(project)
    }

    fn drop_project_from_hub(&mut self, path: &Path) -> Result<(), HubError> {
        self.config
            .recent_projects
            .retain(|project| !project_paths_match(&project.path, path));
        self.config
            .project_metadata
            .remove(&project_metadata_key(path));
        if self
            .selected_project_path
            .as_ref()
            .is_some_and(|selected| project_paths_match(selected, path))
        {
            self.selected_project_path = None;
        }
        if self
            .pending_delete_project_path
            .as_ref()
            .is_some_and(|pending| project_paths_match(pending, path))
        {
            self.pending_delete_project_path = None;
        }
        self.refresh_selected_project_scoped_views()
    }

    fn record_lifecycle_failure(
        &mut self,
        action: HubActionKind,
        target: String,
        detail: HubMessage,
        recovery: HubMessage,
        output_dir: Option<PathBuf>,
    ) -> Result<(), HubError> {
        self.push_lifecycle_record(
            action,
            HubActionStatus::Failed,
            target.clone(),
            detail.clone(),
            Some(recovery.clone()),
            output_dir,
        );
        self.task_status =
            TaskStatus::error(format!("{} failed", action.label()), detail, recovery)
                .with_operation(TaskOperationKind::Project, target);
        self.persist(None)
    }

    fn record_create_project_kept_folder_failure(
        &mut self,
        target: String,
        project_root: &Path,
        error: HubError,
    ) -> Result<(), HubError> {
        let detail = HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::FolderCreatedButRecordFailed),
            [
                project_root.to_string_lossy().into_owned(),
                error.to_string(),
            ],
        );
        let recovery =
            HubMessage::new(HubMessageId::Project(ProjectMessageId::KeptFolderUseImport));
        self.push_lifecycle_record(
            HubActionKind::CreateProject,
            HubActionStatus::Failed,
            target.clone(),
            detail.clone(),
            Some(recovery.clone()),
            Some(project_root.to_path_buf()),
        );
        self.task_status = TaskStatus::error("Create Project failed", detail, recovery)
            .with_operation(TaskOperationKind::Project, target);
        let _ = self.persist_unchecked(None);
        Ok(())
    }

    fn push_lifecycle_record(
        &mut self,
        action: HubActionKind,
        status: HubActionStatus,
        target: String,
        detail: HubMessage,
        recovery: Option<HubMessage>,
        output_dir: Option<PathBuf>,
    ) {
        crate::state::push_action_record(
            &mut self.config.action_history,
            HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action,
                status,
                target,
                detail: detail.clone(),
                log_excerpt: detail,
                recovery,
                process_id: None,
                command_line: Vec::new(),
                output_dir,
            },
        );
    }
}

fn import_project_picker_title(text: HubTextBundle) -> &'static str {
    text.pair("Import Zircon Project", "导入 Zircon 项目")
}

fn project_validation_error(project_root: &Path) -> Option<HubMessage> {
    match validate_project_root(project_root) {
        ProjectValidation::Valid => None,
        ProjectValidation::MissingRoot => Some(HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::FolderDoesNotExist),
            [project_root.to_string_lossy().into_owned()],
        )),
        ProjectValidation::MissingManifest => Some(HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::ManifestNotFound),
            [project_root.to_string_lossy().into_owned()],
        )),
        ProjectValidation::InvalidManifest => Some(HubMessage::with_params(
            HubMessageId::Project(ProjectMessageId::ManifestParseFailed),
            [project_root.to_string_lossy().into_owned()],
        )),
    }
}

fn create_project_error_message(detail: &str) -> HubMessage {
    match detail {
        "Project root is required" => {
            HubMessage::new(HubMessageId::Project(ProjectMessageId::ProjectRootRequired))
        }
        "Target path already exists as a file" => {
            HubMessage::new(HubMessageId::Project(ProjectMessageId::TargetPathIsFile))
        }
        "Target directory must be empty" => HubMessage::new(HubMessageId::Project(
            ProjectMessageId::TargetDirectoryMustBeEmpty,
        )),
        "Project path is required" => {
            HubMessage::new(HubMessageId::Project(ProjectMessageId::ProjectPathRequired))
        }
        _ => HubMessage::legacy(detail.to_string()),
    }
}

fn project_display_name_from_path(path: &Path) -> String {
    path.file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .unwrap_or("Zircon Project")
        .to_string()
}

#[cfg(test)]
mod tests {
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
        fs::write(target.join("zircon-project.toml"), "name = \"Game\"\n").unwrap();
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
        fs::write(project.join("zircon-project.toml"), "name = \"Imported\"\n").unwrap();
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
        assert_eq!(session.config.recent_projects[0].display_name, "Imported");
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
        fs::write(project.join("zircon-project.toml"), "name = \"Imported\"\n").unwrap();
        let mut session = session_with_source(&temp, &source);
        session.config.recent_projects = vec![RecentProject::new("Existing Imported", &project, 1)];

        session
            .apply_action(HubActionRequest {
                action_id: "import-project".to_string(),
                target_id: None,
                payload: Some(serde_json::json!({ "path": project.join(".") })),
            })
            .expect("duplicate import should select the existing project entry");

        assert_eq!(session.config.recent_projects.len(), 1);
        assert_eq!(
            session.config.recent_projects[0].display_name,
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
        fs::write(project.join("zircon-project.toml"), "name = \"Game\"\n").unwrap();
        let mut session = session_with_source(&temp, &source);
        session.config.recent_projects =
            vec![crate::projects::RecentProject::new("Game", &project, 1)];
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
    fn project_management_actions_resolve_typed_project_path_before_legacy_target() {
        let temp = temp_test_dir("zircon-hub-project-management-typed-target");
        let source = temp.join("ZirconEngine");
        fs::create_dir_all(&source).unwrap();
        let fallback = temp.join("Fallback");
        let target = temp.join("Target");
        fs::create_dir_all(&fallback).unwrap();
        fs::create_dir_all(&target).unwrap();
        fs::write(
            fallback.join("zircon-project.toml"),
            "name = \"Fallback\"\n",
        )
        .unwrap();
        fs::write(target.join("zircon-project.toml"), "name = \"Target\"\n").unwrap();
        let mut session = session_with_source(&temp, &source);
        session.config.recent_projects = vec![
            crate::projects::RecentProject::new("Fallback", &fallback, 1),
            crate::projects::RecentProject::new("Target", &target, 2),
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
        fs::write(
            fallback.join("zircon-project.toml"),
            "name = \"Fallback\"\n",
        )
        .unwrap();
        fs::write(target.join("zircon-project.toml"), "name = \"Target\"\n").unwrap();
        let mut session = session_with_source(&temp, &source);
        session.config.recent_projects = vec![
            crate::projects::RecentProject::new("Fallback", &fallback, 1),
            crate::projects::RecentProject::new("Target", &target, 2),
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
        fs::write(project.join("zircon-project.toml"), "name = \"Game\"\n").unwrap();
        let mut session = session_with_source(&temp, &source);
        session.config.recent_projects = vec![RecentProject::new("Game", &project, 1)];
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
        fs::write(project.join("zircon-project.toml"), "name = \"Game\"\n").unwrap();
        let mut session = session_with_source(&temp, &source);
        session.config.recent_projects = vec![RecentProject::new("Game", &project, 1)];
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
