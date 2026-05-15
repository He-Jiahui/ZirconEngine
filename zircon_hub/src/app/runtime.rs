use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use slint::{ComponentHandle, PhysicalPosition};

use crate::build::{run_build_command, BuildCommand, BuildCommandOptions};
use crate::engines::{validate_source_engine, SourceEngineInstall, SourceEngineValidation};
use crate::error::HubError;
use crate::process::{
    launch_editor, open_folder, preferred_editor_executable, preferred_editor_executable_exists,
    EditorLaunchCommand, EditorLaunchRequest, OpenFolderCommand,
};
use crate::projects::{
    load_editor_recent_projects, merge_recent_projects, save_editor_recent_projects,
    save_editor_recent_projects_with_last_project, validate_project_root, CreateProjectRequest,
    ProjectTemplate, ProjectValidation, RecentProject,
};
use crate::settings::{default_hub_config_path, editor_config_path, HubConfig};
use crate::state::{HubPage, HubSnapshot, ProjectSortMode, ProjectViewMode, TaskStatus};

use super::binding;
use super::quick_action::HubQuickAction;
use super::HubWindow;

pub(super) fn run() -> Result<(), HubError> {
    let ui = HubWindow::new()?;
    let runtime = Rc::new(RefCell::new(HubRuntime::load()?));
    binding::apply_snapshot(&ui, &runtime.borrow().snapshot());
    wire_callbacks(&ui, runtime);
    ui.run()?;
    Ok(())
}

struct HubRuntime {
    config_path: PathBuf,
    editor_config_path: PathBuf,
    config: HubConfig,
    selected_page: HubPage,
    project_sort: ProjectSortMode,
    project_view_mode: ProjectViewMode,
    search_query: String,
    task_status: TaskStatus,
}

#[derive(Clone, Copy, Debug)]
struct WindowDragState {
    origin: PhysicalPosition,
    press_x: f32,
    press_y: f32,
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
            project_sort: ProjectSortMode::LastModified,
            project_view_mode: ProjectViewMode::Grid,
            search_query: String::new(),
            task_status: TaskStatus::idle(),
        };
        runtime.register_source_engine_from_settings();
        runtime.persist_hub_config()?;
        Ok(runtime)
    }

    fn snapshot(&self) -> HubSnapshot {
        HubSnapshot {
            selected_page: self.selected_page,
            project_sort: self.project_sort,
            project_view_mode: self.project_view_mode,
            search_query: self.search_query.clone(),
            task_status: self.task_status.clone(),
            recent_projects: self.config.recent_projects.clone(),
            engines: self.config.engines.clone(),
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

    fn sync_from_ui(&mut self, ui: &HubWindow) {
        self.search_query = ui.get_search_query().to_string();
        self.config.settings = binding::read_settings(ui, self.config.settings.clone());
    }

    fn save_settings(&mut self, ui: &HubWindow) -> Result<(), HubError> {
        self.sync_from_ui(ui);
        self.register_source_engine_from_settings();
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
            Some(HubQuickAction::OpenEditor) => self.launch_editor_without_project(ui),
            Some(action @ (HubQuickAction::InstallToDevice | HubQuickAction::PackageProject)) => {
                self.task_status = TaskStatus {
                    label: "Action unavailable".to_string(),
                    detail: action.unavailable_detail().to_string(),
                    running: false,
                };
                Ok(())
            }
            None => Err(HubError::message(format!(
                "Unknown quick action: {action_id}"
            ))),
        }
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
            self.config.settings.default_source_dir.clone(),
            self.config.settings.default_build_output_dir.clone(),
            self.config.settings.build_profile,
            Some(self.config.settings.jobs),
        ));
        let report = run_build_command(&command)?;
        if !report.succeeded() {
            self.task_status = TaskStatus {
                label: "Build failed".to_string(),
                detail: report
                    .stderr
                    .lines()
                    .last()
                    .unwrap_or("build failed")
                    .to_string(),
                running: false,
            };
            return Err(HubError::message(self.task_status.detail.clone()));
        }
        if let Some(engine) = self.config.engines.first_mut() {
            engine.last_build_unix_ms = Some(crate::projects::now_unix_ms());
        }
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
        self.config.recent_projects = merge_recent_projects(
            std::iter::once(project),
            self.config.recent_projects.clone(),
        );
        self.persist_with_last_project(Some(&last_project_path))
    }

    fn register_source_engine_from_settings(&mut self) {
        let settings = &self.config.settings;
        if settings.default_source_dir.as_os_str().is_empty() {
            return;
        }
        let engine = SourceEngineInstall {
            id: "local-source".to_string(),
            display_name: "Local Source".to_string(),
            source_dir: settings.default_source_dir.clone(),
            output_dir: settings.default_build_output_dir.clone(),
            last_build_unix_ms: self
                .config
                .engines
                .first()
                .and_then(|engine| engine.last_build_unix_ms),
        };
        self.config.engines = vec![engine];
    }

    fn staged_engine_dir(&self) -> PathBuf {
        self.config
            .settings
            .default_build_output_dir
            .join("ZirconEngine")
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

    wire_quick_actions(ui, Rc::clone(&runtime));
    wire_window_controls(ui);
}

fn wire_quick_actions(ui: &HubWindow, runtime: Rc<RefCell<HubRuntime>>) {
    let weak = ui.as_weak();
    ui.on_quick_action(move |action_id| {
        with_runtime(&weak, &runtime, |runtime, ui| {
            runtime.quick_action(ui, &action_id)
        })
    });
}

fn wire_window_controls(ui: &HubWindow) {
    let weak = ui.as_weak();
    ui.on_window_minimize(move || {
        if let Some(ui) = weak.upgrade() {
            ui.window().set_minimized(true);
        }
    });

    let weak = ui.as_weak();
    ui.on_window_toggle_maximize(move || {
        if let Some(ui) = weak.upgrade() {
            let maximized = ui.window().is_maximized();
            ui.window().set_maximized(!maximized);
        }
    });

    let weak = ui.as_weak();
    ui.on_window_close(move || {
        if let Some(ui) = weak.upgrade() {
            let _ = ui.window().hide();
        }
        let _ = slint::quit_event_loop();
    });

    let drag_state = Rc::new(RefCell::new(None::<WindowDragState>));
    let weak = ui.as_weak();
    let drag_for_start = Rc::clone(&drag_state);
    ui.on_window_drag_start(move |press_x, press_y| {
        if let Some(ui) = weak.upgrade() {
            *drag_for_start.borrow_mut() = Some(WindowDragState {
                origin: ui.window().position(),
                press_x,
                press_y,
            });
        }
    });

    let weak = ui.as_weak();
    let drag_for_move = Rc::clone(&drag_state);
    ui.on_window_drag_move(move |mouse_x, mouse_y| {
        let Some(ui) = weak.upgrade() else {
            return;
        };
        let Some(state) = *drag_for_move.borrow() else {
            return;
        };
        let scale = ui.window().scale_factor();
        let delta_x = ((mouse_x - state.press_x) * scale) as i32;
        let delta_y = ((mouse_y - state.press_y) * scale) as i32;
        ui.window().set_position(PhysicalPosition::new(
            state.origin.x + delta_x,
            state.origin.y + delta_y,
        ));
    });

    ui.on_window_drag_end(move || {
        *drag_state.borrow_mut() = None;
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
