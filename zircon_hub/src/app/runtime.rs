use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use crate::assets::AssetCatalogEntry;
use slint::ComponentHandle;

use crate::build::{run_build_command, BuildCommand, BuildCommandOptions, BuildExecutionReport};
use crate::engines::{
    active_source_engine, active_source_engine_mut, ensure_active_source_engine,
    remove_source_engine, upsert_source_engine, validate_source_engine, SourceBuildRecord,
    SourceEngineInstall, SourceEngineValidation,
};
use crate::error::HubError;
use crate::learn::LearnCatalogEntry;
use crate::plugins::PluginCatalogEntry;
use crate::process::{
    open_folder, preferred_editor_executable, preferred_editor_executable_exists, OpenFolderCommand,
};
use crate::projects::{
    load_editor_recent_project_session, merge_recent_projects, project_paths_match,
    save_editor_recent_projects, save_editor_recent_projects_with_last_project, ProjectTemplate,
};
use crate::settings::{default_hub_config_path, editor_config_path, HubConfig};
use crate::state::{
    HubPage, HubSnapshot, ProjectFilterMode, ProjectSortMode, ProjectSubpage, ProjectViewMode,
    TaskStatus,
};
use crate::team::TeamOverview;

use self::source_engine_paths::{
    same_source_engine_path, source_engine_display_name, source_engine_id,
};
use super::binding;
use super::quick_action::HubQuickAction;
use super::HubWindow;

mod asset_catalog;
mod folder_picker;
mod learn_catalog;
mod plugin_catalog;
mod project_workspace;
mod root_paths;
mod source_engine_paths;
mod source_scoped_views;
mod team_overview;
mod window_controls;

pub(super) fn run() -> Result<(), HubError> {
    let ui = HubWindow::new()?;
    let runtime = Rc::new(RefCell::new(HubRuntime::load()?));
    binding::apply_snapshot(&ui, &runtime.borrow().snapshot());
    runtime.borrow().apply_window_state(&ui);
    wire_callbacks(&ui, runtime);
    ui.run()?;
    Ok(())
}

struct HubRuntime {
    config_path: PathBuf,
    editor_config_path: PathBuf,
    config: HubConfig,
    selected_page: HubPage,
    project_filter: ProjectFilterMode,
    project_sort: ProjectSortMode,
    project_view_mode: ProjectViewMode,
    project_subpage: ProjectSubpage,
    search_query: String,
    selected_project_path: Option<PathBuf>,
    selected_template_id: String,
    new_project_location: PathBuf,
    new_project_engine_id: Option<String>,
    pending_delete_project_path: Option<PathBuf>,
    task_status: TaskStatus,
    asset_catalog: Vec<AssetCatalogEntry>,
    learn_catalog: Vec<LearnCatalogEntry>,
    plugin_catalog: Vec<PluginCatalogEntry>,
    team_overview: TeamOverview,
}

impl HubRuntime {
    fn load() -> Result<Self, HubError> {
        let config_path = default_hub_config_path();
        let editor_config_path = editor_config_path();
        let mut config = HubConfig::load(&config_path)?;
        let editor_recent = load_editor_recent_project_session(&editor_config_path)?;
        let last_project_path = editor_recent.last_project_path;
        config.recent_projects =
            merge_recent_projects(config.recent_projects, editor_recent.recent_projects);
        let selected_project_path =
            startup_selected_project_path(last_project_path.as_deref(), &config.recent_projects);
        let new_project_location = config.settings.default_project_dir.clone();
        let mut runtime = Self {
            config_path,
            editor_config_path,
            config,
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path,
            selected_template_id: ProjectTemplate::RenderableEmpty.id().to_string(),
            new_project_location,
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
        };
        runtime.register_source_engine_from_settings();
        if let Some(path) = runtime.selected_project_path.clone() {
            runtime.activate_project_engine_for_path(&path);
        }
        runtime.ensure_new_project_engine_selection();
        runtime.refresh_source_scoped_views()?;
        runtime.persist()?;
        Ok(runtime)
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
            engines: self.config.engines.clone(),
            active_engine_id: self.config.active_engine_id.clone(),
            settings: self.config.settings.clone(),
        }
    }

    fn select_page(&mut self, page: HubPage) {
        self.selected_page = page;
    }

    fn select_page_by_id(&mut self, page_id: &str) -> Result<(), HubError> {
        let Some(page) = HubPage::from_id(page_id) else {
            return Err(HubError::message(format!("Unknown Hub page: {page_id}")));
        };
        self.select_page(page);
        Ok(())
    }

    fn select_engine_by_id(&mut self, ui: &HubWindow, engine_id: &str) -> Result<(), HubError> {
        self.sync_from_ui(ui);
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
        self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());
        self.refresh_source_scoped_views()?;
        self.persist_hub_config()?;
        self.task_status = TaskStatus {
            label: "Engine selected".to_string(),
            detail: engine.display_name,
            running: false,
        };
        Ok(())
    }

    fn rename_active_engine(&mut self, ui: &HubWindow, name: &str) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let name = name.trim();
        if name.is_empty() {
            self.task_status = TaskStatus {
                label: "Rename skipped".to_string(),
                detail: "Engine name cannot be empty".to_string(),
                running: false,
            };
            return Ok(());
        }
        let Some(engine) = active_source_engine_mut(
            &mut self.config.engines,
            self.config.active_engine_id.as_deref(),
        ) else {
            self.task_status = TaskStatus {
                label: "No engine".to_string(),
                detail: "Configure a source checkout before renaming".to_string(),
                running: false,
            };
            return Ok(());
        };
        engine.display_name = name.to_string();
        self.persist_hub_config()?;
        self.task_status = TaskStatus {
            label: "Engine renamed".to_string(),
            detail: name.to_string(),
            running: false,
        };
        Ok(())
    }

    fn remove_engine_by_id(&mut self, ui: &HubWindow, engine_id: &str) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let active_engine_before = self.config.active_engine_id.clone();
        let Some(removed) = remove_source_engine(
            &mut self.config.engines,
            &mut self.config.active_engine_id,
            engine_id,
        ) else {
            return Err(HubError::message(format!(
                "Unknown source engine: {engine_id}"
            )));
        };
        self.sync_settings_from_active_engine();
        self.sync_new_project_engine_after_active_engine_change(active_engine_before.as_deref());
        self.refresh_source_scoped_views()?;
        self.persist_hub_config()?;
        self.task_status = TaskStatus {
            label: "Engine removed".to_string(),
            detail: removed.display_name,
            running: false,
        };
        Ok(())
    }

    fn cycle_active_engine(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        if self.config.engines.is_empty() {
            self.task_status = TaskStatus {
                label: "No engines".to_string(),
                detail: "Configure a source checkout first".to_string(),
                running: false,
            };
            return Ok(());
        }
        let current_index = self
            .config
            .active_engine_id
            .as_deref()
            .and_then(|id| {
                self.config
                    .engines
                    .iter()
                    .position(|engine| engine.id == id)
            })
            .unwrap_or(0);
        let next_index = (current_index + 1) % self.config.engines.len();
        let next_id = self.config.engines[next_index].id.clone();
        self.select_engine_by_id(ui, &next_id)
    }

    fn sync_from_ui(&mut self, ui: &HubWindow) {
        self.search_query = ui.get_search_query().to_string();
        self.new_project_location = PathBuf::from(ui.get_new_project_location().to_string());
        self.config.settings = binding::read_settings(ui, self.config.settings.clone());
    }

    fn save_settings(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.register_source_engine_from_settings();
        self.refresh_source_scoped_views()?;
        self.persist()?;
        self.task_status = TaskStatus {
            label: "Settings saved".to_string(),
            detail: self.config_path.to_string_lossy().into_owned(),
            running: false,
        };
        Ok(())
    }

    fn quick_action(&mut self, ui: &HubWindow, action_id: &str) -> Result<(), HubError> {
        match HubQuickAction::from_id(action_id) {
            Some(HubQuickAction::BuildProject) => self.build_selected_project_engine(ui),
            Some(HubQuickAction::OpenEditor) => self.open_selected_project_or_editor(ui),
            Some(HubQuickAction::PackageProject) => self.package_recent_project(ui),
            Some(HubQuickAction::InstallToDevice) => self.install_recent_project_to_device(ui),
            None => Err(HubError::message(format!(
                "Unknown quick action: {action_id}"
            ))),
        }
    }

    fn build_selected_project_engine(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.selected_or_latest_recent_project_with_engine_for_action()?;
        self.build_editor_runtime_after_sync(ui)
    }

    fn build_selected_project_engine_only(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.selected_project_with_engine_for_action()?;
        self.build_editor_runtime_after_sync(ui)
    }

    fn build_editor_runtime(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.build_editor_runtime_after_sync(ui)
    }

    fn build_editor_runtime_after_sync(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.register_source_engine_from_settings();
        self.refresh_source_scoped_views()?;
        let validation = validate_source_engine(&self.config.settings.default_source_dir);
        if validation != SourceEngineValidation::Valid {
            return Err(HubError::message(format!(
                "Source engine is not valid: {validation:?}"
            )));
        }
        self.task_status = TaskStatus::running("Building", "Running tools/zircon_build.py");
        binding::apply_snapshot(ui, &self.snapshot());
        let command = BuildCommand::for_editor_runtime(&BuildCommandOptions::new(
            self.config.settings.python_path.clone(),
            self.config.settings.cargo_path.clone(),
            self.config.settings.default_source_dir.clone(),
            self.config.settings.default_build_output_dir.clone(),
            self.config.settings.build_profile,
            Some(self.config.settings.jobs),
        ));
        let report = match run_build_command(&command) {
            Ok(report) => report,
            Err(error) => {
                self.record_active_build(false, error.to_string());
                self.persist_hub_config()?;
                return Err(error);
            }
        };
        if !report.succeeded() {
            let detail = build_failure_detail(&report);
            self.record_active_build(false, detail.clone());
            self.persist_hub_config()?;
            self.task_status = TaskStatus {
                label: "Build failed".to_string(),
                detail,
                running: false,
            };
            return Err(HubError::message(self.task_status.detail.clone()));
        }
        self.record_active_build(true, "Staged editor/runtime payload".to_string());
        self.persist()?;
        self.task_status = TaskStatus {
            label: "Build complete".to_string(),
            detail: self.staged_engine_dir().to_string_lossy().into_owned(),
            running: false,
        };
        Ok(())
    }

    fn ensure_editor_available(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        if preferred_editor_executable_exists(self.staged_engine_dir()) {
            return Ok(());
        }
        self.build_editor_runtime(ui)
    }

    fn open_output(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let command = OpenFolderCommand::new(self.staged_engine_dir());
        open_folder(&command)?;
        self.task_status = TaskStatus {
            label: "Output opened".to_string(),
            detail: self
                .config
                .settings
                .default_build_output_dir
                .to_string_lossy()
                .into_owned(),
            running: false,
        };
        Ok(())
    }

    fn launch_editor_without_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.ensure_editor_available(ui)?;
        let executable = preferred_editor_executable(self.staged_engine_dir());
        std::process::Command::new(&executable).spawn()?;
        self.task_status = TaskStatus {
            label: "Editor launched".to_string(),
            detail: executable.to_string_lossy().into_owned(),
            running: false,
        };
        Ok(())
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

    fn staged_engine_dir(&self) -> PathBuf {
        active_source_engine(
            &self.config.engines,
            self.config.active_engine_id.as_deref(),
        )
        .map(SourceEngineInstall::staged_engine_dir)
        .unwrap_or_else(|| {
            self.config
                .settings
                .default_build_output_dir
                .join("ZirconEngine")
        })
    }

    fn sync_settings_from_active_engine(&mut self) {
        if let Some(engine) = active_source_engine(
            &self.config.engines,
            self.config.active_engine_id.as_deref(),
        ) {
            self.config.settings.default_source_dir = engine.source_dir.clone();
            self.config.settings.default_build_output_dir = engine.output_dir.clone();
        }
    }

    fn record_active_build(&mut self, success: bool, detail: String) {
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
            });
        }
    }

    fn persist(&self) -> Result<(), HubError> {
        self.persist_with_last_project(None)
    }

    fn persist_hub_config(&self) -> Result<(), HubError> {
        self.config.save(&self.config_path)?;
        Ok(())
    }

    fn persist_with_last_project(
        &self,
        last_project_path: Option<&std::path::Path>,
    ) -> Result<(), HubError> {
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
}

fn build_failure_detail(report: &BuildExecutionReport) -> String {
    report
        .stderr
        .lines()
        .last()
        .or_else(|| report.stdout.lines().last())
        .unwrap_or("build failed")
        .to_string()
}

fn startup_selected_project_path(
    last_project_path: Option<&Path>,
    recent_projects: &[crate::projects::RecentProject],
) -> Option<PathBuf> {
    let last_project_path = last_project_path?;
    recent_projects
        .iter()
        .find(|project| project_paths_match(&project.path, last_project_path))
        .map(|project| project.path.clone())
}

fn wire_callbacks(ui: &HubWindow, runtime: Rc<RefCell<HubRuntime>>) {
    let weak = ui.as_weak();
    let runtime_for_page = Rc::clone(&runtime);
    ui.on_show_page(move |page_id| {
        with_runtime(&weak, &runtime_for_page, |runtime, _ui| {
            runtime.select_page_by_id(&page_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_save = Rc::clone(&runtime);
    ui.on_save_settings(move || {
        with_runtime(&weak, &runtime_for_save, |runtime, ui| {
            runtime.save_settings(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_import = Rc::clone(&runtime);
    ui.on_import_project(move || {
        with_runtime(&weak, &runtime_for_import, |runtime, ui| {
            runtime.import_project(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_open = Rc::clone(&runtime);
    ui.on_open_project(move || {
        with_runtime(&weak, &runtime_for_open, |runtime, ui| {
            runtime.open_project(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_recent = Rc::clone(&runtime);
    ui.on_open_recent_project(move |path| {
        with_runtime(&weak, &runtime_for_recent, |runtime, ui| {
            runtime.open_recent_project(ui, &path)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_search = Rc::clone(&runtime);
    ui.on_search_projects(move |query| {
        with_runtime(&weak, &runtime_for_search, |runtime, _ui| {
            runtime.search_projects(&query);
            Ok(())
        })
    });

    let weak = ui.as_weak();
    let runtime_for_project_selection = Rc::clone(&runtime);
    ui.on_select_project(move |path| {
        with_runtime(&weak, &runtime_for_project_selection, |runtime, _ui| {
            runtime.select_project_path(&path)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_project_page = Rc::clone(&runtime);
    ui.on_show_project_subpage(move |subpage_id| {
        with_runtime(&weak, &runtime_for_project_page, |runtime, _ui| {
            runtime.show_project_subpage_by_id(&subpage_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_project_detail = Rc::clone(&runtime);
    ui.on_open_project_detail(move |path| {
        with_runtime(&weak, &runtime_for_project_detail, |runtime, _ui| {
            runtime.open_project_detail(&path)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_view_all = Rc::clone(&runtime);
    ui.on_view_all_projects(move || {
        with_runtime(&weak, &runtime_for_view_all, |runtime, _ui| {
            runtime.view_all_projects();
            Ok(())
        })
    });

    let weak = ui.as_weak();
    let runtime_for_browse = Rc::clone(&runtime);
    ui.on_browse_folder(move |target| {
        with_runtime(&weak, &runtime_for_browse, |runtime, ui| {
            runtime.browse_folder(ui, &target)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_filter = Rc::clone(&runtime);
    ui.on_set_project_filter(move |filter_id| {
        with_runtime(&weak, &runtime_for_filter, |runtime, _ui| {
            runtime.set_project_filter_by_id(&filter_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_sort = Rc::clone(&runtime);
    ui.on_set_project_sort(move |sort_id| {
        with_runtime(&weak, &runtime_for_sort, |runtime, _ui| {
            runtime.set_project_sort_by_id(&sort_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_view = Rc::clone(&runtime);
    ui.on_set_project_view_mode(move |mode_id| {
        with_runtime(&weak, &runtime_for_view, |runtime, _ui| {
            runtime.set_project_view_mode_by_id(&mode_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_template = Rc::clone(&runtime);
    ui.on_select_project_template(move |template_id| {
        with_runtime(&weak, &runtime_for_template, |runtime, _ui| {
            runtime.select_project_template_by_id(&template_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_new_project_engine = Rc::clone(&runtime);
    ui.on_select_new_project_engine(move |engine_id| {
        with_runtime(&weak, &runtime_for_new_project_engine, |runtime, _ui| {
            runtime.select_new_project_engine_by_id(&engine_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_detail_engine = Rc::clone(&runtime);
    ui.on_select_project_detail_engine(move |engine_id| {
        with_runtime(&weak, &runtime_for_detail_engine, |runtime, _ui| {
            runtime.select_project_detail_engine_by_id(&engine_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_pin = Rc::clone(&runtime);
    ui.on_toggle_project_pin(move || {
        with_runtime(&weak, &runtime_for_pin, |runtime, _ui| {
            runtime.toggle_selected_project_pin()
        })
    });

    let weak = ui.as_weak();
    let runtime_for_remove_project = Rc::clone(&runtime);
    ui.on_remove_project_from_hub(move || {
        with_runtime(&weak, &runtime_for_remove_project, |runtime, _ui| {
            runtime.remove_selected_project_from_hub()
        })
    });

    let weak = ui.as_weak();
    let runtime_for_delete_request = Rc::clone(&runtime);
    ui.on_request_delete_project(move || {
        with_runtime(&weak, &runtime_for_delete_request, |runtime, _ui| {
            runtime.request_delete_selected_project()
        })
    });

    let weak = ui.as_weak();
    let runtime_for_delete_cancel = Rc::clone(&runtime);
    ui.on_cancel_delete_project(move || {
        with_runtime(&weak, &runtime_for_delete_cancel, |runtime, _ui| {
            runtime.cancel_delete_project();
            Ok(())
        })
    });

    let weak = ui.as_weak();
    let runtime_for_delete_confirm = Rc::clone(&runtime);
    ui.on_confirm_delete_project(move || {
        with_runtime(&weak, &runtime_for_delete_confirm, |runtime, _ui| {
            runtime.confirm_delete_project()
        })
    });

    let weak = ui.as_weak();
    let runtime_for_engine_select = Rc::clone(&runtime);
    ui.on_select_engine(move |engine_id| {
        with_runtime(&weak, &runtime_for_engine_select, |runtime, ui| {
            runtime.select_engine_by_id(ui, &engine_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_engine_rename = Rc::clone(&runtime);
    ui.on_rename_active_engine(move |name| {
        with_runtime(&weak, &runtime_for_engine_rename, |runtime, ui| {
            runtime.rename_active_engine(ui, &name)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_engine_remove = Rc::clone(&runtime);
    ui.on_remove_engine(move |engine_id| {
        with_runtime(&weak, &runtime_for_engine_remove, |runtime, ui| {
            runtime.remove_engine_by_id(ui, &engine_id)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_engine_cycle = Rc::clone(&runtime);
    ui.on_cycle_engine(move || {
        with_runtime(&weak, &runtime_for_engine_cycle, |runtime, ui| {
            runtime.cycle_active_engine(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_create = Rc::clone(&runtime);
    ui.on_create_project(move || {
        with_runtime(&weak, &runtime_for_create, |runtime, ui| {
            runtime.create_project(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_build = Rc::clone(&runtime);
    ui.on_build_engine(move || {
        with_runtime(&weak, &runtime_for_build, |runtime, ui| {
            runtime.build_editor_runtime(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_selected_build = Rc::clone(&runtime);
    ui.on_build_selected_project_engine(move || {
        with_runtime(&weak, &runtime_for_selected_build, |runtime, ui| {
            runtime.build_selected_project_engine_only(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_open_output = Rc::clone(&runtime);
    ui.on_open_output(move || {
        with_runtime(&weak, &runtime_for_open_output, |runtime, ui| {
            runtime.open_output(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_launch = Rc::clone(&runtime);
    ui.on_launch_editor(move || {
        with_runtime(&weak, &runtime_for_launch, |runtime, ui| {
            runtime.open_selected_project_or_editor(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_selected_launch = Rc::clone(&runtime);
    ui.on_launch_selected_project(move || {
        with_runtime(&weak, &runtime_for_selected_launch, |runtime, ui| {
            runtime.open_selected_project_in_editor(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_selected_package = Rc::clone(&runtime);
    ui.on_package_selected_project(move || {
        with_runtime(&weak, &runtime_for_selected_package, |runtime, ui| {
            runtime.package_selected_project(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_selected_install = Rc::clone(&runtime);
    ui.on_install_selected_project(move || {
        with_runtime(&weak, &runtime_for_selected_install, |runtime, ui| {
            runtime.install_selected_project_to_device(ui)
        })
    });

    let weak = ui.as_weak();
    let runtime_for_learn = Rc::clone(&runtime);
    ui.on_open_learn_resource(move |path| {
        with_runtime(&weak, &runtime_for_learn, |runtime, _ui| {
            runtime.open_learn_resource(&path)
        })
    });

    wire_quick_actions(ui, Rc::clone(&runtime));
    window_controls::wire_window_controls(ui, runtime);
}

fn wire_quick_actions(ui: &HubWindow, runtime: Rc<RefCell<HubRuntime>>) {
    let weak = ui.as_weak();
    ui.on_quick_action(move |action_id| {
        with_runtime(&weak, &runtime, |runtime, ui| {
            runtime.quick_action(ui, &action_id)
        })
    });
}

fn with_runtime(
    weak: &slint::Weak<HubWindow>,
    runtime: &Rc<RefCell<HubRuntime>>,
    action: impl FnOnce(&mut HubRuntime, &HubWindow) -> Result<(), HubError>,
) {
    let Some(ui) = weak.upgrade() else {
        return;
    };
    let mut runtime = runtime.borrow_mut();
    if let Err(error) = action(&mut runtime, &ui) {
        runtime.task_status = TaskStatus {
            label: "Action failed".to_string(),
            detail: error.to_string(),
            running: false,
        };
    }
    binding::apply_snapshot(&ui, &runtime.snapshot());
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::projects::RecentProject;
    use crate::settings::HubConfig;

    fn runtime_with_projects(projects: Vec<RecentProject>) -> HubRuntime {
        HubRuntime {
            config_path: PathBuf::from("hub.toml"),
            editor_config_path: PathBuf::from("editor.json"),
            config: HubConfig {
                recent_projects: projects,
                ..HubConfig::default()
            },
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            project_subpage: ProjectSubpage::Dashboard,
            search_query: String::new(),
            selected_project_path: None,
            selected_template_id: ProjectTemplate::RenderableEmpty.id().to_string(),
            new_project_location: HubConfig::default().settings.default_project_dir,
            new_project_engine_id: None,
            pending_delete_project_path: None,
            task_status: TaskStatus::idle(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
        }
    }

    #[test]
    fn selecting_project_records_path_and_status() {
        let mut runtime = runtime_with_projects(vec![RecentProject::new(
            "Stellar Outpost",
            "E:/Projects/StellarOutpost",
            20,
        )]);

        runtime
            .select_project_path("E:/Projects/StellarOutpost")
            .unwrap();

        assert_eq!(
            runtime.selected_project_path,
            Some(PathBuf::from("E:/Projects/StellarOutpost"))
        );
        assert_eq!(runtime.task_status.label, "Project selected");
        assert_eq!(runtime.task_status.detail, "Stellar Outpost");
    }

    #[test]
    fn startup_selection_restores_editor_last_project_when_it_matches_recent_projects() {
        let recent_projects = vec![
            RecentProject::new("Elysium", "E:/Projects/Elysium", 30),
            RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 10),
        ];

        let selected = startup_selected_project_path(
            Some(Path::new("E:\\Projects\\StellarOutpost\\")),
            &recent_projects,
        );

        assert_eq!(selected, Some(PathBuf::from("E:/Projects/StellarOutpost")));
        assert!(startup_selected_project_path(
            Some(Path::new("E:/Projects/Missing")),
            &recent_projects,
        )
        .is_none());
    }

    #[test]
    fn quick_action_target_prefers_selected_project_over_latest_recent() {
        let mut runtime = runtime_with_projects(vec![
            RecentProject::new("Elysium", "E:/Projects/Elysium", 30),
            RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 10),
        ]);
        runtime.selected_project_path = Some(PathBuf::from("E:\\Projects\\StellarOutpost\\"));

        let project = runtime.selected_or_latest_recent_project().unwrap();

        assert_eq!(project.display_name, "Stellar Outpost");
        assert_eq!(
            runtime.selected_project_path,
            Some(PathBuf::from("E:/Projects/StellarOutpost"))
        );
    }

    #[test]
    fn quick_action_target_falls_back_to_latest_recent_project() {
        let mut runtime = runtime_with_projects(vec![
            RecentProject::new("Elysium", "E:/Projects/Elysium", 30),
            RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 10),
        ]);

        let project = runtime.selected_or_latest_recent_project().unwrap();

        assert_eq!(project.display_name, "Elysium");
        assert_eq!(
            runtime.selected_project_path,
            Some(PathBuf::from("E:/Projects/Elysium"))
        );
    }

    #[test]
    fn quick_action_target_does_not_fallback_when_selected_project_is_stale() {
        let mut runtime = runtime_with_projects(vec![
            RecentProject::new("Elysium", "E:/Projects/Elysium", 30),
            RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 10),
        ]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Missing"));

        let project = runtime.selected_or_latest_recent_project();

        assert!(project.is_none());
        assert!(runtime.selected_project_path.is_none());
    }

    #[test]
    fn quick_action_build_reports_stale_selected_project() {
        let mut runtime = runtime_with_projects(vec![RecentProject::new(
            "Elysium",
            "E:/Projects/Elysium",
            30,
        )]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Missing"));

        let error = runtime
            .selected_or_latest_recent_project_with_engine_for_action()
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Selected project is no longer available to build"
        );
    }

    #[test]
    fn view_all_projects_resets_filter_search_and_view_mode() {
        let mut runtime = runtime_with_projects(Vec::new());
        runtime.search_query = "stellar".to_string();
        runtime.project_filter = ProjectFilterMode::Missing;
        runtime.project_view_mode = ProjectViewMode::List;

        runtime.view_all_projects();

        assert!(runtime.search_query.is_empty());
        assert_eq!(runtime.project_filter, ProjectFilterMode::All);
        assert_eq!(runtime.project_view_mode, ProjectViewMode::List);
        assert_eq!(runtime.project_subpage, ProjectSubpage::ProjectBrowser);
        assert_eq!(runtime.task_status.label, "All projects");
    }

    #[test]
    fn new_project_location_is_tracked_independently_from_settings_default() {
        let mut runtime = runtime_with_projects(Vec::new());
        runtime.config.settings.default_project_dir = PathBuf::from("E:/Projects/Default");
        runtime.new_project_location = PathBuf::from("D:/Drafts/Zircon");

        let snapshot = runtime.snapshot();

        assert_eq!(
            snapshot.new_project_location,
            PathBuf::from("D:/Drafts/Zircon")
        );
        assert_eq!(
            runtime.config.settings.default_project_dir,
            PathBuf::from("E:/Projects/Default")
        );
    }

    #[test]
    fn list_view_opens_project_browser_subpage() {
        let mut runtime = runtime_with_projects(Vec::new());

        runtime.set_project_view_mode_by_id("list").unwrap();

        assert_eq!(runtime.project_view_mode, ProjectViewMode::List);
        assert_eq!(runtime.project_subpage, ProjectSubpage::ProjectBrowser);
    }

    #[test]
    fn opening_project_detail_selects_project_and_enters_detail_subpage() {
        let mut runtime = runtime_with_projects(vec![RecentProject::new(
            "Stellar Outpost",
            "E:/Projects/StellarOutpost",
            20,
        )]);

        runtime
            .open_project_detail("E:/Projects/StellarOutpost")
            .unwrap();

        assert_eq!(
            runtime.selected_project_path,
            Some(PathBuf::from("E:/Projects/StellarOutpost"))
        );
        assert_eq!(runtime.project_view_mode, ProjectViewMode::List);
        assert_eq!(runtime.project_subpage, ProjectSubpage::ProjectDetail);
    }

    #[test]
    fn remove_project_from_hub_does_not_delete_project_directory() {
        let root = std::env::temp_dir().join(format!(
            "zircon-hub-remove-only-test-{}",
            crate::projects::now_unix_ms()
        ));
        std::fs::create_dir_all(&root).unwrap();
        let mut runtime = runtime_with_projects(vec![RecentProject::new("Game", root.clone(), 1)]);
        let alias = PathBuf::from(format!("{}/", root.to_string_lossy().replace('\\', "/")));
        let remove_alias =
            PathBuf::from(format!("{}\\", root.to_string_lossy().replace('/', "\\")));
        runtime.selected_project_path = Some(alias.clone());
        runtime.pending_delete_project_path = Some(alias);
        runtime.config.project_metadata.insert(
            crate::projects::project_metadata_key(&root),
            crate::projects::ProjectMetadata {
                pinned: true,
                engine_id: Some("source".to_string()),
                last_selected_template: Some("renderable-empty".to_string()),
            },
        );

        runtime.remove_project_from_hub_path(&remove_alias);

        assert!(root.exists());
        assert!(runtime.config.recent_projects.is_empty());
        assert!(runtime.config.project_metadata.is_empty());
        assert!(runtime.selected_project_path.is_none());
        assert!(runtime.pending_delete_project_path.is_none());
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn source_engine_registration_deduplicates_same_source_path_and_migrates_metadata() {
        let project = PathBuf::from("E:/Projects/Game");
        let source_dir = PathBuf::from("E:/Git/ZirconEngine");
        let mut runtime = runtime_with_projects(vec![RecentProject::new("Game", &project, 1)]);
        runtime.config.settings.default_source_dir = source_dir.clone();
        runtime.config.settings.default_build_output_dir = PathBuf::from("E:/Git/ZirconEngine/out");
        runtime.config.engines.push(SourceEngineInstall {
            id: "source-local".to_string(),
            display_name: "ZirconEngine Source".to_string(),
            source_dir: PathBuf::from("E:/Git/ZirconEngine/"),
            output_dir: PathBuf::from("E:/Git/ZirconEngine/old-out"),
            last_build_unix_ms: Some(42),
            build_history: Vec::new(),
        });
        runtime.config.active_engine_id = Some("source-local".to_string());
        runtime.new_project_engine_id = Some("source-local".to_string());
        runtime.config.project_metadata.insert(
            crate::projects::project_metadata_key(&project),
            crate::projects::ProjectMetadata {
                pinned: true,
                engine_id: Some("source-local".to_string()),
                last_selected_template: Some("renderable-empty".to_string()),
            },
        );

        runtime.register_source_engine_from_settings();

        let registered_id = source_engine_id(&source_dir);
        assert_eq!(runtime.config.engines.len(), 1);
        assert_eq!(runtime.config.engines[0].id, registered_id);
        assert_eq!(runtime.config.engines[0].last_build_unix_ms, Some(42));
        assert_eq!(
            runtime.config.active_engine_id.as_deref(),
            Some(registered_id.as_str())
        );
        assert_eq!(
            runtime.new_project_engine_id.as_deref(),
            Some(registered_id.as_str())
        );
        assert_eq!(
            runtime
                .config
                .project_metadata
                .get(&crate::projects::project_metadata_key(&project))
                .and_then(|metadata| metadata.engine_id.as_deref()),
            Some(registered_id.as_str())
        );
    }
}
