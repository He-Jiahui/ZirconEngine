use std::path::{Path, PathBuf};

use crate::error::HubError;
use crate::process::{launch_editor, EditorLaunchCommand, EditorLaunchRequest};
use crate::projects::{
    install_package_to_device, metadata_for_path, metadata_for_path_mut, project_metadata_key,
    prune_empty_metadata, recycle_delete_project, validate_project_root, CreateProjectRequest,
    DeviceInstallRequest, ProjectPackageReport, ProjectPackageRequest, ProjectTemplate,
    ProjectValidation, RecentProject,
};
use crate::state::{
    ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskStatus,
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
            .find(|project| project.path == path)
            .cloned()
        else {
            return Err(HubError::message(format!(
                "Unknown recent project: {project_path}"
            )));
        };
        self.selected_project_path = Some(project.path.clone());
        self.task_status = TaskStatus {
            label: "Project selected".to_string(),
            detail: recent_project_display_name(&project),
            running: false,
        };
        Ok(())
    }

    pub(super) fn view_all_projects(&mut self) {
        self.search_query.clear();
        self.project_filter = ProjectFilterMode::All;
        self.project_view_mode = ProjectViewMode::List;
        self.project_subpage = ProjectSubpage::ProjectBrowser;
        self.task_status = TaskStatus {
            label: "All projects".to_string(),
            detail: "Showing all recent projects".to_string(),
            running: false,
        };
    }

    pub(super) fn set_project_filter_by_id(&mut self, filter_id: &str) -> Result<(), HubError> {
        let Some(filter) = ProjectFilterMode::from_id(filter_id) else {
            return Err(HubError::message(format!(
                "Unknown project filter mode: {filter_id}"
            )));
        };
        self.project_filter = filter;
        self.task_status = TaskStatus {
            label: "Projects filtered".to_string(),
            detail: format!("Showing {}", self.project_filter.label()),
            running: false,
        };
        Ok(())
    }

    pub(super) fn set_project_sort_by_id(&mut self, sort_id: &str) -> Result<(), HubError> {
        let Some(sort) = ProjectSortMode::from_id(sort_id) else {
            return Err(HubError::message(format!(
                "Unknown project sort mode: {sort_id}"
            )));
        };
        self.project_sort = sort;
        self.task_status = TaskStatus {
            label: "Projects sorted".to_string(),
            detail: format!("Sorting by {}", self.project_sort.label()),
            running: false,
        };
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
            .find(|project| project.path == project_path)
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
            return Err(HubError::message("Project path is required"));
        }
        if validate_project_root(&project_path) != ProjectValidation::Valid {
            return Err(HubError::message(format!(
                "Project root is not valid: {}",
                project_path.to_string_lossy()
            )));
        }
        let display_name = display_name_hint.unwrap_or_else(|| {
            project_path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("Zircon Project")
                .to_string()
        });
        self.activate_project_engine_for_path(&project_path);
        self.ensure_editor_available(ui)?;
        let command = EditorLaunchCommand::from_preferred_engine(
            self.staged_engine_dir(),
            EditorLaunchRequest::OpenProject {
                project_path: project_path.clone(),
            },
        );
        launch_editor(&command)?;
        self.remember_project_metadata_for_path(
            &project_path,
            self.config.active_engine_id.clone(),
            None,
        );
        self.remember_project(RecentProject::with_now(display_name.clone(), project_path))?;
        self.task_status = TaskStatus {
            label: "Editor launched".to_string(),
            detail: format!("Opening {display_name}"),
            running: false,
        };
        Ok(())
    }

    pub(super) fn install_recent_project_to_device(
        &mut self,
        ui: &HubWindow,
    ) -> Result<(), HubError> {
        let package_report = self.package_recent_project_to_output(ui)?;
        let project_name = self.selected_project_label();
        let install_report = install_package_to_device(&DeviceInstallRequest::new(
            package_report.package_dir,
            self.config.settings.default_device_install_dir.clone(),
        ))?;
        self.task_status = TaskStatus {
            label: "Installed to device".to_string(),
            detail: format!(
                "{} -> {} ({} files)",
                project_name,
                install_report.install_dir.to_string_lossy(),
                install_report.files_copied
            ),
            running: false,
        };
        Ok(())
    }

    pub(super) fn package_recent_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        let report = self.package_recent_project_to_output(ui)?;
        let project_name = self.selected_project_label();
        self.task_status = TaskStatus {
            label: "Package created".to_string(),
            detail: format!(
                "{} -> {} ({} files)",
                project_name,
                report.package_dir.to_string_lossy(),
                report.files_copied
            ),
            running: false,
        };
        Ok(())
    }

    pub(super) fn package_recent_project_to_output(
        &mut self,
        ui: &HubWindow,
    ) -> Result<ProjectPackageReport, HubError> {
        self.sync_from_ui(ui);
        let Some(project) = self.selected_or_latest_recent_project() else {
            return Err(HubError::message(
                "No recent project is available to package",
            ));
        };
        if validate_project_root(&project.path) != ProjectValidation::Valid {
            return Err(HubError::message(format!(
                "Project root is not valid: {}",
                project.path.to_string_lossy()
            )));
        }
        let display_name = recent_project_display_name(&project);
        let request = ProjectPackageRequest::new(
            display_name.clone(),
            project.path.clone(),
            self.config.settings.default_build_output_dir.clone(),
        );
        crate::projects::package_project(&request)
    }

    pub(super) fn open_selected_project_or_editor(
        &mut self,
        ui: &HubWindow,
    ) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let Some(project) = self.selected_recent_project() else {
            return self.launch_editor_without_project(ui);
        };
        let display_name = recent_project_display_name(&project);
        self.open_project_path(ui, project.path, Some(display_name))
    }

    pub(super) fn create_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let request = CreateProjectRequest::new(
            ui.get_project_name().to_string(),
            PathBuf::from(ui.get_project_location().to_string()),
            self.selected_template_for_create()?,
        );
        if request.project_name.trim().is_empty() {
            return Err(HubError::message("Project name is required"));
        }
        if request.location.as_os_str().is_empty() {
            return Err(HubError::message("Project location is required"));
        }
        let root = request.location.join(&request.project_name);
        let display_name = request.project_name.clone();
        let engine_id = self.new_project_engine_id.clone();
        if let Some(engine_id) = engine_id.clone() {
            self.config.active_engine_id = Some(engine_id);
            self.sync_settings_from_active_engine();
        }
        self.ensure_editor_available(ui)?;
        let command = EditorLaunchCommand::from_preferred_engine(
            self.staged_engine_dir(),
            EditorLaunchRequest::CreateProject(request),
        );
        launch_editor(&command)?;
        self.remember_project_metadata_for_path(
            &root,
            engine_id,
            Some(self.selected_template_id.clone()),
        );
        self.remember_project(RecentProject::with_now(display_name, root))?;
        self.task_status = TaskStatus {
            label: "Editor launched".to_string(),
            detail: "Creating renderable empty project".to_string(),
            running: false,
        };
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
        metadata_for_path_mut(&mut self.config.project_metadata, &path).engine_id =
            Some(engine_id.to_string());
        self.persist_hub_config()?;
        self.task_status = TaskStatus {
            label: "Project engine updated".to_string(),
            detail: self.engine_label(engine_id),
            running: false,
        };
        Ok(())
    }

    pub(super) fn toggle_selected_project_pin(&mut self) -> Result<(), HubError> {
        let path = self.selected_project_path_required()?;
        let metadata = metadata_for_path_mut(&mut self.config.project_metadata, &path);
        metadata.pinned = !metadata.pinned;
        let pinned = metadata.pinned;
        prune_empty_metadata(&mut self.config.project_metadata);
        self.persist_hub_config()?;
        self.task_status = TaskStatus {
            label: if pinned {
                "Project pinned".to_string()
            } else {
                "Project unpinned".to_string()
            },
            detail: path.to_string_lossy().into_owned(),
            running: false,
        };
        Ok(())
    }

    pub(super) fn remove_selected_project_from_hub(&mut self) -> Result<(), HubError> {
        let path = self.selected_project_path_required()?;
        self.remove_project_from_hub_path(&path);
        self.project_subpage = ProjectSubpage::ProjectBrowser;
        self.persist()?;
        self.task_status = TaskStatus {
            label: "Project removed from Hub".to_string(),
            detail: path.to_string_lossy().into_owned(),
            running: false,
        };
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
        self.task_status = TaskStatus {
            label: "Confirm project deletion".to_string(),
            detail: "The project will be moved to the Windows Recycle Bin".to_string(),
            running: false,
        };
        Ok(())
    }

    pub(super) fn cancel_delete_project(&mut self) {
        self.pending_delete_project_path = None;
        self.task_status = TaskStatus {
            label: "Delete cancelled".to_string(),
            detail: "Project files were not changed".to_string(),
            running: false,
        };
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
        self.persist()?;
        self.task_status = TaskStatus {
            label: "Project moved to Recycle Bin".to_string(),
            detail: path.to_string_lossy().into_owned(),
            running: false,
        };
        Ok(())
    }

    pub(super) fn activate_project_engine_for_path(&mut self, path: &Path) {
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
        self.selected_project_path = Some(last_project_path.clone());
        self.config.recent_projects = crate::projects::merge_recent_projects(
            std::iter::once(project),
            self.config.recent_projects.clone(),
        );
        self.refresh_asset_catalog()?;
        self.persist_with_last_project(Some(&last_project_path))
    }

    pub(super) fn selected_recent_project(&mut self) -> Option<RecentProject> {
        let selected_path = self.selected_project_path.clone()?;
        let project = self
            .config
            .recent_projects
            .iter()
            .find(|project| project.path == selected_path)
            .cloned();
        if project.is_none() {
            self.selected_project_path = None;
        }
        project
    }

    pub(super) fn selected_or_latest_recent_project(&mut self) -> Option<RecentProject> {
        if let Some(project) = self.selected_recent_project() {
            return Some(project);
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
            .retain(|project| project.path != path);
        self.config
            .project_metadata
            .remove(&project_metadata_key(path));
        if self
            .selected_project_path
            .as_ref()
            .is_some_and(|selected| selected == path)
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
