use std::cell::RefCell;
use std::path::PathBuf;
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
    launch_editor, open_folder, preferred_editor_executable, preferred_editor_executable_exists,
    EditorLaunchCommand, EditorLaunchRequest, OpenFolderCommand,
};
use crate::projects::{
    install_package_to_device, load_editor_recent_projects, merge_recent_projects, package_project,
    save_editor_recent_projects, save_editor_recent_projects_with_last_project,
    validate_project_root, CreateProjectRequest, DeviceInstallRequest, ProjectPackageRequest,
    ProjectTemplate, ProjectValidation, RecentProject,
};
use crate::settings::{default_hub_config_path, editor_config_path, HubConfig};
use crate::state::{
    HubPage, HubSnapshot, ProjectFilterMode, ProjectSortMode, ProjectViewMode, TaskStatus,
};
use crate::team::TeamOverview;

use super::binding;
use super::quick_action::HubQuickAction;
use super::HubWindow;

mod asset_catalog;
mod folder_picker;
mod learn_catalog;
mod plugin_catalog;
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
    search_query: String,
    selected_project_path: Option<PathBuf>,
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
        let editor_recent = load_editor_recent_projects(&editor_config_path)?;
        config.recent_projects = merge_recent_projects(config.recent_projects, editor_recent);
        let mut runtime = Self {
            config_path,
            editor_config_path,
            config,
            selected_page: HubPage::Projects,
            project_filter: ProjectFilterMode::All,
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: String::new(),
            selected_project_path: None,
            task_status: TaskStatus::idle(),
            asset_catalog: Vec::new(),
            learn_catalog: Vec::new(),
            plugin_catalog: Vec::new(),
            team_overview: TeamOverview::empty(),
        };
        runtime.register_source_engine_from_settings();
        runtime.refresh_asset_catalog()?;
        runtime.refresh_learn_catalog()?;
        runtime.refresh_plugin_catalog()?;
        runtime.refresh_team_overview()?;
        runtime.persist()?;
        Ok(runtime)
    }

    fn snapshot(&self) -> HubSnapshot {
        HubSnapshot {
            selected_page: self.selected_page,
            project_filter: self.project_filter,
            project_sort: self.project_sort,
            project_view_mode: self.project_view_mode,
            search_query: self.search_query.clone(),
            selected_project_path: self.selected_project_path.clone(),
            task_status: self.task_status.clone(),
            recent_projects: self.config.recent_projects.clone(),
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

    fn search_projects(&mut self, query: &str) {
        self.search_query = query.to_string();
    }

    fn select_project_path(&mut self, project_path: &str) -> Result<(), HubError> {
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

    fn view_all_projects(&mut self) {
        self.search_query.clear();
        self.project_filter = ProjectFilterMode::All;
        self.project_view_mode = ProjectViewMode::Grid;
        self.task_status = TaskStatus {
            label: "All projects".to_string(),
            detail: "Showing all recent projects".to_string(),
            running: false,
        };
    }

    fn cycle_project_filter(&mut self) {
        self.project_filter = self.project_filter.next();
        self.task_status = TaskStatus {
            label: "Projects filtered".to_string(),
            detail: format!("Showing {}", self.project_filter.label()),
            running: false,
        };
    }

    fn cycle_project_sort(&mut self) {
        self.project_sort = self.project_sort.next();
        self.task_status = TaskStatus {
            label: "Projects sorted".to_string(),
            detail: format!("Sorting by {}", self.project_sort.label()),
            running: false,
        };
    }

    fn set_project_view_mode_by_id(&mut self, mode_id: &str) -> Result<(), HubError> {
        let Some(mode) = ProjectViewMode::from_id(mode_id) else {
            return Err(HubError::message(format!(
                "Unknown project view mode: {mode_id}"
            )));
        };
        self.project_view_mode = mode;
        Ok(())
    }

    fn select_engine_by_id(&mut self, ui: &HubWindow, engine_id: &str) -> Result<(), HubError> {
        self.sync_from_ui(ui);
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
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()?;
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
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()?;
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
        self.config.settings = binding::read_settings(ui, self.config.settings.clone());
    }

    fn save_settings(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.register_source_engine_from_settings();
        self.refresh_asset_catalog()?;
        self.refresh_learn_catalog()?;
        self.refresh_plugin_catalog()?;
        self.refresh_team_overview()?;
        self.persist()?;
        self.task_status = TaskStatus {
            label: "Settings saved".to_string(),
            detail: self.config_path.to_string_lossy().into_owned(),
            running: false,
        };
        Ok(())
    }

    fn open_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let project_path = PathBuf::from(ui.get_project_path().to_string());
        self.open_project_path(ui, project_path, None)
    }

    fn open_recent_project(&mut self, ui: &HubWindow, project_path: &str) -> Result<(), HubError> {
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

    fn open_project_path(
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
        self.ensure_editor_available(ui)?;
        let command = EditorLaunchCommand::from_preferred_engine(
            self.staged_engine_dir(),
            EditorLaunchRequest::OpenProject {
                project_path: project_path.clone(),
            },
        );
        launch_editor(&command)?;
        self.remember_project(RecentProject::with_now(display_name.clone(), project_path))?;
        self.task_status = TaskStatus {
            label: "Editor launched".to_string(),
            detail: format!("Opening {display_name}"),
            running: false,
        };
        Ok(())
    }

    fn quick_action(&mut self, ui: &HubWindow, action_id: &str) -> Result<(), HubError> {
        match HubQuickAction::from_id(action_id) {
            Some(HubQuickAction::BuildProject) => self.build_editor_runtime(ui),
            Some(HubQuickAction::OpenEditor) => self.open_selected_project_or_editor(ui),
            Some(HubQuickAction::PackageProject) => self.package_recent_project(ui),
            Some(HubQuickAction::InstallToDevice) => self.install_recent_project_to_device(ui),
            None => Err(HubError::message(format!(
                "Unknown quick action: {action_id}"
            ))),
        }
    }

    fn install_recent_project_to_device(&mut self, ui: &HubWindow) -> Result<(), HubError> {
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

    fn package_recent_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
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

    fn package_recent_project_to_output(
        &mut self,
        ui: &HubWindow,
    ) -> Result<crate::projects::ProjectPackageReport, HubError> {
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
        package_project(&request)
    }

    fn open_selected_project_or_editor(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let Some(project) = self.selected_recent_project() else {
            return self.launch_editor_without_project(ui);
        };
        let display_name = recent_project_display_name(&project);
        self.open_project_path(ui, project.path, Some(display_name))
    }

    fn create_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let request = CreateProjectRequest::new(
            ui.get_project_name().to_string(),
            PathBuf::from(ui.get_project_location().to_string()),
            ProjectTemplate::RenderableEmpty,
        );
        if request.project_name.trim().is_empty() {
            return Err(HubError::message("Project name is required"));
        }
        if request.location.as_os_str().is_empty() {
            return Err(HubError::message("Project location is required"));
        }
        let root = request.location.join(&request.project_name);
        let display_name = request.project_name.clone();
        self.ensure_editor_available(ui)?;
        let command = EditorLaunchCommand::from_preferred_engine(
            self.staged_engine_dir(),
            EditorLaunchRequest::CreateProject(request),
        );
        launch_editor(&command)?;
        self.remember_project(RecentProject::with_now(display_name, root))?;
        self.task_status = TaskStatus {
            label: "Editor launched".to_string(),
            detail: "Creating renderable empty project".to_string(),
            running: false,
        };
        Ok(())
    }

    fn build_editor_runtime(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.register_source_engine_from_settings();
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

    fn remember_project(&mut self, project: RecentProject) -> Result<(), HubError> {
        let last_project_path = project.path.clone();
        self.selected_project_path = Some(last_project_path.clone());
        self.config.recent_projects = merge_recent_projects(
            std::iter::once(project),
            self.config.recent_projects.clone(),
        );
        self.refresh_asset_catalog()?;
        self.persist_with_last_project(Some(&last_project_path))
    }

    fn selected_recent_project(&mut self) -> Option<RecentProject> {
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

    fn selected_or_latest_recent_project(&mut self) -> Option<RecentProject> {
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

    fn selected_project_label(&mut self) -> String {
        self.selected_recent_project()
            .map(|project| recent_project_display_name(&project))
            .unwrap_or_else(|| "Project".to_string())
    }

    fn register_source_engine_from_settings(&mut self) {
        let settings = &self.config.settings;
        if settings.default_source_dir.as_os_str().is_empty() {
            return;
        }
        let engine_id = source_engine_id(&settings.default_source_dir);
        let last_build_unix_ms = self
            .config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id)
            .and_then(|engine| engine.last_build_unix_ms);
        let existing = self
            .config
            .engines
            .iter()
            .find(|engine| engine.id == engine_id);
        let engine = SourceEngineInstall {
            id: engine_id.clone(),
            display_name: existing
                .map(|engine| engine.display_name.clone())
                .unwrap_or_else(|| source_engine_display_name(&settings.default_source_dir)),
            source_dir: settings.default_source_dir.clone(),
            output_dir: settings.default_build_output_dir.clone(),
            last_build_unix_ms,
            build_history: existing
                .map(|engine| engine.build_history.clone())
                .unwrap_or_default(),
        };
        upsert_source_engine(&mut self.config.engines, engine);
        self.config.active_engine_id = Some(engine_id);
        ensure_active_source_engine(&self.config.engines, &mut self.config.active_engine_id);
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

fn source_engine_id(source_dir: &std::path::Path) -> String {
    const FNV_OFFSET: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;
    let mut hash = FNV_OFFSET;
    for byte in source_dir.to_string_lossy().to_ascii_lowercase().bytes() {
        hash ^= u64::from(byte);
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    format!("source-{hash:016x}")
}

fn source_engine_display_name(source_dir: &std::path::Path) -> String {
    source_dir
        .file_name()
        .and_then(|name| name.to_str())
        .filter(|name| !name.trim().is_empty())
        .map(|name| format!("{name} Source"))
        .unwrap_or_else(|| "Local Source".to_string())
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

fn build_failure_detail(report: &BuildExecutionReport) -> String {
    report
        .stderr
        .lines()
        .last()
        .or_else(|| report.stdout.lines().last())
        .unwrap_or("build failed")
        .to_string()
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
    ui.on_cycle_project_filter(move || {
        with_runtime(&weak, &runtime_for_filter, |runtime, _ui| {
            runtime.cycle_project_filter();
            Ok(())
        })
    });

    let weak = ui.as_weak();
    let runtime_for_sort = Rc::clone(&runtime);
    ui.on_cycle_project_sort(move || {
        with_runtime(&weak, &runtime_for_sort, |runtime, _ui| {
            runtime.cycle_project_sort();
            Ok(())
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
            runtime.launch_editor_without_project(ui)
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
            search_query: String::new(),
            selected_project_path: None,
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
    fn quick_action_target_prefers_selected_project_over_latest_recent() {
        let mut runtime = runtime_with_projects(vec![
            RecentProject::new("Elysium", "E:/Projects/Elysium", 30),
            RecentProject::new("Stellar Outpost", "E:/Projects/StellarOutpost", 10),
        ]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/StellarOutpost"));

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
    fn view_all_projects_resets_filter_search_and_view_mode() {
        let mut runtime = runtime_with_projects(Vec::new());
        runtime.search_query = "stellar".to_string();
        runtime.project_filter = ProjectFilterMode::Missing;
        runtime.project_view_mode = ProjectViewMode::List;

        runtime.view_all_projects();

        assert!(runtime.search_query.is_empty());
        assert_eq!(runtime.project_filter, ProjectFilterMode::All);
        assert_eq!(runtime.project_view_mode, ProjectViewMode::Grid);
        assert_eq!(runtime.task_status.label, "All projects");
    }
}
