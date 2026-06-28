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
            HubMessage::raw_text(project_root.to_string_lossy().into_owned()),
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
                    let detail = HubMessage::raw_text(error.to_string());
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
            HubMessage::raw_text(project_root.to_string_lossy().into_owned()),
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
            HubMessage::raw_text(recent_project_display_name(&project)),
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
                    HubMessage::raw_text(error.to_string()),
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
        _ => HubMessage::raw_text(detail.to_string()),
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
#[path = "project_actions/tests.rs"]
mod tests;
