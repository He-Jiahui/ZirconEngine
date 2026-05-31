use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::process::{launch_editor, EditorLaunchCommand, EditorLaunchRequest};
use crate::projects::{
    install_package_to_device, metadata_for_path, metadata_for_path_mut, project_metadata_key,
    project_paths_match, prune_empty_metadata, recycle_delete_project, validate_project_root,
    CreateProjectRequest, DeviceInstallRequest, ProjectPackageReport, ProjectPackageRequest,
    ProjectTemplate, ProjectValidation, RecentProject,
};
use crate::state::{
    HubActionKind, HubActionRecord, HubActionStatus, ProjectFilterMode, ProjectSortMode,
    ProjectSubpage, ProjectViewMode, TaskOperationKind, TaskStatus,
};

use super::super::HubWindow;
use super::HubRuntime;

impl HubRuntime {
    pub(super) fn ensure_new_project_engine_selection(&mut self) {
        let current = self.new_project_engine_id.clone();
        if current
            .as_deref()
            .is_some_and(|id| self.config.engines.iter().any(|engine| engine.id == id))
        {
            return;
        }
        self.new_project_engine_id = self
            .config
            .active_engine_id
            .clone()
            .filter(|id| self.config.engines.iter().any(|engine| engine.id == *id))
            .or_else(|| self.config.engines.first().map(|engine| engine.id.clone()));
    }

    pub(super) fn sync_new_project_engine_after_active_engine_change(
        &mut self,
        previous_active_engine_id: Option<&str>,
    ) {
        let active_engine_id = self
            .config
            .active_engine_id
            .clone()
            .filter(|id| self.config.engines.iter().any(|engine| engine.id == *id));
        let current = self.new_project_engine_id.clone();
        let current_is_valid = current
            .as_deref()
            .is_some_and(|id| self.config.engines.iter().any(|engine| engine.id == id));
        let followed_previous_active =
            current.as_deref().is_some() && current.as_deref() == previous_active_engine_id;
        if current.is_none() || !current_is_valid || followed_previous_active {
            self.new_project_engine_id = active_engine_id;
        }
    }

    pub(super) fn show_project_subpage_by_id(&mut self, subpage_id: &str) -> Result<(), HubError> {
        let Some(subpage) = ProjectSubpage::from_id(subpage_id) else {
            return Err(HubError::message(format!(
                "Unknown Projects page: {subpage_id}"
            )));
        };
        self.project_subpage = subpage;
        if subpage == ProjectSubpage::ProjectBrowser {
            self.project_view_mode = ProjectViewMode::List;
        }
        if subpage == ProjectSubpage::NewProject {
            self.ensure_new_project_engine_selection();
        }
        self.pending_delete_project_path = None;
        Ok(())
    }

    pub(super) fn search_projects(&mut self, query: &str) {
        self.search_query = query.to_string();
    }

    pub(super) fn select_project_path(&mut self, project_path: &str) -> Result<(), HubError> {
        let path = PathBuf::from(project_path);
        let Some(project) = self
            .config
            .recent_projects
            .iter()
            .find(|project| project_paths_match(&project.path, &path))
            .cloned()
        else {
            return Err(HubError::message(format!(
                "Unknown recent project: {project_path}"
            )));
        };
        self.selected_project_path = Some(project.path.clone());
        let active_engine_before = self.config.active_engine_id.clone();
        self.activate_project_engine_for_path(&project.path);
        self.refresh_project_context_views(
            true,
            self.config.active_engine_id != active_engine_before,
        )?;
        let display_name = recent_project_display_name(&project);
        self.task_status = TaskStatus::success("Project selected", display_name.clone())
            .with_operation(TaskOperationKind::Project, display_name);
        Ok(())
    }

    pub(super) fn view_all_projects(&mut self) {
        self.search_query.clear();
        self.project_filter = ProjectFilterMode::All;
        self.project_view_mode = ProjectViewMode::List;
        self.project_subpage = ProjectSubpage::ProjectBrowser;
        self.task_status = TaskStatus::success("All projects", "Showing all recent projects");
    }

    pub(super) fn set_project_filter_by_id(&mut self, filter_id: &str) -> Result<(), HubError> {
        let Some(filter) = ProjectFilterMode::from_id(filter_id) else {
            return Err(HubError::message(format!(
                "Unknown project filter mode: {filter_id}"
            )));
        };
        self.project_filter = filter;
        self.task_status = TaskStatus::success(
            "Projects filtered",
            format!("Showing {}", self.project_filter.label()),
        );
        Ok(())
    }

    pub(super) fn set_project_sort_by_id(&mut self, sort_id: &str) -> Result<(), HubError> {
        let Some(sort) = ProjectSortMode::from_id(sort_id) else {
            return Err(HubError::message(format!(
                "Unknown project sort mode: {sort_id}"
            )));
        };
        self.project_sort = sort;
        self.task_status = TaskStatus::success(
            "Projects sorted",
            format!("Sorting by {}", self.project_sort.label()),
        );
        Ok(())
    }

    pub(super) fn set_project_view_mode_by_id(&mut self, mode_id: &str) -> Result<(), HubError> {
        let Some(mode) = ProjectViewMode::from_id(mode_id) else {
            return Err(HubError::message(format!(
                "Unknown project view mode: {mode_id}"
            )));
        };
        self.project_view_mode = mode;
        self.project_subpage = if mode == ProjectViewMode::List {
            ProjectSubpage::ProjectBrowser
        } else {
            ProjectSubpage::Dashboard
        };
        Ok(())
    }

    pub(super) fn open_project_detail(&mut self, project_path: &str) -> Result<(), HubError> {
        self.select_project_path(project_path)?;
        self.project_subpage = ProjectSubpage::ProjectDetail;
        self.project_view_mode = ProjectViewMode::List;
        self.pending_delete_project_path = None;
        Ok(())
    }

    pub(super) fn select_project_template_by_id(
        &mut self,
        template_id: &str,
    ) -> Result<(), HubError> {
        let template_id = template_id.trim();
        if template_id.is_empty() {
            return Err(HubError::message("Project template is required"));
        }
        self.selected_template_id = template_id.to_string();
        Ok(())
    }

    pub(super) fn selected_template_for_create(&self) -> Result<ProjectTemplate, HubError> {
        ProjectTemplate::from_enabled_id(&self.selected_template_id).ok_or_else(|| {
            HubError::message(format!(
                "Project template is not available: {}",
                self.selected_template_id
            ))
        })
    }

    pub(super) fn open_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let project_path = PathBuf::from(ui.get_project_path().to_string());
        self.open_project_path(ui, project_path, None)
    }

    pub(super) fn open_recent_project(
        &mut self,
        ui: &HubWindow,
        project_path: &str,
    ) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let project_path = PathBuf::from(project_path);
        let display_name = self
            .config
            .recent_projects
            .iter()
            .find(|project| project_paths_match(&project.path, &project_path))
            .map(|project| project.display_name.clone());
        self.open_project_path(ui, project_path, display_name)
    }

    pub(super) fn open_project_path(
        &mut self,
        ui: &HubWindow,
        project_path: PathBuf,
        display_name_hint: Option<String>,
    ) -> Result<(), HubError> {
        if project_path.as_os_str().is_empty() {
            let detail = "Project path is required".to_string();
            self.record_editor_launch_failure(
                "Project".to_string(),
                detail.clone(),
                Vec::new(),
                "Choose a valid Zircon project before opening it in Editor",
            )?;
            return Err(HubError::message(detail));
        }
        if validate_project_root(&project_path) != ProjectValidation::Valid {
            let detail = format!(
                "Project root is not valid: {}",
                project_path.to_string_lossy()
            );
            self.record_editor_launch_failure(
                display_name_hint
                    .clone()
                    .unwrap_or_else(|| project_path.to_string_lossy().into_owned()),
                detail.clone(),
                Vec::new(),
                "Check that the selected project directory contains a Zircon project manifest",
            )?;
            return Err(HubError::message(detail));
        }
        let display_name = display_name_hint.unwrap_or_else(|| {
            project_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Zircon Project")
                .to_string()
        });
        self.activate_project_engine_for_path(&project_path);
        if let Err(error) = self.ensure_editor_available(ui) {
            let detail = error.to_string();
            self.record_editor_launch_failure(
                display_name.clone(),
                detail,
                Vec::new(),
                "Build the editor/runtime payload or fix Source Engine settings before opening the project",
            )?;
            return Err(error);
        }
        let command = EditorLaunchCommand::from_preferred_engine(
            self.staged_engine_dir(),
            EditorLaunchRequest::OpenProject {
                project_path: project_path.clone(),
            },
        );
        let command_line = command.command_line();
        let child = match launch_editor(&command) {
            Ok(child) => child,
            Err(error) => {
                let detail = error.to_string();
                self.record_editor_launch_failure(
                    display_name.clone(),
                    detail,
                    command_line,
                    "Verify the staged zircon_editor executable exists and the project path is accessible",
                )?;
                return Err(error);
            }
        };
        let process_id = child.id();
        self.remember_project(RecentProject::with_now(display_name.clone(), project_path))?;
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::OpenEditor,
            status: HubActionStatus::Success,
            target: display_name.clone(),
            detail: format!("Started process {process_id}"),
            log_excerpt: String::new(),
            recovery: None,
            process_id: Some(process_id),
            command_line,
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        self.task_status = TaskStatus::success(
            "Editor launched",
            format!("Opening {display_name} (process {process_id})"),
        )
        .with_operation(TaskOperationKind::Project, display_name);
        Ok(())
    }

    pub(super) fn import_project_path(&mut self, project_path: PathBuf) -> Result<(), HubError> {
        if project_path.as_os_str().is_empty() {
            return Err(HubError::message("Project path is required"));
        }
        if validate_project_root(&project_path) != ProjectValidation::Valid {
            return Err(HubError::message(format!(
                "Project root is not valid: {}",
                project_path.to_string_lossy()
            )));
        }
        let display_name = project_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Zircon Project")
            .to_string();
        self.remember_project(RecentProject::with_now(
            display_name.clone(),
            project_path.clone(),
        ))?;
        self.project_filter = ProjectFilterMode::All;
        self.project_view_mode = ProjectViewMode::List;
        self.project_subpage = ProjectSubpage::ProjectDetail;
        self.pending_delete_project_path = None;
        self.search_query.clear();
        self.task_status =
            TaskStatus::success("Project imported", format!("Added {display_name} to Hub"))
                .with_operation(TaskOperationKind::Project, display_name);
        Ok(())
    }

    pub(super) fn install_recent_project_to_device(
        &mut self,
        ui: &HubWindow,
    ) -> Result<(), HubError> {
        let package_report = match self.package_recent_project_to_output_with_messages(
            ui,
            "No recent project is available to install",
            "Selected project is no longer available to install",
        ) {
            Ok(report) => report,
            Err(error) => {
                let detail = error.to_string();
                self.record_project_action_failure(
                    HubActionKind::InstallProject,
                    self.action_target_for_project_failure(),
                    detail,
                    "Select a valid project and package it before installing to a device",
                    Some(self.config.settings.default_device_install_dir.clone()),
                )?;
                return Err(error);
            }
        };
        let project_name = self.selected_project_label();
        let install_report =
            self.install_package_for_project(project_name.clone(), package_report)?;
        let detail = format!(
            "{} -> {} ({} files)",
            project_name,
            install_report.install_dir.to_string_lossy(),
            install_report.files_copied
        );
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::InstallProject,
            status: HubActionStatus::Success,
            target: project_name.clone(),
            detail: detail.clone(),
            log_excerpt: String::new(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: Some(install_report.install_dir.clone()),
        })?;
        self.task_status = TaskStatus::success("Installed to device", detail)
            .with_operation(TaskOperationKind::Project, project_name);
        Ok(())
    }

    pub(super) fn install_selected_project_to_device(
        &mut self,
        ui: &HubWindow,
    ) -> Result<(), HubError> {
        let package_report = match self.package_selected_project_to_output_with_messages(
            ui,
            "Select a project before installing",
            "Selected project is no longer available to install",
        ) {
            Ok(report) => report,
            Err(error) => {
                let detail = error.to_string();
                self.record_project_action_failure(
                    HubActionKind::InstallProject,
                    self.action_target_for_project_failure(),
                    detail,
                    "Select a valid project and package it before installing to a device",
                    Some(self.config.settings.default_device_install_dir.clone()),
                )?;
                return Err(error);
            }
        };
        let project_name = self.selected_project_label();
        let install_report =
            self.install_package_for_project(project_name.clone(), package_report)?;
        let detail = format!(
            "{} -> {} ({} files)",
            project_name,
            install_report.install_dir.to_string_lossy(),
            install_report.files_copied
        );
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::InstallProject,
            status: HubActionStatus::Success,
            target: project_name.clone(),
            detail: detail.clone(),
            log_excerpt: String::new(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: Some(install_report.install_dir.clone()),
        })?;
        self.task_status = TaskStatus::success("Installed to device", detail)
            .with_operation(TaskOperationKind::Project, project_name);
        Ok(())
    }

    pub(super) fn package_recent_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        let report = self.package_recent_project_to_output(ui)?;
        let project_name = self.selected_project_label();
        let detail = format!(
            "{} -> {} ({} files)",
            project_name,
            report.package_dir.to_string_lossy(),
            report.files_copied
        );
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::PackageProject,
            status: HubActionStatus::Success,
            target: project_name.clone(),
            detail: detail.clone(),
            log_excerpt: String::new(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: Some(report.package_dir),
        })?;
        self.task_status = TaskStatus::success("Package created", detail)
            .with_operation(TaskOperationKind::Project, project_name);
        Ok(())
    }

    pub(super) fn package_selected_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        let report = self.package_selected_project_to_output(ui)?;
        let project_name = self.selected_project_label();
        let detail = format!(
            "{} -> {} ({} files)",
            project_name,
            report.package_dir.to_string_lossy(),
            report.files_copied
        );
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::PackageProject,
            status: HubActionStatus::Success,
            target: project_name.clone(),
            detail: detail.clone(),
            log_excerpt: String::new(),
            recovery: None,
            process_id: None,
            command_line: Vec::new(),
            output_dir: Some(report.package_dir),
        })?;
        self.task_status = TaskStatus::success("Package created", detail)
            .with_operation(TaskOperationKind::Project, project_name);
        Ok(())
    }

    pub(super) fn package_recent_project_to_output(
        &mut self,
        ui: &HubWindow,
    ) -> Result<ProjectPackageReport, HubError> {
        self.package_recent_project_to_output_with_messages(
            ui,
            "No recent project is available to package",
            "Selected project is no longer available to package",
        )
    }

    fn package_recent_project_to_output_with_messages(
        &mut self,
        ui: &HubWindow,
        missing_project_message: &str,
        stale_project_message: &str,
    ) -> Result<ProjectPackageReport, HubError> {
        self.sync_from_ui(ui);
        let project = match self.selected_or_latest_recent_project_for_named_action(
            missing_project_message,
            stale_project_message,
        ) {
            Ok(project) => project,
            Err(error) => {
                let detail = error.to_string();
                self.record_project_action_failure(
                    HubActionKind::PackageProject,
                    self.action_target_for_project_failure(),
                    detail,
                    "Select an available project before packaging",
                    Some(self.config.settings.default_build_output_dir.clone()),
                )?;
                return Err(error);
            }
        };
        self.package_project_to_output(project)
    }

    pub(super) fn package_selected_project_to_output(
        &mut self,
        ui: &HubWindow,
    ) -> Result<ProjectPackageReport, HubError> {
        self.package_selected_project_to_output_with_messages(
            ui,
            "Select a project before packaging",
            "Selected project is no longer available to package",
        )
    }

    fn package_selected_project_to_output_with_messages(
        &mut self,
        ui: &HubWindow,
        missing_project_message: &str,
        stale_project_message: &str,
    ) -> Result<ProjectPackageReport, HubError> {
        self.sync_from_ui(ui);
        let project = match self
            .selected_project_for_named_action(missing_project_message, stale_project_message)
        {
            Ok(project) => project,
            Err(error) => {
                let detail = error.to_string();
                self.record_project_action_failure(
                    HubActionKind::PackageProject,
                    self.action_target_for_project_failure(),
                    detail,
                    "Select an available project before packaging",
                    Some(self.config.settings.default_build_output_dir.clone()),
                )?;
                return Err(error);
            }
        };
        self.package_project_to_output(project)
    }

    pub(super) fn package_project_to_output(
        &mut self,
        project: RecentProject,
    ) -> Result<ProjectPackageReport, HubError> {
        if validate_project_root(&project.path) != ProjectValidation::Valid {
            let detail = format!(
                "Project root is not valid: {}",
                project.path.to_string_lossy()
            );
            self.record_project_action_failure(
                HubActionKind::PackageProject,
                recent_project_display_name(&project),
                detail.clone(),
                "Check that the selected project directory contains a Zircon project manifest",
                Some(self.config.settings.default_build_output_dir.clone()),
            )?;
            return Err(HubError::message(detail));
        }
        let display_name = recent_project_display_name(&project);
        let request = ProjectPackageRequest::new(
            display_name.clone(),
            project.path.clone(),
            self.config.settings.default_build_output_dir.clone(),
        );
        match crate::projects::package_project(&request) {
            Ok(report) => Ok(report),
            Err(error) => {
                let detail = error.to_string();
                self.record_project_action_failure(
                    HubActionKind::PackageProject,
                    display_name,
                    detail,
                    "Check that the project root exists and the package output is outside the project",
                    Some(self.config.settings.default_build_output_dir.clone()),
                )?;
                Err(error)
            }
        }
    }

    pub(super) fn open_selected_project_in_editor(
        &mut self,
        ui: &HubWindow,
    ) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let project = match self.selected_project_for_named_action(
            "Select a project before opening",
            "Selected project is no longer available to open",
        ) {
            Ok(project) => project,
            Err(error) => {
                let detail = error.to_string();
                self.record_editor_launch_failure(
                    self.action_target_for_project_failure(),
                    detail,
                    Vec::new(),
                    "Select an available project before opening it in Editor",
                )?;
                return Err(error);
            }
        };
        let display_name = recent_project_display_name(&project);
        self.open_project_path(ui, project.path, Some(display_name))
    }

    pub(super) fn open_selected_project_or_editor(
        &mut self,
        ui: &HubWindow,
    ) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let Some(project) = (match self.selected_or_latest_recent_project_for_action() {
            Ok(project) => project,
            Err(error) => {
                let detail = error.to_string();
                self.record_editor_launch_failure(
                    self.action_target_for_project_failure(),
                    detail,
                    Vec::new(),
                    "Select an available project or launch Editor without a project",
                )?;
                return Err(error);
            }
        }) else {
            return self.launch_editor_without_project(ui);
        };
        let display_name = recent_project_display_name(&project);
        self.open_project_path(ui, project.path, Some(display_name))
    }

    pub(super) fn create_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let request = CreateProjectRequest::new(
            ui.get_project_name().to_string(),
            self.new_project_location.clone(),
            self.selected_template_for_create()?,
        );
        request
            .validate_launch_fields()
            .map_err(HubError::message)?;
        let root = request.target_root();
        let display_name = request.project_name.clone();
        let engine_id = self
            .new_project_engine_id
            .clone()
            .ok_or_else(|| HubError::message("No Source Engine selected for new project"))?;
        self.require_engine(&engine_id)?;
        self.config.active_engine_id = Some(engine_id.clone());
        self.sync_settings_from_active_engine();
        if let Err(error) = self.ensure_editor_available(ui) {
            let detail = error.to_string();
            self.record_editor_launch_failure(
                display_name.clone(),
                detail,
                Vec::new(),
                "Build the editor/runtime payload or fix Source Engine settings before creating the project",
            )?;
            return Err(error);
        }
        let command = EditorLaunchCommand::from_preferred_engine(
            self.staged_engine_dir(),
            EditorLaunchRequest::CreateProject(request),
        );
        let command_line = command.command_line();
        let child = match launch_editor(&command) {
            Ok(child) => child,
            Err(error) => {
                let detail = error.to_string();
                self.record_editor_launch_failure(
                    display_name.clone(),
                    detail,
                    command_line,
                    "Verify the staged zircon_editor executable exists and the target project location is writable",
                )?;
                return Err(error);
            }
        };
        let process_id = child.id();
        self.remember_project_metadata_for_path(
            &root,
            Some(engine_id),
            Some(self.selected_template_id.clone()),
        );
        self.remember_project(RecentProject::with_now(display_name.clone(), root))?;
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::OpenEditor,
            status: HubActionStatus::Success,
            target: display_name.clone(),
            detail: format!("Started process {process_id}"),
            log_excerpt: String::new(),
            recovery: None,
            process_id: Some(process_id),
            command_line,
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        self.task_status =
            TaskStatus::success("Editor launched", "Creating renderable empty project")
                .with_operation(TaskOperationKind::Project, display_name);
        Ok(())
    }

    pub(super) fn select_new_project_engine_by_id(
        &mut self,
        engine_id: &str,
    ) -> Result<(), HubError> {
        self.require_engine(engine_id)?;
        self.new_project_engine_id = Some(engine_id.to_string());
        Ok(())
    }

    pub(super) fn select_project_detail_engine_by_id(
        &mut self,
        engine_id: &str,
    ) -> Result<(), HubError> {
        self.require_engine(engine_id)?;
        let path = self.selected_project_path_required()?;
        let active_engine_before = self.config.active_engine_id.clone();
        metadata_for_path_mut(&mut self.config.project_metadata, &path).engine_id =
            Some(engine_id.to_string());
        self.config.active_engine_id = Some(engine_id.to_string());
        self.sync_settings_from_active_engine();
        self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());
        self.refresh_project_context_views(
            true,
            self.config.active_engine_id != active_engine_before,
        )?;
        self.persist_hub_config()?;
        self.task_status =
            TaskStatus::success("Project engine updated", self.engine_label(engine_id))
                .with_operation(
                    TaskOperationKind::Project,
                    path.to_string_lossy().into_owned(),
                );
        Ok(())
    }

    pub(super) fn toggle_selected_project_pin(&mut self) -> Result<(), HubError> {
        let path = self.selected_project_path_required()?;
        let metadata = metadata_for_path_mut(&mut self.config.project_metadata, &path);
        metadata.pinned = !metadata.pinned;
        let pinned = metadata.pinned;
        prune_empty_metadata(&mut self.config.project_metadata);
        self.persist_hub_config()?;
        self.task_status = TaskStatus::success(
            if pinned {
                "Project pinned"
            } else {
                "Project unpinned"
            },
            path.to_string_lossy().into_owned(),
        );
        Ok(())
    }

    pub(super) fn remove_selected_project_from_hub(&mut self) -> Result<(), HubError> {
        let path = self.selected_project_path_required()?;
        self.remove_project_from_hub_path(&path);
        self.project_subpage = ProjectSubpage::ProjectBrowser;
        self.refresh_selected_project_scoped_views()?;
        self.persist()?;
        self.task_status = TaskStatus::success(
            "Project removed from Hub",
            path.to_string_lossy().into_owned(),
        );
        Ok(())
    }

    pub(super) fn request_delete_selected_project(&mut self) -> Result<(), HubError> {
        let path = self.selected_project_path_required()?;
        if !cfg!(target_os = "windows") {
            return Err(HubError::message(
                "Project deletion is only available on Windows in this Hub build",
            ));
        }
        if !path.exists() {
            return Err(HubError::message(format!(
                "Project path is missing: {}",
                path.to_string_lossy()
            )));
        }
        self.pending_delete_project_path = Some(path.clone());
        self.task_status = TaskStatus::warning(
            "Confirm project deletion",
            "The project will be moved to the Windows Recycle Bin",
            "Confirm only if the selected project directory should leave the Hub and filesystem",
        );
        Ok(())
    }

    pub(super) fn cancel_delete_project(&mut self) {
        self.pending_delete_project_path = None;
        self.task_status =
            TaskStatus::success("Delete cancelled", "Project files were not changed");
    }

    pub(super) fn confirm_delete_project(&mut self) -> Result<(), HubError> {
        let path = self
            .pending_delete_project_path
            .clone()
            .ok_or_else(|| HubError::message("No project deletion is pending"))?;
        recycle_delete_project(path.clone())?;
        self.remove_project_from_hub_path(&path);
        self.pending_delete_project_path = None;
        self.project_subpage = ProjectSubpage::ProjectBrowser;
        self.refresh_selected_project_scoped_views()?;
        self.persist()?;
        self.task_status = TaskStatus::success(
            "Project moved to Recycle Bin",
            path.to_string_lossy().into_owned(),
        );
        Ok(())
    }

    pub(super) fn activate_project_engine_for_path(&mut self, path: &Path) {
        let active_engine_before = self.config.active_engine_id.clone();
        let Some(engine_id) = metadata_for_path(&self.config.project_metadata, path)
            .and_then(|metadata| metadata.engine_id.clone())
        else {
            return;
        };
        if self
            .config
            .engines
            .iter()
            .any(|engine| engine.id == engine_id)
        {
            self.config.active_engine_id = Some(engine_id);
            self.sync_settings_from_active_engine();
            self.sync_new_project_engine_after_active_engine_change(
                active_engine_before.as_deref(),
            );
        }
    }

    pub(super) fn remember_project_metadata_for_path(
        &mut self,
        path: &Path,
        engine_id: Option<String>,
        template_id: Option<String>,
    ) {
        let metadata = metadata_for_path_mut(&mut self.config.project_metadata, path);
        if let Some(engine_id) = engine_id {
            metadata.engine_id = Some(engine_id);
        }
        if let Some(template_id) = template_id {
            metadata.last_selected_template = Some(template_id);
        }
        prune_empty_metadata(&mut self.config.project_metadata);
    }

    pub(super) fn remember_project(&mut self, project: RecentProject) -> Result<(), HubError> {
        let last_project_path = project.path.clone();
        let active_engine_before = self.config.active_engine_id.clone();
        self.selected_project_path = Some(last_project_path.clone());
        self.config.recent_projects = crate::projects::merge_recent_projects(
            std::iter::once(project),
            self.config.recent_projects.clone(),
        );
        self.activate_project_engine_for_path(&last_project_path);
        self.refresh_project_context_views(
            true,
            self.config.active_engine_id != active_engine_before,
        )?;
        self.persist_with_last_project(Some(&last_project_path))
    }

    fn refresh_selected_project_scoped_views(&mut self) -> Result<(), HubError> {
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()
    }

    fn refresh_project_context_views(
        &mut self,
        selected_project_changed: bool,
        active_engine_changed: bool,
    ) -> Result<(), HubError> {
        if active_engine_changed {
            self.refresh_source_scoped_views()
        } else if selected_project_changed {
            self.refresh_selected_project_scoped_views()
        } else {
            Ok(())
        }
    }

    pub(super) fn selected_recent_project(&mut self) -> Option<RecentProject> {
        let selected_path = self.selected_project_path.clone()?;
        let project = self
            .config
            .recent_projects
            .iter()
            .find(|project| project_paths_match(&project.path, &selected_path))
            .cloned();
        if let Some(project) = &project {
            if self
                .selected_project_path
                .as_ref()
                .is_some_and(|selected| selected != &project.path)
            {
                self.selected_project_path = Some(project.path.clone());
            }
        } else {
            self.selected_project_path = None;
        }
        project
    }

    pub(super) fn selected_or_latest_recent_project(&mut self) -> Option<RecentProject> {
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

    pub(super) fn selected_or_latest_recent_project_for_action(
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

    pub(super) fn selected_or_latest_recent_project_with_engine_for_action(
        &mut self,
    ) -> Result<RecentProject, HubError> {
        let project = self.selected_or_latest_recent_project_for_named_action(
            "No recent project is available to build",
            "Selected project is no longer available to build",
        )?;
        self.require_project_bound_engine(&project)?;
        Ok(project)
    }

    fn selected_or_latest_recent_project_for_named_action(
        &mut self,
        missing_project_message: &str,
        stale_project_message: &str,
    ) -> Result<RecentProject, HubError> {
        let had_selected_project = self.selected_project_path.is_some();
        let Some(project) = self.selected_or_latest_recent_project_for_action()? else {
            return Err(HubError::message(if had_selected_project {
                stale_project_message
            } else {
                missing_project_message
            }));
        };
        Ok(project)
    }

    pub(super) fn selected_project_for_named_action(
        &mut self,
        missing_project_message: &str,
        stale_project_message: &str,
    ) -> Result<RecentProject, HubError> {
        let selected_before = self.selected_project_path.clone();
        let active_engine_before = self.config.active_engine_id.clone();
        let had_selected_project = self.selected_project_path.is_some();
        let Some(project) = self.selected_recent_project() else {
            let selected_project_changed = selected_project_path_changed(
                selected_before.as_deref(),
                self.selected_project_path.as_deref(),
            );
            self.refresh_project_context_views(selected_project_changed, false)?;
            return Err(HubError::message(if had_selected_project {
                stale_project_message
            } else {
                missing_project_message
            }));
        };
        self.activate_project_engine_for_path(&project.path);
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

    pub(super) fn selected_project_with_engine_for_named_action(
        &mut self,
        missing_project_message: &str,
        stale_project_message: &str,
    ) -> Result<RecentProject, HubError> {
        let project =
            self.selected_project_for_named_action(missing_project_message, stale_project_message)?;
        self.require_project_bound_engine(&project)?;
        Ok(project)
    }

    fn require_project_bound_engine(&self, project: &RecentProject) -> Result<(), HubError> {
        let Some(engine_id) = metadata_for_path(&self.config.project_metadata, &project.path)
            .and_then(|metadata| metadata.engine_id.as_deref())
        else {
            return Err(HubError::message(format!(
                "Project has no bound Source Engine: {}",
                recent_project_display_name(project)
            )));
        };
        if self
            .config
            .engines
            .iter()
            .any(|engine| engine.id == engine_id)
        {
            return Ok(());
        }
        Err(HubError::message(format!(
            "Project bound Source Engine is unavailable: {} -> {}",
            recent_project_display_name(project),
            engine_id
        )))
    }

    fn install_package_for_project(
        &mut self,
        project_name: String,
        package_report: ProjectPackageReport,
    ) -> Result<crate::projects::DeviceInstallReport, HubError> {
        let package_dir = package_report.package_dir;
        let install_request = DeviceInstallRequest::new(
            package_dir.clone(),
            self.config.settings.default_device_install_dir.clone(),
        );
        match install_package_to_device(&install_request) {
            Ok(report) => Ok(report),
            Err(error) => {
                let detail = error.to_string();
                self.record_project_action_failure(
                    HubActionKind::InstallProject,
                    project_name,
                    detail,
                    "Check the package output and configured local device install directory before retrying",
                    Some(self.config.settings.default_device_install_dir.clone()),
                )?;
                Err(error)
            }
        }
    }

    fn record_editor_launch_failure(
        &mut self,
        target: String,
        detail: String,
        command_line: Vec<String>,
        recovery: &str,
    ) -> Result<(), HubError> {
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::OpenEditor,
            status: HubActionStatus::Failed,
            target: target.clone(),
            detail: detail.clone(),
            log_excerpt: String::new(),
            recovery: Some(recovery.to_string()),
            process_id: None,
            command_line,
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        self.set_action_failure_status(HubActionKind::OpenEditor, target, detail, recovery);
        Ok(())
    }

    pub(super) fn record_project_action_failure(
        &mut self,
        action: HubActionKind,
        target: String,
        detail: String,
        recovery: &str,
        output_dir: Option<PathBuf>,
    ) -> Result<(), HubError> {
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action,
            status: HubActionStatus::Failed,
            target: target.clone(),
            detail: detail.clone(),
            log_excerpt: String::new(),
            recovery: Some(recovery.to_string()),
            process_id: None,
            command_line: Vec::new(),
            output_dir,
        })?;
        self.set_action_failure_status(action, target, detail, recovery);
        Ok(())
    }

    pub(super) fn action_target_for_project_failure(&self) -> String {
        self.selected_project_path
            .as_ref()
            .map(|path| path.to_string_lossy().into_owned())
            .unwrap_or_else(|| "Project".to_string())
    }

    pub(super) fn selected_project_label(&mut self) -> String {
        self.selected_recent_project()
            .map(|project| recent_project_display_name(&project))
            .unwrap_or_else(|| "Project".to_string())
    }

    pub(super) fn selected_project_path_required(&mut self) -> Result<PathBuf, HubError> {
        self.selected_recent_project()
            .map(|project| project.path)
            .ok_or_else(|| HubError::message("No project is selected"))
    }

    pub(super) fn remove_project_from_hub_path(&mut self, path: &Path) {
        self.config
            .recent_projects
            .retain(|project| !project_paths_match(&project.path, path));
        self.config
            .project_metadata
            .remove(&project_metadata_key(path));
        if self
            .pending_delete_project_path
            .as_ref()
            .is_some_and(|pending| project_paths_match(pending, path))
        {
            self.pending_delete_project_path = None;
        }
        if self
            .selected_project_path
            .as_ref()
            .is_some_and(|selected| project_paths_match(selected, path))
        {
            self.selected_project_path = None;
        }
    }

    fn require_engine(&self, engine_id: &str) -> Result<(), HubError> {
        if self
            .config
            .engines
            .iter()
            .any(|engine| engine.id == engine_id)
        {
            return Ok(());
        }
        Err(HubError::message(format!(
            "Unknown source engine: {engine_id}"
        )))
    }

    fn engine_label(&self, engine_id: &str) -> String {
        self.config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
            .map(|engine| engine.display_name.clone())
            .unwrap_or_else(|| engine_id.to_string())
    }
}

fn selected_project_path_changed(before: Option<&Path>, after: Option<&Path>) -> bool {
    match (before, after) {
        (Some(before), Some(after)) => !project_paths_match(before, after),
        (None, None) => false,
        _ => true,
    }
}

fn recent_project_display_name(project: &RecentProject) -> String {
    if project.display_name.trim().is_empty() {
        return project
            .path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Zircon Project")
            .to_string();
    }
    project.display_name.clone()
}
