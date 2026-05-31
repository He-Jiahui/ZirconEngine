use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;
use std::thread;

use crate::assets::AssetCatalogEntry;
use slint::{ComponentHandle, Timer, Weak};

use crate::build::BuildExecutionReport;
use crate::build::{run_build_command, BuildCommand, BuildCommandOptions};
use crate::engines::{
    active_source_engine, active_source_engine_mut, ensure_active_source_engine,
    prune_project_engine_bindings, remove_source_engine, upsert_source_engine,
    validate_source_engine, SourceBuildRecord, SourceEngineInstall, SourceEngineValidation,
};
use crate::error::HubError;
use crate::learn::LearnCatalogEntry;
use crate::plugins::PluginCatalogEntry;
use crate::process::{
    open_folder, preferred_editor_executable, preferred_editor_executable_exists, OpenFolderCommand,
};
use crate::settings::HubConfig;
use crate::state::{
    HubActionKind, HubActionRecord, HubActionStatus, HubPage, HubSnapshot, ProjectFilterMode,
    ProjectSortMode, ProjectSubpage, ProjectViewMode, TaskOperationKind, TaskSeverity, TaskStatus,
};
use crate::team::TeamOverview;

use self::source_engine_paths::{
    same_source_engine_path, source_engine_display_name, source_engine_id,
};
use super::binding;
use super::quick_action::HubQuickAction;
use super::HubWindow;

type BuildRunner = fn(&BuildCommand) -> Result<BuildExecutionReport, HubError>;

#[derive(Clone, Debug)]
struct PendingBuild {
    command: BuildCommand,
    command_line: Vec<String>,
    output_dir: PathBuf,
    engine_target: String,
    staged_engine_dir: PathBuf,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BuildExecutionMode {
    Background,
    Blocking,
}

mod action_failures;
mod asset_catalog;
mod folder_picker;
mod learn_catalog;
mod persistence;
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
            action_history: self.config.action_history.clone(),
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
        self.task_status = TaskStatus::success("Engine selected", engine.display_name.clone())
            .with_operation(TaskOperationKind::SourceEngine, engine.display_name);
        Ok(())
    }

    fn rename_active_engine(&mut self, ui: &HubWindow, name: &str) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let name = name.trim();
        if name.is_empty() {
            self.task_status = TaskStatus::warning(
                "Rename skipped",
                "Engine name cannot be empty",
                "Enter a display name before saving the Source Engine",
            );
            return Ok(());
        }
        let Some(engine) = active_source_engine_mut(
            &mut self.config.engines,
            self.config.active_engine_id.as_deref(),
        ) else {
            self.task_status = TaskStatus::warning(
                "No engine",
                "Configure a source checkout before renaming",
                "Use Settings > Source Checkout to register a Source Engine first",
            );
            return Ok(());
        };
        engine.display_name = name.to_string();
        self.persist_hub_config()?;
        self.task_status = TaskStatus::success("Engine renamed", name.to_string())
            .with_operation(TaskOperationKind::SourceEngine, name.to_string());
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
        self.task_status = TaskStatus::success("Engine removed", removed.display_name.clone())
            .with_operation(TaskOperationKind::SourceEngine, removed.display_name);
        Ok(())
    }

    fn cycle_active_engine(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        if self.config.engines.is_empty() {
            self.task_status = TaskStatus::warning(
                "No engines",
                "Configure a source checkout first",
                "Use Settings > Source Checkout to register a Source Engine",
            );
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
        self.task_status = TaskStatus::success(
            "Settings saved",
            self.config_path.to_string_lossy().into_owned(),
        );
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
        if let Err(error) = self.selected_or_latest_recent_project_with_engine_for_action() {
            let detail = error.to_string();
            self.record_build_action_failure(
                self.action_target_for_project_failure(),
                detail,
                Vec::new(),
                Some(self.config.settings.default_build_output_dir.clone()),
                "Select a valid project with a bound Source Engine before building",
            )?;
            return Err(error);
        }
        self.start_editor_runtime_build(ui, BuildExecutionMode::Background)
    }

    fn build_selected_project_engine_only(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        if let Err(error) = self.selected_project_with_engine_for_named_action(
            "Select a project before building",
            "Selected project is no longer available to build",
        ) {
            let detail = error.to_string();
            self.record_build_action_failure(
                self.action_target_for_project_failure(),
                detail,
                Vec::new(),
                Some(self.config.settings.default_build_output_dir.clone()),
                "Select a valid project with a bound Source Engine before building",
            )?;
            return Err(error);
        }
        self.start_editor_runtime_build(ui, BuildExecutionMode::Background)
    }

    fn build_editor_runtime(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.start_editor_runtime_build(ui, BuildExecutionMode::Background)
    }

    fn start_editor_runtime_build(
        &mut self,
        ui: &HubWindow,
        mode: BuildExecutionMode,
    ) -> Result<(), HubError> {
        let pending_build = self.prepare_editor_runtime_build(ui)?;
        match mode {
            BuildExecutionMode::Background => {
                spawn_editor_runtime_build(
                    ui.as_weak(),
                    pending_build,
                    self.config_path.clone(),
                    run_build_command,
                );
                Ok(())
            }
            BuildExecutionMode::Blocking => {
                let result = run_build_command(&pending_build.command);
                self.complete_editor_runtime_build(pending_build, result)
            }
        }
    }

    fn prepare_editor_runtime_build(&mut self, ui: &HubWindow) -> Result<PendingBuild, HubError> {
        self.register_source_engine_from_settings();
        self.refresh_source_scoped_views()?;
        let command = BuildCommand::for_editor_runtime(&BuildCommandOptions::new(
            self.config.settings.python_path.clone(),
            self.config.settings.cargo_path.clone(),
            self.config.settings.default_source_dir.clone(),
            self.config.settings.default_build_output_dir.clone(),
            self.config.settings.build_profile,
            Some(self.config.settings.jobs),
        ));
        let command_line = command.command_line();
        self.validate_active_source_engine_for_build(command_line.clone())?;
        self.task_status = TaskStatus::running_operation(
            "Building",
            "Running tools/zircon_build.py",
            TaskOperationKind::Build,
            self.action_engine_target(),
        );
        binding::apply_snapshot(ui, &self.snapshot());
        Timer::single_shot(std::time::Duration::from_millis(1), || {});
        let output_dir = self.config.settings.default_build_output_dir.clone();
        Ok(PendingBuild {
            command,
            command_line,
            output_dir,
            engine_target: self.action_engine_target(),
            staged_engine_dir: self.staged_engine_dir(),
        })
    }

    fn complete_editor_runtime_build(
        &mut self,
        pending_build: PendingBuild,
        result: Result<BuildExecutionReport, HubError>,
    ) -> Result<(), HubError> {
        let PendingBuild {
            command_line,
            output_dir,
            engine_target,
            staged_engine_dir,
            ..
        } = pending_build;
        let report = match result {
            Ok(report) => report,
            Err(error) => {
                let detail = error.to_string();
                self.record_active_build(
                    false,
                    detail.clone(),
                    detail.clone(),
                    command_line.clone(),
                );
                self.record_action_and_persist(HubActionRecord {
                    finished_unix_ms: crate::projects::now_unix_ms(),
                    action: HubActionKind::BuildEditorRuntime,
                    status: HubActionStatus::Failed,
                    target: engine_target.clone(),
                    detail: detail.clone(),
                    log_excerpt: detail.clone(),
                    recovery: Some(
                        "Check Python, Cargo, and Source Checkout settings before retrying"
                            .to_string(),
                    ),
                    process_id: None,
                    command_line,
                    output_dir: Some(output_dir),
                })?;
                self.task_status = TaskStatus::error(
                    "Build failed",
                    detail,
                    "Check Python, Cargo, and Source Checkout settings before retrying",
                )
                .with_operation(TaskOperationKind::Build, engine_target);
                return Ok(());
            }
        };
        if !report.succeeded() {
            let detail = report.summary_line();
            self.record_active_build(
                false,
                detail.clone(),
                report.log_excerpt(),
                command_line.clone(),
            );
            self.record_action_and_persist(HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action: HubActionKind::BuildEditorRuntime,
                status: HubActionStatus::Failed,
                target: engine_target.clone(),
                detail: detail.clone(),
                log_excerpt: report.log_excerpt(),
                recovery: Some(report.recovery_hint()),
                process_id: None,
                command_line,
                output_dir: Some(output_dir),
            })?;
            self.task_status = TaskStatus::error(
                "Build failed",
                detail,
                "Open Build History and fix the first reported error before retrying",
            )
            .with_operation(TaskOperationKind::Build, engine_target);
            return Ok(());
        }
        self.record_active_build(
            true,
            "Staged editor/runtime payload".to_string(),
            report.log_excerpt(),
            command_line.clone(),
        );
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::BuildEditorRuntime,
            status: HubActionStatus::Success,
            target: engine_target.clone(),
            detail: "Staged editor/runtime payload".to_string(),
            log_excerpt: report.log_excerpt(),
            recovery: None,
            process_id: None,
            command_line,
            output_dir: Some(output_dir),
        })?;
        self.task_status = TaskStatus::success(
            "Build complete",
            staged_engine_dir.to_string_lossy().into_owned(),
        )
        .with_operation(TaskOperationKind::Build, engine_target);
        Ok(())
    }

    fn ensure_editor_available(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        if preferred_editor_executable_exists(self.staged_engine_dir()) {
            return Ok(());
        }
        self.sync_from_ui(ui);
        self.start_editor_runtime_build(ui, BuildExecutionMode::Blocking)
    }

    fn open_output(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        let command = OpenFolderCommand::new(self.staged_engine_dir());
        let command_line = command.command_line();
        let output_dir = self.config.settings.default_build_output_dir.clone();
        if let Err(error) = open_folder(&command) {
            let detail = error.to_string();
            let target = self.action_engine_target();
            let recovery = "Check that the staged build output directory exists and can be opened";
            self.record_action_and_persist(HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action: HubActionKind::OpenOutput,
                status: HubActionStatus::Failed,
                target: target.clone(),
                detail: detail.clone(),
                log_excerpt: String::new(),
                recovery: Some(recovery.to_string()),
                process_id: None,
                command_line: command_line.clone(),
                output_dir: Some(output_dir),
            })?;
            self.set_action_failure_status(HubActionKind::OpenOutput, target, detail, recovery);
            return Err(error);
        }
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::OpenOutput,
            status: HubActionStatus::Success,
            target: self.action_engine_target(),
            detail: self
                .config
                .settings
                .default_build_output_dir
                .to_string_lossy()
                .into_owned(),
            log_excerpt: String::new(),
            recovery: None,
            process_id: None,
            command_line,
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        self.task_status = TaskStatus::success(
            "Output opened",
            self.config
                .settings
                .default_build_output_dir
                .to_string_lossy()
                .into_owned(),
        )
        .with_operation(TaskOperationKind::Process, self.action_engine_target());
        Ok(())
    }

    fn launch_editor_without_project(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        if let Err(error) = self.ensure_editor_available(ui) {
            let detail = error.to_string();
            let target = "Editor without project".to_string();
            let recovery =
                "Build the editor/runtime payload or fix Source Engine settings before launching";
            self.record_action_and_persist(HubActionRecord {
                finished_unix_ms: crate::projects::now_unix_ms(),
                action: HubActionKind::OpenEditor,
                status: HubActionStatus::Failed,
                target: target.clone(),
                detail: detail.clone(),
                log_excerpt: String::new(),
                recovery: Some(recovery.to_string()),
                process_id: None,
                command_line: Vec::new(),
                output_dir: Some(self.config.settings.default_build_output_dir.clone()),
            })?;
            self.set_action_failure_status(HubActionKind::OpenEditor, target, detail, recovery);
            return Err(error);
        }
        let executable = preferred_editor_executable(self.staged_engine_dir());
        let command_line = vec![executable.to_string_lossy().into_owned()];
        let child = match std::process::Command::new(&executable).spawn() {
            Ok(child) => child,
            Err(error) => {
                let detail = error.to_string();
                let target = "Editor without project".to_string();
                let recovery = "Verify the staged zircon_editor executable exists and is runnable";
                self.record_action_and_persist(HubActionRecord {
                    finished_unix_ms: crate::projects::now_unix_ms(),
                    action: HubActionKind::OpenEditor,
                    status: HubActionStatus::Failed,
                    target: target.clone(),
                    detail: detail.clone(),
                    log_excerpt: String::new(),
                    recovery: Some(recovery.to_string()),
                    process_id: None,
                    command_line: command_line.clone(),
                    output_dir: Some(self.config.settings.default_build_output_dir.clone()),
                })?;
                self.set_action_failure_status(HubActionKind::OpenEditor, target, detail, recovery);
                return Err(error.into());
            }
        };
        let process_id = child.id();
        self.record_action_and_persist(HubActionRecord {
            finished_unix_ms: crate::projects::now_unix_ms(),
            action: HubActionKind::OpenEditor,
            status: HubActionStatus::Success,
            target: "Editor without project".to_string(),
            detail: format!("Started process {process_id}"),
            log_excerpt: String::new(),
            recovery: None,
            process_id: Some(process_id),
            command_line,
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        self.task_status = TaskStatus::success("Editor launched", format!("Process {process_id}"))
            .with_operation(TaskOperationKind::Process, "Editor without project");
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

    fn record_active_build(
        &mut self,
        success: bool,
        detail: String,
        log_excerpt: String,
        command_line: Vec<String>,
    ) {
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
                log_excerpt,
                command_line,
            });
        }
    }

    fn set_action_failure_status(
        &mut self,
        action: HubActionKind,
        target: String,
        detail: String,
        recovery: &str,
    ) {
        self.task_status = TaskStatus::error(
            format!("{} failed", action.label()),
            detail,
            recovery.to_string(),
        )
        .with_operation(action_operation_kind(action), target);
    }

    pub(super) fn action_engine_target(&self) -> String {
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

    fn validate_active_source_engine_for_build(
        &mut self,
        command_line: Vec<String>,
    ) -> Result<(), HubError> {
        let validation = validate_source_engine(&self.config.settings.default_source_dir);
        if validation == SourceEngineValidation::Valid {
            return Ok(());
        }
        let detail = validation.summary().to_string();
        let recovery = validation.recovery_hint().to_string();
        let target = self.action_engine_target();
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
            output_dir: Some(self.config.settings.default_build_output_dir.clone()),
        })?;
        self.task_status = TaskStatus::error("Source Engine invalid", detail, recovery)
            .with_operation(TaskOperationKind::SourceEngine, target);
        Err(HubError::message(self.task_status.detail_with_recovery()))
    }
}

fn action_operation_kind(action: HubActionKind) -> TaskOperationKind {
    match action {
        HubActionKind::BuildEditorRuntime => TaskOperationKind::Build,
        HubActionKind::OpenEditor | HubActionKind::OpenOutput => TaskOperationKind::Process,
        HubActionKind::PackageProject | HubActionKind::InstallProject => TaskOperationKind::Project,
    }
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
    match action(&mut runtime, &ui) {
        Ok(()) => {
            if let Err(error) = runtime.persist_hub_config() {
                runtime.task_status = TaskStatus::error(
                    "State save failed",
                    error.to_string(),
                    "Hub could not save the current page and selection state",
                );
            }
        }
        Err(error) => {
            if runtime.task_status.severity != TaskSeverity::Error {
                runtime.task_status = TaskStatus::error(
                    "Action failed",
                    error.to_string(),
                    "Review the message, correct the project or Source Engine configuration, and retry",
                );
            }
        }
    }
    binding::apply_snapshot(&ui, &runtime.snapshot());
}

fn spawn_editor_runtime_build(
    weak_ui: Weak<HubWindow>,
    pending_build: PendingBuild,
    config_path: PathBuf,
    build_runner: BuildRunner,
) {
    thread::spawn(move || {
        let result = build_runner(&pending_build.command);
        let _ = weak_ui.upgrade_in_event_loop(move |ui| {
            match HubRuntime::load_from_config_path(config_path.clone()) {
                Ok(mut runtime) => {
                    if let Err(error) = runtime.complete_editor_runtime_build(pending_build, result)
                    {
                        runtime.task_status = TaskStatus::error(
                            "Build failed",
                            error.to_string(),
                            "Review Build History and retry after correcting the build setup",
                        );
                    }
                    binding::apply_snapshot(&ui, &runtime.snapshot());
                }
                Err(error) => {
                    let mut fallback = HubRuntime::empty_for_error(config_path.clone());
                    fallback.task_status = TaskStatus::error(
                        "Build failed",
                        error.to_string(),
                        "Hub could not reload build state after the background task finished",
                    );
                    binding::apply_snapshot(&ui, &fallback.snapshot());
                }
            }
        });
    });
}

#[cfg(test)]
pub(super) mod tests {
    use super::*;
    use crate::projects::{ProjectTemplate, RecentProject};
    use crate::settings::HubConfig;
    use std::fs;

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

    pub(super) fn runtime_with_config_path(
        projects: Vec<RecentProject>,
        config_path: PathBuf,
    ) -> HubRuntime {
        HubRuntime {
            config_path,
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

    pub(super) fn temp_test_dir(prefix: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "{prefix}-{}-{}",
            std::process::id(),
            crate::projects::now_unix_ms()
        ));
        fs::create_dir_all(&path).unwrap();
        path
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
    fn quick_action_build_reports_missing_bound_source_engine_before_building() {
        let mut runtime = runtime_with_projects(vec![RecentProject::new(
            "Elysium",
            "E:/Projects/Elysium",
            30,
        )]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Elysium"));

        let error = runtime
            .selected_or_latest_recent_project_with_engine_for_action()
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Project has no bound Source Engine: Elysium"
        );
    }

    #[test]
    fn builds_page_build_requires_selected_project_bound_source_engine() {
        let mut runtime = runtime_with_projects(vec![RecentProject::new(
            "Elysium",
            "E:/Projects/Elysium",
            30,
        )]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Elysium"));

        let error = runtime
            .selected_project_with_engine_for_named_action(
                "Select a project before building",
                "Selected project is no longer available to build",
            )
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            "Project has no bound Source Engine: Elysium"
        );
    }

    #[test]
    fn builds_page_actions_report_stale_selected_project_without_quick_action_fallback() {
        let mut runtime = runtime_with_projects(vec![RecentProject::new(
            "Elysium",
            "E:/Projects/Elysium",
            30,
        )]);
        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Missing"));

        let open_error = runtime
            .selected_project_for_named_action(
                "Select a project before opening",
                "Selected project is no longer available to open",
            )
            .unwrap_err();
        assert_eq!(
            open_error.to_string(),
            "Selected project is no longer available to open"
        );

        runtime.selected_project_path = Some(PathBuf::from("E:/Projects/Missing"));
        let build_error = runtime
            .selected_project_with_engine_for_named_action(
                "Select a project before building",
                "Selected project is no longer available to build",
            )
            .unwrap_err();
        assert_eq!(
            build_error.to_string(),
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

    #[test]
    fn invalid_source_engine_sets_recoverable_task_status() {
        let temp = temp_test_dir("zircon-hub-invalid-source-action-history");
        let mut runtime = runtime_with_config_path(Vec::new(), temp.join("hub.toml"));
        runtime.config.settings.default_source_dir = PathBuf::from("E:/missing/ZirconEngine");

        let error = runtime
            .validate_active_source_engine_for_build(vec![
                "python".to_string(),
                "tools/zircon_build.py".to_string(),
            ])
            .unwrap_err();

        assert_eq!(runtime.task_status.label, "Source Engine invalid");
        assert_eq!(
            runtime.task_status.severity,
            crate::state::TaskSeverity::Error
        );
        assert_eq!(
            runtime.task_status.recovery.as_deref(),
            Some("Locate an existing ZirconEngine checkout or update Settings > Source Checkout")
        );
        assert!(error
            .to_string()
            .contains("Source checkout directory is missing"));
        assert_eq!(runtime.config.action_history.len(), 1);
        let record = &runtime.config.action_history[0];
        assert_eq!(record.action, HubActionKind::BuildEditorRuntime);
        assert_eq!(record.status, HubActionStatus::Failed);
        assert!(record
            .detail
            .contains("Source checkout directory is missing"));
        assert!(record
            .command_line
            .contains(&"tools/zircon_build.py".to_string()));
        assert!(record
            .log_excerpt
            .contains("Source checkout directory is missing"));
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn runtime_prunes_stale_project_engine_bindings_without_losing_pins() {
        let project = PathBuf::from("E:/Projects/Game");
        let mut runtime = runtime_with_projects(vec![RecentProject::new("Game", &project, 1)]);
        runtime.config.project_metadata.insert(
            crate::projects::project_metadata_key(&project),
            crate::projects::ProjectMetadata {
                pinned: true,
                engine_id: Some("missing-source".to_string()),
                last_selected_template: None,
            },
        );

        let pruned = runtime.prune_stale_project_engine_bindings();

        assert_eq!(pruned, 1);
        let metadata = runtime
            .config
            .project_metadata
            .get(&crate::projects::project_metadata_key(&project))
            .expect("pinned metadata should remain after stale engine pruning");
        assert!(metadata.pinned);
        assert!(metadata.engine_id.is_none());
    }

    #[test]
    fn runtime_action_history_records_process_ids_and_keeps_newest() {
        let mut runtime = runtime_with_projects(Vec::new());

        for index in 0..18 {
            runtime.record_action(HubActionRecord {
                finished_unix_ms: index,
                action: HubActionKind::OpenEditor,
                status: HubActionStatus::Success,
                target: format!("Project {index}"),
                detail: format!("Started process {index}"),
                log_excerpt: String::new(),
                recovery: None,
                process_id: Some(index as u32),
                command_line: vec!["zircon_editor".to_string()],
                output_dir: None,
            });
        }

        assert_eq!(
            runtime.config.action_history.len(),
            crate::state::ACTION_HISTORY_LIMIT
        );
        assert_eq!(runtime.config.action_history[0].process_id, Some(17));
        assert_eq!(runtime.config.action_history[0].target, "Project 17");
    }

    #[test]
    fn package_failure_action_history_persists_and_reloads_from_config() {
        let temp = temp_test_dir("zircon-hub-package-action-history");
        let project = temp.join("Game");
        let config_path = temp.join("hub.toml");
        fs::create_dir_all(&project).unwrap();
        fs::write(project.join("zircon-project.toml"), "name = \"Game\"\n").unwrap();
        let mut runtime = runtime_with_config_path(
            vec![RecentProject::new("Game", &project, 1)],
            config_path.clone(),
        );
        runtime.selected_project_path = Some(project.clone());
        runtime.config.settings.default_build_output_dir = project.join("package-output-inside");

        let error = runtime.package_project_to_output(RecentProject::new("Game", &project, 1));

        assert!(error.is_err());
        let reloaded = HubConfig::load(&config_path).unwrap();
        assert_eq!(reloaded.action_history.len(), 1);
        let record = &reloaded.action_history[0];
        assert_eq!(record.action, HubActionKind::PackageProject);
        assert_eq!(record.status, HubActionStatus::Failed);
        assert_eq!(record.target, "Game");
        assert!(record
            .recovery
            .as_deref()
            .unwrap()
            .contains("package output"));
        assert_eq!(runtime.task_status.label, "Package Project failed");
        assert_eq!(runtime.task_status.operation_summary(), "Project: Game");
        assert!(runtime
            .task_status
            .detail_with_recovery()
            .contains("package output"));
        fs::remove_dir_all(temp).unwrap();
    }

    #[test]
    fn install_failure_action_history_persists_and_reloads_from_config() {
        let temp = temp_test_dir("zircon-hub-install-action-history");
        let config_path = temp.join("hub.toml");
        let mut runtime = runtime_with_config_path(Vec::new(), config_path.clone());
        runtime.config.settings.default_device_install_dir = temp.join("device");

        runtime
            .record_project_action_failure(
                HubActionKind::InstallProject,
                "Game".to_string(),
                "Package directory is not available".to_string(),
                "Check the package output and configured local device install directory before retrying",
                Some(runtime.config.settings.default_device_install_dir.clone()),
            )
            .unwrap();

        let reloaded = HubConfig::load(&config_path).unwrap();
        assert_eq!(
            reloaded.action_history[0].action,
            HubActionKind::InstallProject
        );
        assert_eq!(reloaded.action_history[0].status, HubActionStatus::Failed);
        assert!(reloaded.action_history[0]
            .recovery
            .as_deref()
            .unwrap()
            .contains("device install directory"));
        assert_eq!(runtime.task_status.label, "Install to Device failed");
        assert_eq!(runtime.task_status.operation_summary(), "Project: Game");
        assert!(runtime
            .task_status
            .detail_with_recovery()
            .contains("device install directory"));
        fs::remove_dir_all(temp).unwrap();
    }
}
