use std::{
    collections::VecDeque,
    env,
    path::{Path, PathBuf},
};

mod action_targets;
pub(in crate::tauri_app) mod action_tasks;
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
    source_engine_display_name, source_engine_id, upsert_source_engine, validate_source_engine,
    SourceEngineInstall, SourceEngineValidation,
};
use crate::error::HubError;
use crate::learn::LearnCatalogEntry;
use crate::plugins::PluginCatalogEntry;
use crate::projects::{
    load_editor_recent_project_session, merge_recent_projects, metadata_for_path,
    project_filesystem_path_key, project_metadata_key, project_paths_match,
    save_editor_recent_projects, save_editor_recent_projects_with_last_project, RecentProject,
};
use crate::settings::{
    default_hub_config_path, editor_config_path, HubConfig, HubRuntimeState, HubSettings,
};
use crate::state::{
    EngineMessageId, HubMessage, HubMessageId, HubPage, HubSnapshot, ProjectFilterMode,
    ProjectMessageId, ProjectSortMode, ProjectSubpage, ProjectViewMode, SettingsMessageId,
    ShellMessageId, TaskOperationKind, TaskStatus,
};
use crate::team::TeamOverview;

use super::action_id::HubActionId;
use super::action_request::{HubAction, HubActionRequest};
use super::view_model::{validate_settings_for_save, HubSettingsPayload, HubViewModel};

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
    folder_picker: fn(&crate::process::FolderPickerRequest) -> Result<Option<PathBuf>, HubError>,
    recycle_delete: fn(PathBuf) -> Result<(), HubError>,
    task_status: TaskStatus,
    background_task_counter: u64,
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
            folder_picker: crate::process::pick_folder,
            recycle_delete: |path| crate::projects::recycle_delete_project(path),
            task_status: TaskStatus::idle(),
            background_task_counter: 0,
            background_worker_active: false,
            background_action_queue: VecDeque::new(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
        };
        if let Err(validation) = session.register_source_engine_from_settings() {
            session.task_status = TaskStatus::warning(
                "Source Engine invalid",
                source_engine_validation_detail(validation),
                source_engine_validation_recovery(validation),
            )
            .with_operation(TaskOperationKind::SourceEngine, "Settings source checkout");
        }
        session.prune_stale_project_engine_bindings();
        session.config.repair_registries();
        if let Some(path) = session.selected_project_path.clone() {
            session.activate_project_engine_for_path(&path);
        }
        session.ensure_new_project_engine_selection();
        session.refresh_source_scoped_views()?;
        session.apply_visual_task_state_override_from_env();
        session.persist(None)?;
        Ok(session)
    }

    pub(super) fn view_model(&self) -> HubViewModel {
        HubViewModel::from_snapshot(&self.snapshot())
    }

    pub(super) fn apply_action(
        &mut self,
        request: HubActionRequest,
    ) -> Result<HubViewModel, HubError> {
        let action_id = request.action()?;
        let action = match request.parse_as(action_id) {
            Ok(action) => action,
            Err(error) => {
                self.record_action_payload_failure(action_id, error)?;
                return Ok(self.view_model());
            }
        };

        match action {
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
            HubAction::UpdateSettingsDraft { payload } => {
                self.update_settings_draft_from_action(payload)?
            }
            HubAction::SaveSettings { payload } => self.save_settings_from_action(payload)?,
            HubAction::DiscardSettingsDraft => self.discard_settings_draft(),
            HubAction::RestoreDefaultSettings => self.restore_default_settings(),
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
                    HubActionId::BuildProject,
                )?;
                self.build_selected_project_engine()?
            }
            HubAction::PackageProject { target_id, payload } => {
                self.apply_action_project_target(
                    target_id.as_deref(),
                    payload.as_ref(),
                    HubActionId::PackageProject,
                )?;
                self.package_recent_project()?
            }
            HubAction::InstallDevice { target_id, payload } => {
                self.apply_action_project_target(
                    target_id.as_deref(),
                    payload.as_ref(),
                    HubActionId::InstallDevice,
                )?;
                self.install_recent_project_to_device()?
            }
            HubAction::OpenEditor { target_id, payload } => {
                self.apply_action_project_target(
                    target_id.as_deref(),
                    payload.as_ref(),
                    HubActionId::OpenEditor,
                )?;
                self.open_selected_project_or_editor()?
            }
        }

        Ok(self.view_model())
    }

    fn record_action_payload_failure(
        &mut self,
        action: HubActionId,
        error: HubError,
    ) -> Result<(), HubError> {
        let (detail, recovery) = error.into_status_messages();
        self.task_status = TaskStatus::error(
            "Action failed",
            detail,
            recovery.unwrap_or_else(|| {
                HubMessage::new(HubMessageId::Shell(ShellMessageId::ReviewActionPayload))
            }),
        )
        .with_operation(TaskOperationKind::Hub, action.as_str());
        self.persist(None)
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
            queued_background_actions: self.background_action_queue.len(),
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
        self.persist(None)
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
        self.persist(None)
    }

    fn search_projects(&mut self, query: &str) {
        self.search_query = query.to_string();
        let _ = self.persist(None);
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
            HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::ShowingFilter),
                [project_filter_label(
                    self.project_filter,
                    self.config.settings.language,
                )],
            ),
        )
        .with_operation(TaskOperationKind::Hub, "Projects");
        self.persist(None)
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
            HubMessage::with_params(
                HubMessageId::Project(ProjectMessageId::SortingBy),
                [project_sort_label(
                    self.project_sort,
                    self.config.settings.language,
                )],
            ),
        )
        .with_operation(TaskOperationKind::Hub, "Projects");
        self.persist(None)
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
        self.persist(None)
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
        self.task_status = TaskStatus::success(
            "Project selected",
            HubMessage::raw_text(display_name.clone()),
        )
        .with_operation(TaskOperationKind::Project, display_name);
        self.persist(Some(&project.path))
    }

    fn open_project_detail(&mut self, target: &str) -> Result<(), HubError> {
        self.select_project_target(target)?;
        self.project_subpage = ProjectSubpage::ProjectDetail;
        self.project_view_mode = ProjectViewMode::List;
        self.pending_delete_project_path = None;
        self.persist(None)
    }

    fn view_all_projects(&mut self) {
        self.search_query.clear();
        self.project_filter = ProjectFilterMode::All;
        self.project_view_mode = ProjectViewMode::List;
        self.project_subpage = ProjectSubpage::ProjectBrowser;
        self.task_status = TaskStatus::success(
            "All projects",
            HubMessage::new(HubMessageId::Project(
                ProjectMessageId::ShowingAllRecentProjects,
            )),
        )
        .with_operation(TaskOperationKind::Hub, "Projects");
        let _ = self.persist(None);
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
        self.persist(None)?;
        self.task_status = TaskStatus::success(
            "Engine selected",
            HubMessage::raw_text(engine.display_name.clone()),
        )
        .with_operation(TaskOperationKind::SourceEngine, engine.display_name);
        Ok(())
    }

    fn save_settings(
        &mut self,
        settings_payload: Option<HubSettingsPayload>,
    ) -> Result<(), HubError> {
        let settings = if let Some(settings_payload) = settings_payload {
            let mut settings = self.config.settings.clone();
            if let Err(error) = settings_payload.apply_to(&mut settings) {
                self.record_settings_save_failure(error.into_status_messages().0);
                return Ok(());
            }
            settings
        } else {
            self.settings_draft.clone()
        };
        if let Err(error) = validate_settings_for_save(&settings) {
            self.record_settings_save_failure(error.into_status_messages().0);
            return Ok(());
        }
        if let Err(validation) = validate_settings_source_engine(&settings) {
            self.record_settings_save_failure(source_engine_validation_detail(validation));
            return Ok(());
        }
        self.config.settings = settings;
        if let Err(validation) = self.register_source_engine_from_settings() {
            self.record_settings_save_failure(source_engine_validation_detail(validation));
            return Ok(());
        }
        self.refresh_source_scoped_views()?;
        self.persist(None)?;
        self.settings_draft = self.config.settings.clone();
        self.task_status = TaskStatus::success(
            "Settings saved",
            HubMessage::with_params(
                HubMessageId::Settings(SettingsMessageId::SettingsSavedPath),
                [self.config_path.to_string_lossy()],
            ),
        )
        .with_operation(TaskOperationKind::Settings, "Hub settings");
        Ok(())
    }

    fn persist(&mut self, last_project_path: Option<&Path>) -> Result<(), HubError> {
        match self.persist_unchecked(last_project_path) {
            Ok(()) => Ok(()),
            Err(error) => {
                self.task_status = TaskStatus::error(
                    "Save Hub state failed",
                    HubMessage::raw_text(error.to_string()),
                    HubMessage::new(HubMessageId::Shell(ShellMessageId::CheckConfigPath)),
                )
                .with_operation(TaskOperationKind::Hub, "Hub state");
                Err(error)
            }
        }
    }

    fn persist_unchecked(&self, last_project_path: Option<&Path>) -> Result<(), HubError> {
        let mut config = self.config.clone();
        config.runtime = self.runtime_state_for_config();
        config.save(&self.config_path)?;
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
                    HubMessage::new(HubMessageId::Shell(ShellMessageId::RefreshingCatalogs)),
                    TaskOperationKind::Hub,
                    "Visual verification",
                );
            }
            "error" => {
                self.task_status = TaskStatus::error(
                    "Action failed",
                    HubMessage::new(HubMessageId::Shell(ShellMessageId::VisualVerificationError)),
                    HubMessage::new(HubMessageId::Shell(
                        ShellMessageId::CheckHighlightedWorkflowTarget,
                    )),
                )
                .with_operation(TaskOperationKind::Hub, "Visual verification");
            }
            "warning" => {
                self.task_status = TaskStatus::warning(
                    "Warning",
                    HubMessage::new(HubMessageId::Shell(
                        ShellMessageId::VisualVerificationWarning,
                    )),
                    HubMessage::new(HubMessageId::Shell(
                        ShellMessageId::ReviewSettingsBeforeContinuing,
                    )),
                )
                .with_operation(TaskOperationKind::Hub, "Visual verification");
            }
            "success" => {
                self.task_status = TaskStatus::success(
                    "Success",
                    HubMessage::new(HubMessageId::Shell(
                        ShellMessageId::VisualVerificationSuccess,
                    )),
                )
                .with_operation(TaskOperationKind::Hub, "Visual verification");
            }
            _ => {}
        }
    }

    fn register_source_engine_from_settings(&mut self) -> Result<(), SourceEngineValidation> {
        let source_dir = self.config.settings.default_source_dir.clone();
        let output_dir = self.config.settings.default_build_output_dir.clone();
        if source_dir.as_os_str().is_empty() {
            return Ok(());
        }
        let validation = validate_source_engine(&source_dir);
        if validation != SourceEngineValidation::Valid {
            return Err(validation);
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
        Ok(())
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

    fn find_recent_project_by_filesystem_key(&self, path: &Path) -> Option<RecentProject> {
        let key = project_filesystem_path_key(path);
        self.config
            .recent_projects
            .iter()
            .find(|project| project_filesystem_path_key(&project.path) == key)
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

fn validate_settings_source_engine(settings: &HubSettings) -> Result<(), SourceEngineValidation> {
    if settings.default_source_dir.as_os_str().is_empty() {
        return Ok(());
    }
    let validation = validate_source_engine(&settings.default_source_dir);
    if validation == SourceEngineValidation::Valid {
        Ok(())
    } else {
        Err(validation)
    }
}

fn source_engine_validation_detail(validation: SourceEngineValidation) -> HubMessage {
    let id = match validation {
        SourceEngineValidation::Valid => EngineMessageId::SourceEngineReady,
        SourceEngineValidation::MissingRoot => EngineMessageId::CheckoutDirectoryMissing,
        SourceEngineValidation::MissingWorkspaceManifest => EngineMessageId::MissingCargoToml,
        SourceEngineValidation::MissingRuntimeWorkspaceMember => {
            EngineMessageId::MissingRuntimeMember
        }
        SourceEngineValidation::MissingBuildTool => EngineMessageId::MissingBuildTool,
    };
    HubMessage::new(HubMessageId::Engine(id))
}

fn source_engine_validation_recovery(validation: SourceEngineValidation) -> HubMessage {
    let id = match validation {
        SourceEngineValidation::Valid => {
            return HubMessage::new(HubMessageId::Shell(ShellMessageId::NoRecoveryRequired));
        }
        SourceEngineValidation::MissingRoot => EngineMessageId::LocateCheckoutRecovery,
        SourceEngineValidation::MissingWorkspaceManifest => EngineMessageId::SelectRepositoryRoot,
        SourceEngineValidation::MissingRuntimeWorkspaceMember => {
            EngineMessageId::SelectRepositoryWithRuntime
        }
        SourceEngineValidation::MissingBuildTool => EngineMessageId::SelectCompleteCheckout,
    };
    HubMessage::new(HubMessageId::Engine(id))
}

fn project_filter_label(
    filter: ProjectFilterMode,
    language: crate::settings::HubLanguage,
) -> String {
    match language {
        crate::settings::HubLanguage::Chinese => match filter {
            ProjectFilterMode::All => "全部项目",
            ProjectFilterMode::Existing => "存在项目",
            ProjectFilterMode::Missing => "缺失项目",
        },
        crate::settings::HubLanguage::English => filter.label(),
    }
    .to_string()
}

fn project_sort_label(sort: ProjectSortMode, language: crate::settings::HubLanguage) -> String {
    match language {
        crate::settings::HubLanguage::Chinese => match sort {
            ProjectSortMode::LastModified => "最近修改",
            ProjectSortMode::Name => "名称",
        },
        crate::settings::HubLanguage::English => sort.label(),
    }
    .to_string()
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
#[path = "runtime_state/tests.rs"]
mod tests;
