use std::{
    collections::VecDeque,
    env,
    path::{Path, PathBuf},
};

mod action_targets;
mod action_tasks;
mod build_actions;
mod editor_launch_actions;
mod learn_actions;
mod new_project_actions;
mod output_actions;
mod project_actions;
mod project_delivery_actions;
mod quick_actions;
mod scoped_views;
mod settings_actions;

use crate::assets::AssetCatalogEntry;
use crate::engines::{
    ensure_active_source_engine, prune_project_engine_bindings, same_source_engine_path,
    source_engine_display_name, source_engine_id, upsert_source_engine, SourceEngineInstall,
};
use crate::error::HubError;
use crate::learn::LearnCatalogEntry;
use crate::plugins::PluginCatalogEntry;
use crate::projects::{
    load_editor_recent_project_session, merge_recent_projects, metadata_for_path,
    project_metadata_key, project_paths_match, save_editor_recent_projects,
    save_editor_recent_projects_with_last_project, RecentProject,
};
use crate::settings::{
    default_hub_config_path, editor_config_path, HubConfig, HubRuntimeState, HubSettings,
};
use crate::state::{
    HubPage, HubSnapshot, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode,
    TaskOperationKind, TaskStatus,
};
use crate::team::TeamOverview;

use super::action_request::{HubAction, HubActionRequest};
use super::view_model::{HubSettingsPayload, HubViewModel};

const VISUAL_TASK_STATE_ENV: &str = "ZIRCON_HUB_VISUAL_TASK_STATE";

pub(super) struct HubRuntimeSession {
    config_path: PathBuf,
    editor_config_path: PathBuf,
    config: HubConfig,
    settings_draft: HubSettings,
    selected_page: HubPage,
    project_filter: ProjectFilterMode,
    project_sort: ProjectSortMode,
    project_view_mode: ProjectViewMode,
    project_subpage: ProjectSubpage,
    search_query: String,
    selected_project_path: Option<PathBuf>,
    new_project_name: String,
    selected_template_id: String,
    new_project_location: PathBuf,
    new_project_engine_id: Option<String>,
    pending_delete_project_path: Option<PathBuf>,
    task_status: TaskStatus,
    background_worker_active: bool,
    background_action_queue: VecDeque<HubActionRequest>,
    asset_catalog: Vec<AssetCatalogEntry>,
    learn_catalog: Vec<LearnCatalogEntry>,
    plugin_catalog: Vec<PluginCatalogEntry>,
    team_overview: TeamOverview,
}

impl HubRuntimeSession {
    pub(super) fn load() -> Result<Self, HubError> {
        Self::load_from_paths(default_hub_config_path(), editor_config_path())
    }

    pub(super) fn load_from_paths(
        config_path: PathBuf,
        editor_config_path: PathBuf,
    ) -> Result<Self, HubError> {
        let mut config = HubConfig::load(&config_path)?;
        let editor_recent = load_editor_recent_project_session(&editor_config_path)?;
        let last_project_path = editor_recent.last_project_path;
        config.recent_projects =
            merge_recent_projects(config.recent_projects, editor_recent.recent_projects);
        config.repair_registries();

        let runtime_state = config.runtime.clone();
        let selected_project_path = startup_selected_project_path(
            runtime_state.selected_project_path.as_deref(),
            last_project_path.as_deref(),
            &config.recent_projects,
        );

        let settings_draft = config.settings.clone();
        let mut session = Self {
            config_path,
            editor_config_path,
            config,
            settings_draft,
            selected_page: runtime_state.selected_page,
            project_filter: runtime_state.project_filter,
            project_sort: runtime_state.project_sort,
            project_view_mode: runtime_state.project_view_mode,
            project_subpage: runtime_state.project_subpage,
            search_query: runtime_state.search_query,
            selected_project_path,
            new_project_name: runtime_state.new_project_name,
            selected_template_id: runtime_state.selected_template_id,
            new_project_location: runtime_state.new_project_location,
            new_project_engine_id: runtime_state.new_project_engine_id,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            background_worker_active: false,
            background_action_queue: VecDeque::new(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
        };
        session.register_source_engine_from_settings();
        session.prune_stale_project_engine_bindings();
        session.config.repair_registries();
        if let Some(path) = session.selected_project_path.clone() {
            session.activate_project_engine_for_path(&path);
        }
        session.ensure_new_project_engine_selection();
        session.refresh_source_scoped_views()?;
        session.apply_visual_task_state_override_from_env();
        session.persist()?;
        Ok(session)
    }

    pub(super) fn view_model(&self) -> HubViewModel {
        HubViewModel::from_snapshot(&self.snapshot())
    }

    pub(super) fn apply_action(
        &mut self,
        request: HubActionRequest,
    ) -> Result<HubViewModel, HubError> {
        match request.parse()? {
            HubAction::ShowPage { target_id } => self.select_page_by_id(&target_id)?,
            HubAction::ShowProjectSubpage { target_id } => {
                self.show_project_subpage_by_id(&target_id)?
            }
            HubAction::SearchProjects { query } => self.search_projects(&query),
            HubAction::SetProjectFilter { target_id } => {
                self.set_project_filter_by_id(&target_id)?
            }
            HubAction::SetProjectSort { target_id } => self.set_project_sort_by_id(&target_id)?,
            HubAction::SetProjectViewMode { target_id } => {
                self.set_project_view_mode_by_id(&target_id)?
            }
            HubAction::SelectProject { target_id } => self.select_project_target(&target_id)?,
            HubAction::OpenProjectDetail { target_id } => self.open_project_detail(&target_id)?,
            HubAction::ViewAllProjects => self.view_all_projects(),
            HubAction::NewProject => {
                self.show_project_subpage_by_id(ProjectSubpage::NewProject.id())?
            }
            HubAction::UpdateNewProjectDraft { payload } => {
                self.update_new_project_draft(payload)?
            }
            HubAction::SelectEngine { target_id } => self.select_engine_by_id(&target_id)?,
            HubAction::SaveSettings { payload } => self.save_settings_from_action(payload)?,
            HubAction::BrowseSettingsFolder { target_id, payload } => {
                self.browse_settings_folder(target_id.as_deref(), payload)?
            }
            HubAction::CreateProject { payload } => self.create_project_from_payload(payload)?,
            HubAction::ImportProject { target_id, payload } => {
                self.import_project_from_action(target_id.as_deref(), payload)?
            }
            HubAction::PinProject { target_id, payload } => {
                self.set_project_pinned(target_id.as_deref(), payload.as_ref(), true)?
            }
            HubAction::UnpinProject { target_id, payload } => {
                self.set_project_pinned(target_id.as_deref(), payload.as_ref(), false)?
            }
            HubAction::RemoveFromHub { target_id, payload } => {
                self.remove_project_from_hub(target_id.as_deref(), payload.as_ref())?
            }
            HubAction::RequestDelete { target_id, payload } => {
                self.request_project_delete(target_id.as_deref(), payload.as_ref())?
            }
            HubAction::CancelDelete { target_id, payload } => {
                self.cancel_project_delete(target_id.as_deref(), payload.as_ref())?
            }
            HubAction::ConfirmDelete { target_id, payload } => {
                self.confirm_project_delete(target_id.as_deref(), payload.as_ref())?
            }
            HubAction::OpenResource { target_id, payload } => {
                self.open_learn_resource(target_id.as_deref(), payload)?
            }
            HubAction::OpenOutputFolder { target_id, payload } => {
                self.open_output_folder(target_id.as_deref(), payload)?
            }
            HubAction::BuildProject { target_id, payload } => {
                self.apply_action_project_target(
                    target_id.as_deref(),
                    payload.as_ref(),
                    "build-project",
                )?;
                self.build_selected_project_engine()?
            }
            HubAction::PackageProject { target_id, payload } => {
                self.apply_action_project_target(
                    target_id.as_deref(),
                    payload.as_ref(),
                    "package-project",
                )?;
                self.package_recent_project()?
            }
            HubAction::InstallDevice { target_id, payload } => {
                self.apply_action_project_target(
                    target_id.as_deref(),
                    payload.as_ref(),
                    "install-device",
                )?;
                self.install_recent_project_to_device()?
            }
            HubAction::OpenEditor { target_id, payload } => {
                self.apply_action_project_target(
                    target_id.as_deref(),
                    payload.as_ref(),
                    "open-editor",
                )?;
                self.open_selected_project_or_editor()?
            }
        }

        Ok(self.view_model())
    }

    fn snapshot(&self) -> HubSnapshot {
        HubSnapshot {
            selected_page: self.selected_page,
            project_filter: self.project_filter,
            project_sort: self.project_sort,
            project_view_mode: self.project_view_mode,
            project_subpage: self.project_subpage,
            search_query: self.search_query.clone(),
            selected_project_path: self.selected_project_path.clone(),
            new_project_name: self.new_project_name.clone(),
            selected_template_id: self.selected_template_id.clone(),
            new_project_location: self.new_project_location.clone(),
            new_project_engine_id: self.new_project_engine_id.clone(),
            pending_delete_project_path: self.pending_delete_project_path.clone(),
            task_status: self.task_status.clone(),
            recent_projects: self.config.recent_projects.clone(),
            project_metadata: self.config.project_metadata.clone(),
            assets: self.asset_catalog.clone(),
            learn_resources: self.learn_catalog.clone(),
            plugins: self.plugin_catalog.clone(),
            team: self.team_overview.clone(),
            action_history: self.config.action_history.clone(),
            engines: self.config.engines.clone(),
            active_engine_id: self.config.active_engine_id.clone(),
            settings: self.config.settings.clone(),
            settings_draft: self.settings_draft.clone(),
        }
    }

    fn select_page_by_id(&mut self, page_id: &str) -> Result<(), HubError> {
        let Some(page) = HubPage::from_id(page_id) else {
            return Err(HubError::message(format!("Unknown Hub page: {page_id}")));
        };
        self.selected_page = page;
        self.persist_hub_config()
    }

    fn show_project_subpage_by_id(&mut self, subpage_id: &str) -> Result<(), HubError> {
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
        self.persist_hub_config()
    }

    fn search_projects(&mut self, query: &str) {
        self.search_query = query.to_string();
        let _ = self.persist_hub_config();
    }

    fn set_project_filter_by_id(&mut self, filter_id: &str) -> Result<(), HubError> {
        let Some(filter) = ProjectFilterMode::from_id(filter_id) else {
            return Err(HubError::message(format!(
                "Unknown project filter mode: {filter_id}"
            )));
        };
        self.project_filter = filter;
        self.task_status = TaskStatus::success(
            "Projects filtered",
            format!("Showing {}", self.project_filter.label()),
        )
        .with_operation(TaskOperationKind::Hub, "Projects");
        self.persist_hub_config()
    }

    fn set_project_sort_by_id(&mut self, sort_id: &str) -> Result<(), HubError> {
        let Some(sort) = ProjectSortMode::from_id(sort_id) else {
            return Err(HubError::message(format!(
                "Unknown project sort mode: {sort_id}"
            )));
        };
        self.project_sort = sort;
        self.task_status = TaskStatus::success(
            "Projects sorted",
            format!("Sorting by {}", self.project_sort.label()),
        )
        .with_operation(TaskOperationKind::Hub, "Projects");
        self.persist_hub_config()
    }

    fn set_project_view_mode_by_id(&mut self, mode_id: &str) -> Result<(), HubError> {
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
        self.persist_hub_config()
    }

    fn select_project_target(&mut self, target: &str) -> Result<(), HubError> {
        let Some(project) = self.find_recent_project(target) else {
            return Err(HubError::message(format!(
                "Unknown recent project: {target}"
            )));
        };
        let active_engine_before = self.config.active_engine_id.clone();
        self.selected_project_path = Some(project.path.clone());
        self.activate_project_engine_for_path(&project.path);
        self.refresh_project_context_views(
            true,
            self.config.active_engine_id != active_engine_before,
        )?;
        let display_name = recent_project_display_name(&project);
        self.task_status = TaskStatus::success("Project selected", display_name.clone())
            .with_operation(TaskOperationKind::Project, display_name);
        self.persist_with_last_project(Some(&project.path))
    }

    fn open_project_detail(&mut self, target: &str) -> Result<(), HubError> {
        self.select_project_target(target)?;
        self.project_subpage = ProjectSubpage::ProjectDetail;
        self.project_view_mode = ProjectViewMode::List;
        self.pending_delete_project_path = None;
        self.persist_hub_config()
    }

    fn view_all_projects(&mut self) {
        self.search_query.clear();
        self.project_filter = ProjectFilterMode::All;
        self.project_view_mode = ProjectViewMode::List;
        self.project_subpage = ProjectSubpage::ProjectBrowser;
        self.task_status = TaskStatus::success("All projects", "Showing all recent projects")
            .with_operation(TaskOperationKind::Hub, "Projects");
        let _ = self.persist_hub_config();
    }

    fn select_engine_by_id(&mut self, engine_id: &str) -> Result<(), HubError> {
        let active_engine_before = self.config.active_engine_id.clone();
        let Some(engine) = self
            .config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
            .cloned()
        else {
            return Err(HubError::message(format!(
                "Unknown source engine: {engine_id}"
            )));
        };
        self.config.active_engine_id = Some(engine.id.clone());
        self.config.settings.default_source_dir = engine.source_dir.clone();
        self.config.settings.default_build_output_dir = engine.output_dir.clone();
        self.settings_draft = self.config.settings.clone();
        self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());
        self.refresh_source_scoped_views()?;
        self.persist_hub_config()?;
        self.task_status = TaskStatus::success("Engine selected", engine.display_name.clone())
            .with_operation(TaskOperationKind::SourceEngine, engine.display_name);
        Ok(())
    }

    fn save_settings(
        &mut self,
        settings_payload: Option<HubSettingsPayload>,
    ) -> Result<(), HubError> {
        if let Some(settings_payload) = settings_payload {
            let mut settings = self.config.settings.clone();
            if let Err(error) = settings_payload.apply_to(&mut settings) {
                self.record_settings_save_failure(error.to_string());
                return Ok(());
            }
            self.config.settings = settings;
        } else {
            self.config.settings = self.settings_draft.clone();
        }
        self.register_source_engine_from_settings();
        self.refresh_source_scoped_views()?;
        self.persist()?;
        self.settings_draft = self.config.settings.clone();
        self.task_status = TaskStatus::success(
            "Settings saved",
            self.config_path.to_string_lossy().into_owned(),
        )
        .with_operation(TaskOperationKind::Settings, "Hub settings");
        Ok(())
    }

    fn persist(&self) -> Result<(), HubError> {
        self.persist_with_last_project(None)
    }

    fn persist_hub_config(&self) -> Result<(), HubError> {
        let mut config = self.config.clone();
        config.runtime = self.runtime_state_for_config();
        config.save(&self.config_path)
    }

    fn persist_with_last_project(&self, last_project_path: Option<&Path>) -> Result<(), HubError> {
        self.persist_hub_config()?;
        match last_project_path {
            Some(path) => save_editor_recent_projects_with_last_project(
                &self.editor_config_path,
                &self.config.recent_projects,
                Some(path),
            )?,
            None => {
                save_editor_recent_projects(&self.editor_config_path, &self.config.recent_projects)?
            }
        }
        Ok(())
    }

    fn runtime_state_for_config(&self) -> HubRuntimeState {
        HubRuntimeState {
            selected_page: self.selected_page,
            project_subpage: self.project_subpage,
            project_filter: self.project_filter,
            project_sort: self.project_sort,
            project_view_mode: self.project_view_mode,
            search_query: self.search_query.clone(),
            selected_project_path: self.selected_project_path.clone(),
            new_project_name: self.new_project_name.clone(),
            selected_template_id: self.selected_template_id.clone(),
            new_project_location: self.new_project_location.clone(),
            new_project_engine_id: self.new_project_engine_id.clone(),
        }
    }

    fn apply_visual_task_state_override_from_env(&mut self) {
        let Ok(visual_state) = env::var(VISUAL_TASK_STATE_ENV) else {
            return;
        };

        match visual_state.trim().to_ascii_lowercase().as_str() {
            "loading" | "running" => {
                self.task_status = TaskStatus::running_operation(
                    "Loading Hub state",
                    "Refreshing projects, source engines, and build workflows",
                    TaskOperationKind::Hub,
                    "Visual verification",
                );
            }
            "error" => {
                self.task_status = TaskStatus::error(
                    "Action failed",
                    "Visual verification error state",
                    "Check the highlighted workflow target before retrying",
                )
                .with_operation(TaskOperationKind::Hub, "Visual verification");
            }
            "warning" => {
                self.task_status = TaskStatus::warning(
                    "Warning",
                    "Visual verification warning state",
                    "Review settings before continuing",
                )
                .with_operation(TaskOperationKind::Hub, "Visual verification");
            }
            "success" => {
                self.task_status =
                    TaskStatus::success("Success", "Visual verification success state")
                        .with_operation(TaskOperationKind::Hub, "Visual verification");
            }
            _ => {}
        }
    }

    fn register_source_engine_from_settings(&mut self) {
        let source_dir = self.config.settings.default_source_dir.clone();
        let output_dir = self.config.settings.default_build_output_dir.clone();
        if source_dir.as_os_str().is_empty() {
            return;
        }
        let active_engine_before = self.config.active_engine_id.clone();
        let engine_id = source_engine_id(&source_dir);
        let existing_index = self.config.engines.iter().position(|engine| {
            engine.id == engine_id || same_source_engine_path(&engine.source_dir, &source_dir)
        });
        let existing = existing_index.and_then(|index| self.config.engines.get(index).cloned());
        if let Some(existing) = &existing {
            if existing.id != engine_id {
                self.migrate_project_engine_metadata(&existing.id, &engine_id);
                self.config
                    .engines
                    .retain(|engine| engine.id != existing.id);
            }
        }
        let engine = SourceEngineInstall {
            id: engine_id.clone(),
            display_name: existing
                .as_ref()
                .map(|engine| engine.display_name.clone())
                .unwrap_or_else(|| source_engine_display_name(&source_dir)),
            source_dir,
            output_dir,
            last_build_unix_ms: existing
                .as_ref()
                .and_then(|engine| engine.last_build_unix_ms),
            build_history: existing
                .as_ref()
                .map(|engine| engine.build_history.clone())
                .unwrap_or_default(),
        };
        upsert_source_engine(&mut self.config.engines, engine);
        self.config.active_engine_id = Some(engine_id);
        ensure_active_source_engine(&self.config.engines, &mut self.config.active_engine_id);
        self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());
    }

    fn prune_stale_project_engine_bindings(&mut self) -> usize {
        prune_project_engine_bindings(&mut self.config.project_metadata, &self.config.engines)
    }

    fn migrate_project_engine_metadata(&mut self, old_engine_id: &str, new_engine_id: &str) {
        for metadata in self.config.project_metadata.values_mut() {
            if metadata.engine_id.as_deref() == Some(old_engine_id) {
                metadata.engine_id = Some(new_engine_id.to_string());
            }
        }
        if self.config.active_engine_id.as_deref() == Some(old_engine_id) {
            self.config.active_engine_id = Some(new_engine_id.to_string());
        }
        if self.new_project_engine_id.as_deref() == Some(old_engine_id) {
            self.new_project_engine_id = Some(new_engine_id.to_string());
        }
    }

    fn ensure_new_project_engine_selection(&mut self) {
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

    fn sync_new_project_engine_after_active_engine_change(
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

    fn activate_project_engine_for_path(&mut self, path: &Path) {
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

    fn sync_settings_from_active_engine(&mut self) {
        if let Some(engine) = self
            .config
            .active_engine_id
            .as_deref()
            .and_then(|id| self.config.engines.iter().find(|engine| engine.id == id))
            .or_else(|| self.config.engines.first())
        {
            self.config.settings.default_source_dir = engine.source_dir.clone();
            self.config.settings.default_build_output_dir = engine.output_dir.clone();
            self.settings_draft = self.config.settings.clone();
        }
    }

    fn selected_recent_project(&mut self) -> Option<RecentProject> {
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

    fn find_recent_project(&self, target: &str) -> Option<RecentProject> {
        let target = target.trim();
        let target_key = project_metadata_key(target);
        self.config
            .recent_projects
            .iter()
            .find(|project| {
                project_paths_match(&project.path, target)
                    || project_metadata_key(&project.path) == target_key
                    || recent_project_slug(project) == target
            })
            .cloned()
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
}

fn startup_selected_project_path(
    persisted_selected_project_path: Option<&Path>,
    last_project_path: Option<&Path>,
    recent_projects: &[RecentProject],
) -> Option<PathBuf> {
    if let Some(path) = persisted_selected_project_path {
        return Some(
            recent_projects
                .iter()
                .find(|project| project_paths_match(&project.path, path))
                .map(|project| project.path.clone())
                .unwrap_or_else(|| path.to_path_buf()),
        );
    }

    let last_project_path = last_project_path?;
    recent_projects
        .iter()
        .find(|project| project_paths_match(&project.path, last_project_path))
        .map(|project| project.path.clone())
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

fn recent_project_slug(project: &RecentProject) -> String {
    recent_project_display_name(project)
        .chars()
        .filter_map(|ch| {
            if ch.is_ascii_alphanumeric() {
                Some(ch.to_ascii_lowercase())
            } else if ch.is_whitespace() || ch == '-' || ch == '_' {
                Some('-')
            } else {
                None
            }
        })
        .collect::<String>()
        .split('-')
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("-")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::projects::{project_metadata_key, RecentProject};
    use crate::settings::{BuildProfile, HubConfig, HubLanguage};

    use super::*;

    fn temp_test_dir(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn startup_selection_preserves_persisted_stale_project_path() {
        let recent_projects = vec![RecentProject::new("Recent", "E:/Projects/Recent", 30)];

        let selected = startup_selected_project_path(
            Some(Path::new("E:/Projects/Missing")),
            Some(Path::new("E:/Projects/Recent")),
            &recent_projects,
        );

        assert_eq!(selected, Some(PathBuf::from("E:/Projects/Missing")));
    }

    #[test]
    fn load_from_paths_merges_repairs_registers_source_and_persists_runtime_state() {
        let temp = temp_test_dir("zircon-hub-tauri-runtime-load");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let project_path = temp.join("Game");
        let source_path = temp.join("ZirconEngine");
        fs::create_dir_all(&project_path).unwrap();
        fs::create_dir_all(&source_path).unwrap();

        let mut config = HubConfig::default();
        config.recent_projects = vec![RecentProject::new("Game", &project_path, 4)];
        config.project_metadata.insert(
            project_metadata_key(&project_path),
            crate::projects::ProjectMetadata {
                pinned: true,
                engine_id: Some("missing-engine".to_string()),
                last_selected_template: None,
            },
        );
        config.settings.default_source_dir = source_path.clone();
        config.settings.default_build_output_dir = temp.join("out");
        config.runtime.selected_project_path = Some(project_path.clone());
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            format!(
                r#"{{"editor.startup.session":{{"last_project_path":"{}","recent_projects":[]}}}}"#,
                project_path.to_string_lossy().replace('\\', "/")
            ),
        )
        .unwrap();

        let session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path.clone())
                .expect("Tauri runtime session should load and persist");

        assert_eq!(session.selected_project_path, Some(project_path.clone()));
        assert_eq!(session.config.engines.len(), 1);
        assert_eq!(
            session.config.active_engine_id.as_deref(),
            Some(source_engine_id(&source_path).as_str())
        );
        assert_eq!(
            session
                .config
                .project_metadata
                .get(&project_metadata_key(&project_path))
                .and_then(|metadata| metadata.engine_id.as_deref()),
            None
        );
        let saved = HubConfig::load(&config_path).unwrap();
        assert_eq!(saved.runtime.selected_project_path, Some(project_path));

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn save_settings_action_applies_typed_payload_and_refreshes_source_engine() {
        let temp = temp_test_dir("zircon-hub-tauri-save-settings-payload");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let source_path = temp.join("ZirconEngine");
        let build_output = temp.join("build-output");
        let device_install = temp.join("device-install");
        fs::create_dir_all(&source_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
                .expect("Tauri runtime session should load");

        let view_model = session
            .apply_action(HubActionRequest {
                action_id: "save-settings".to_string(),
                target_id: None,
                payload: Some(serde_json::json!({
                    "settings": {
                        "pythonPath": "py",
                        "cargoPath": "cargo",
                        "rustupPath": "rustup",
                        "defaultProjectDir": temp.join("projects").to_string_lossy(),
                        "defaultSourceDir": source_path.to_string_lossy(),
                        "defaultBuildOutputDir": build_output.to_string_lossy(),
                        "defaultDeviceInstallDir": device_install.to_string_lossy(),
                        "buildProfile": "release",
                        "jobs": 3,
                        "language": "English"
                    }
                })),
            })
            .expect("save-settings should accept typed settings payload");

        assert_eq!(session.config.settings.build_profile, BuildProfile::Release);
        assert_eq!(session.config.settings.jobs, 3);
        assert_eq!(session.config.settings.language, HubLanguage::English);
        assert_eq!(session.config.settings.default_source_dir, source_path);
        assert_eq!(
            session.config.settings.default_build_output_dir,
            build_output
        );
        assert_eq!(
            session.config.settings.default_device_install_dir,
            device_install
        );
        let expected_engine_id = source_engine_id(&source_path);
        assert_eq!(
            session.config.active_engine_id.as_deref(),
            Some(expected_engine_id.as_str())
        );
        let active_engine = session
            .config
            .engines
            .iter()
            .find(|engine| engine.id == expected_engine_id)
            .expect("payload Source Engine should be registered");
        assert_eq!(active_engine.source_dir, source_path);
        assert_eq!(active_engine.output_dir, build_output);
        assert_eq!(view_model.settings.language, "English");
        assert_eq!(view_model.task_summary.label, "Settings saved");

        let saved = HubConfig::load(&config_path).unwrap();
        assert_eq!(saved.settings.build_profile, BuildProfile::Release);
        assert_eq!(saved.settings.language, HubLanguage::English);

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn save_settings_refreshes_source_scoped_catalogs_in_returned_view_model() {
        let temp = temp_test_dir("zircon-hub-tauri-save-settings-catalogs");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let source_path = temp.join("ZirconEngine");
        let build_output = temp.join("build-output");
        let device_install = temp.join("device-install");
        let asset_path = source_path
            .join("zircon_editor")
            .join("assets")
            .join("icons")
            .join("source-settings-tool.svg");
        let plugin_manifest_path = source_path
            .join("zircon_plugins")
            .join("source_settings_tools")
            .join("plugin.toml");
        let learn_path = source_path
            .join("docs")
            .join("settings")
            .join("source-settings-refresh.md");
        fs::create_dir_all(asset_path.parent().unwrap()).unwrap();
        fs::write(&asset_path, "<svg></svg>").unwrap();
        fs::create_dir_all(plugin_manifest_path.parent().unwrap()).unwrap();
        fs::write(
            &plugin_manifest_path,
            r#"id = "source_settings_tools"
display_name = "Source Settings Tools"
description = "Source plugin loaded after settings save."
category = "editor"
maturity = "stable"
supported_targets = ["editor_host"]

[[modules]]
name = "source.settings"
kind = "editor"
"#,
        )
        .unwrap();
        fs::create_dir_all(learn_path.parent().unwrap()).unwrap();
        fs::write(
            &learn_path,
            "# Source Settings Refresh\n\nSource Engine docs loaded after settings save.\n",
        )
        .unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
                .expect("Tauri runtime session should load");

        let view_model = session
            .apply_action(HubActionRequest {
                action_id: "save-settings".to_string(),
                target_id: None,
                payload: Some(serde_json::json!({
                    "settings": {
                        "pythonPath": "py",
                        "cargoPath": "cargo",
                        "rustupPath": "rustup",
                        "defaultProjectDir": temp.join("projects").to_string_lossy(),
                        "defaultSourceDir": source_path.to_string_lossy(),
                        "defaultBuildOutputDir": build_output.to_string_lossy(),
                        "defaultDeviceInstallDir": device_install.to_string_lossy(),
                        "buildProfile": "debug",
                        "jobs": 2,
                        "language": "English"
                    }
                })),
            })
            .expect("save-settings should refresh source-scoped catalogs");

        let expected_engine_id = source_engine_id(&source_path);
        assert_eq!(
            view_model.active_source_engine_id.as_deref(),
            Some(expected_engine_id.as_str())
        );
        let asset_debug = view_model
            .assets
            .iter()
            .map(|asset| format!("{}:{}", asset.name, asset.source_key))
            .collect::<Vec<_>>();
        assert!(
            view_model.assets.iter().any(|asset| {
                asset.name == "source-settings-tool.svg" && asset.source_key == "engine"
            }),
            "assets should include refreshed Source Engine asset, got {asset_debug:?}"
        );
        let plugin_debug = view_model
            .plugins
            .iter()
            .map(|plugin| format!("{}:{}", plugin.id, plugin.scope_key))
            .collect::<Vec<_>>();
        assert!(
            view_model.plugins.iter().any(|plugin| {
                plugin.id == "source_settings_tools"
                    && plugin.scope_key == "engine"
                    && plugin.editor_scoped
            }),
            "plugins should include refreshed Source Engine plugin, got {plugin_debug:?}"
        );
        let learn_debug = view_model
            .learn_resources
            .iter()
            .map(|resource| format!("{}:{}", resource.title, resource.source_key))
            .collect::<Vec<_>>();
        assert!(
            view_model.learn_resources.iter().any(|resource| {
                resource.title == "Source Settings Refresh" && resource.source_key == "engine"
            }),
            "learn resources should include refreshed Source Engine doc, got {learn_debug:?}"
        );

        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn project_view_action_status_localizes_in_chinese_view_model() {
        let temp = temp_test_dir("zircon-hub-tauri-project-view-localized");
        let config_path = temp.join("hub.toml");
        let editor_config_path = temp.join("editor.json");
        let mut config = HubConfig::default();
        config.settings.language = HubLanguage::Chinese;
        config.save(&config_path).unwrap();
        fs::write(
            &editor_config_path,
            r#"{"editor.startup.session":{"recent_projects":[]}}"#,
        )
        .unwrap();
        let mut session =
            HubRuntimeSession::load_from_paths(config_path.clone(), editor_config_path)
                .expect("Tauri runtime session should load");

        let filter_model = session
            .apply_action(HubActionRequest {
                action_id: "set-project-filter".to_string(),
                target_id: Some("missing".to_string()),
                payload: None,
            })
            .expect("project filter action should return refreshed state");
        assert_eq!(filter_model.task_summary.label, "项目已筛选");
        assert_eq!(filter_model.task_summary.detail, "显示缺失项目");

        let sort_model = session
            .apply_action(HubActionRequest {
                action_id: "set-project-sort".to_string(),
                target_id: Some("name".to_string()),
                payload: None,
            })
            .expect("project sort action should return refreshed state");
        assert_eq!(sort_model.task_summary.label, "项目已排序");
        assert_eq!(sort_model.task_summary.detail, "按名称排序");

        let all_model = session
            .apply_action(HubActionRequest {
                action_id: "view-all-projects".to_string(),
                target_id: None,
                payload: None,
            })
            .expect("view-all-projects action should return refreshed state");
        assert_eq!(all_model.task_summary.label, "全部项目");
        assert_eq!(all_model.task_summary.detail, "显示全部最近项目");

        fs::remove_dir_all(temp).unwrap();
    }
}
