use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use slint::{ComponentHandle, Timer, TimerMode};
use zircon_asset::{ProjectManager, ProjectPaths};
use zircon_core::{ChannelReceiver, CoreHandle};
use zircon_graphics::EditorOrRuntimeFrame;
use zircon_manager::{
    AssetChangeRecord, AssetManager, EditorAssetChangeRecord,
    EditorAssetManager as EditorAssetManagerFacade, InputButton, InputEvent, InputManager,
    ManagerResolver, ResourceManager,
};
use zircon_math::{UVec2, Vec2};
use zircon_resource::{MaterialMarker, ModelMarker, ResourceHandle, ResourceLocator};
use zircon_scene::Scene;
use zircon_ui::UiBindingValue;

use crate::editor_event::{
    EditorAssetSurface, EditorAssetUtilityTab, EditorAssetViewMode, EditorEventRuntime,
    EditorInspectorEvent, EditorViewportEvent,
};
use crate::host::resource_access::resolve_ready_handle;
use crate::paths::canonical_model_source_path;
use crate::project::EditorProjectDocument;
use crate::snapshot::ViewContentKind;
use crate::{
    compute_workbench_shell_geometry, ActivityDrawerMode, ActivityDrawerSlot,
    EditorManager, EditorSessionMode, EditorStartupSessionDocument, EditorState,
    InspectorFieldChange, LayoutCommand, MainPageId, ShellRegionId, ShellSizePx, ViewInstanceId,
    WorkbenchChromeMetrics, WorkbenchShellGeometry, WorkbenchViewModel, EDITOR_MANAGER_NAME,
};

use super::callback_dispatch;
use super::drawer_resize::dispatch_resize_to_group;
use super::event_bridge::SlintDispatchEffects;
use super::tab_drag::{drop_group_label, resolve_tab_drop};
use super::ui::apply_presentation;
use super::viewport::SlintViewportController;
use super::WorkbenchShell;

pub fn run_editor(core: CoreHandle) -> Result<(), Box<dyn Error>> {
    slint::BackendSelector::new()
        .require_wgpu_27(slint::wgpu_27::WGPUConfiguration::default())
        .select()?;

    let ui = WorkbenchShell::new()?;
    let host = Rc::new(RefCell::new(SlintEditorHost::new(core, ui.clone_strong())?));
    wire_callbacks(&ui, &host);
    install_rendering_bridge(&ui, &host);

    host.borrow_mut().refresh_ui();

    let timer = Timer::default();
    let host_weak = Rc::downgrade(&host);
    timer.start(TimerMode::Repeated, Duration::from_millis(16), move || {
        if let Some(host) = host_weak.upgrade() {
            host.borrow_mut().tick();
        }
    });

    ui.run()?;
    timer.stop();
    Ok(())
}

struct SlintEditorHost {
    ui: WorkbenchShell,
    runtime: EditorEventRuntime,
    editor_manager: Arc<EditorManager>,
    viewport: SlintViewportController,
    asset_server: Arc<dyn AssetManager>,
    editor_asset_server: Arc<dyn EditorAssetManagerFacade>,
    resource_server: Arc<dyn ResourceManager>,
    asset_change_events: ChannelReceiver<AssetChangeRecord>,
    editor_asset_change_events: ChannelReceiver<EditorAssetChangeRecord>,
    startup_session: EditorStartupSessionDocument,
    input_manager: Arc<dyn InputManager>,
    viewport_cursor: Vec2,
    viewport_size: UVec2,
    active_layout_preset: Option<String>,
    shell_size: ShellSizePx,
    chrome_metrics: WorkbenchChromeMetrics,
    shell_geometry: Option<WorkbenchShellGeometry>,
    transient_region_preferred: BTreeMap<ShellRegionId, f32>,
    active_drawer_resize: Option<ActiveDrawerResize>,
    presentation_dirty: bool,
    layout_dirty: bool,
    window_metrics_dirty: bool,
    render_dirty: bool,
}

#[derive(Clone, Copy, Debug)]
struct ActiveDrawerResize {
    region: ShellRegionId,
    start_x: f32,
    start_y: f32,
    base_preferred: f32,
}

impl SlintEditorHost {
    fn new(core: CoreHandle, ui: WorkbenchShell) -> Result<Self, Box<dyn Error>> {
        let resolver = ManagerResolver::new(core.clone());
        let asset_server = resolver.asset()?;
        let editor_asset_server = resolver.editor_asset()?;
        let resource_server = resolver.resource()?;
        let input_manager = resolver.input()?;
        let editor_manager = core.resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)?;
        let asset_change_events = asset_server.subscribe_asset_changes();
        let editor_asset_change_events = editor_asset_server.subscribe_editor_asset_changes();

        let viewport_size = UVec2::new(1280, 720);
        let startup_session = editor_manager.resolve_startup_session()?;
        let state = build_startup_state(
            editor_manager.as_ref(),
            &startup_session,
            viewport_size,
        )?;
        let shell_size = ShellSizePx::new(
            ui.get_shell_width_px().max(1.0),
            ui.get_shell_height_px().max(1.0),
        );

        let mut host = Self {
            ui,
            runtime: EditorEventRuntime::new(state, editor_manager.clone()),
            editor_manager,
            viewport: SlintViewportController::new(core),
            asset_server,
            editor_asset_server,
            resource_server,
            asset_change_events,
            editor_asset_change_events,
            startup_session,
            input_manager,
            viewport_cursor: Vec2::ZERO,
            viewport_size,
            active_layout_preset: None,
            shell_size,
            chrome_metrics: WorkbenchChromeMetrics::default(),
            shell_geometry: None,
            transient_region_preferred: BTreeMap::new(),
            active_drawer_resize: None,
            presentation_dirty: true,
            layout_dirty: true,
            window_metrics_dirty: true,
            render_dirty: true,
        };
        host.sync_asset_workspace();
        Ok(host)
    }

    fn tick(&mut self) {
        if let Err(error) = self.refresh_project_assets() {
            self.set_status_line(error);
        }

        self.sync_shell_size();
        self.recompute_if_dirty();

        if self.render_dirty {
            if let Some(scene) = self.runtime.render_snapshot() {
                if let Err(error) = self.viewport.submit_frame(EditorOrRuntimeFrame {
                    scene,
                    viewport: self.runtime.viewport_state(),
                }) {
                    self.set_status_line(format!("Viewport submit failed: {error}"));
                }
            }
            self.render_dirty = false;
        }

        if let Some(image) = self.viewport.poll_image() {
            self.ui.set_viewport_image(image);
        }
        if let Some(error) = self.viewport.take_error() {
            self.set_status_line(error);
            self.recompute_if_dirty();
        }
    }

    fn refresh_ui(&mut self) {
        self.recompute_if_dirty();
    }

    fn build_chrome(&self) -> crate::EditorChromeSnapshot {
        self.runtime.chrome_snapshot()
    }

    fn sync_shell_size(&mut self) {
        let next = ShellSizePx::new(
            self.ui.get_shell_width_px().max(1.0),
            self.ui.get_shell_height_px().max(1.0),
        );
        if (next.width - self.shell_size.width).abs() <= 0.5
            && (next.height - self.shell_size.height).abs() <= 0.5
        {
            return;
        }
        self.shell_size = next;
        self.window_metrics_dirty = true;
        self.presentation_dirty = true;
    }

    fn recompute_if_dirty(&mut self) {
        if !self.presentation_dirty && !self.layout_dirty && !self.window_metrics_dirty {
            return;
        }

        let layout = self.runtime.current_layout();
        let descriptors = self.runtime.descriptors();
        let mut chrome = self.build_chrome();
        let mut model = WorkbenchViewModel::build(&chrome);
        let geometry = compute_workbench_shell_geometry(
            &model,
            &chrome,
            &layout,
            &descriptors,
            self.shell_size,
            &self.chrome_metrics,
            if self.transient_region_preferred.is_empty() {
                None
            } else {
                Some(&self.transient_region_preferred)
            },
        );

        if let Some(next_viewport_size) = viewport_size_from_frame(&geometry) {
            if next_viewport_size != self.viewport_size {
                self.viewport_size = next_viewport_size;
                self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
                    &self.runtime,
                    EditorViewportEvent::Resized {
                        width: self.viewport_size.x,
                        height: self.viewport_size.y,
                    },
                ));
                chrome = self.build_chrome();
                model = WorkbenchViewModel::build(&chrome);
            }
        }

        let preset_names = self.runtime.preset_names();
        apply_presentation(
            &self.ui,
            &model,
            &chrome,
            &geometry,
            &preset_names,
            self.active_layout_preset.as_deref(),
        );
        self.shell_geometry = Some(geometry);
        self.presentation_dirty = false;
        self.layout_dirty = false;
        self.window_metrics_dirty = false;
    }

    fn set_status_line(&mut self, message: impl Into<String>) {
        self.runtime.set_status_line(message);
        self.presentation_dirty = true;
    }

    fn apply_dispatch_effects(&mut self, effects: SlintDispatchEffects) {
        if effects.reset_active_layout_preset {
            self.active_layout_preset = None;
        }
        if effects.layout_dirty {
            self.layout_dirty = true;
        }
        if effects.render_dirty {
            self.render_dirty = true;
        }
        if effects.presentation_dirty {
            self.presentation_dirty = true;
        }
        if effects.sync_asset_workspace {
            self.sync_asset_workspace();
        }
    }

    fn apply_dispatch_result(&mut self, result: Result<SlintDispatchEffects, String>) {
        match result {
            Ok(effects) => self.apply_dispatch_effects(effects),
            Err(error) => self.set_status_line(error),
        }
    }

    fn mark_layout_dirty(&mut self) {
        self.layout_dirty = true;
        self.presentation_dirty = true;
    }

    fn mark_render_and_presentation_dirty(&mut self) {
        self.render_dirty = true;
        self.presentation_dirty = true;
    }

    fn handle_menu_action(&mut self, action: &str) {
        if let Some(name) = action.strip_prefix("SavePreset.") {
            let name = if name.is_empty() { "current" } else { name };
            match callback_dispatch::dispatch_layout_command(
                &self.runtime,
                LayoutCommand::SavePreset {
                    name: name.to_string(),
                },
            ) {
                Ok(effects) => {
                    self.active_layout_preset = Some(name.to_string());
                    self.apply_dispatch_effects(effects);
                    self.set_status_line(format!("Saved layout preset asset {name}"));
                }
                Err(error) => self.set_status_line(error),
            }
            return;
        }

        if let Some(name) = action.strip_prefix("LoadPreset.") {
            match callback_dispatch::dispatch_layout_command(
                &self.runtime,
                LayoutCommand::LoadPreset {
                    name: name.to_string(),
                },
            ) {
                Ok(effects) => {
                    self.active_layout_preset = Some(name.to_string());
                    self.apply_dispatch_effects(effects);
                    self.set_status_line(format!("Loaded layout preset {name}"));
                }
                Err(error) => self.set_status_line(error),
            }
            return;
        }

        match action {
            "OpenProject" => {
                if let Err(error) = self.present_welcome_surface(
                    "Open an existing project or create a renderable empty project.",
                ) {
                    self.set_status_line(error);
                }
            }
            "ResetLayout" => match callback_dispatch::dispatch_menu_action(&self.runtime, action) {
                Ok(effects) => {
                    self.active_layout_preset = None;
                    self.apply_dispatch_effects(effects);
                }
                Err(error) => self.set_status_line(error),
            },
            "OpenScene" | "CreateScene" => {
                self.set_status_line("Scene open/create workflow is not wired yet");
            }
            _ => self.apply_dispatch_result(callback_dispatch::dispatch_menu_action(
                &self.runtime,
                action,
            )),
        }
    }

    fn refresh_welcome_snapshot(&mut self) {
        let snapshot = self.startup_session.welcome_pane_snapshot(false);
        self.runtime.set_welcome_snapshot(snapshot);
        self.presentation_dirty = true;
    }

    fn present_welcome_surface(&mut self, status_message: impl Into<String>) -> Result<(), String> {
        self.startup_session.recent_projects = self
            .editor_manager
            .recent_projects_snapshot()
            .map_err(|error| error.to_string())?;
        self.startup_session.status_message = status_message.into();
        self.editor_manager
            .show_welcome_page()
            .map_err(|error| error.to_string())?;
        if !self.runtime.editor_snapshot().project_open {
            self.runtime.set_session_mode(EditorSessionMode::Welcome);
        }
        self.refresh_welcome_snapshot();
        Ok(())
    }

    fn apply_startup_session(&mut self, mut session: EditorStartupSessionDocument) -> Result<(), String> {
        let welcome_snapshot = session.welcome_pane_snapshot(false);
        let status_message = session.status_message.clone();
        let mode = session.mode;
        let project = session.project.take();
        self.startup_session = session;

        match (mode, project) {
            (EditorSessionMode::Project, Some(document)) => {
                self.editor_manager
                    .apply_project_workspace(document.editor_workspace.clone())
                    .map_err(|error| error.to_string())?;
                let level = self
                    .editor_manager
                    .create_runtime_level(document.world)
                    .map_err(|error| error.to_string())?;
                self.runtime.replace_world(
                    level,
                    document.root_path.to_string_lossy().into_owned(),
                );
                self.runtime.set_session_mode(EditorSessionMode::Project);
                self.runtime.set_welcome_snapshot(welcome_snapshot);
                self.editor_manager
                    .dismiss_welcome_page()
                    .map_err(|error| error.to_string())?;
                self.sync_asset_workspace();
                self.mark_render_and_presentation_dirty();
            }
            (EditorSessionMode::Welcome, _) => {
                self.runtime.set_session_mode(EditorSessionMode::Welcome);
                self.runtime.set_welcome_snapshot(welcome_snapshot);
                self.editor_manager
                    .show_welcome_page()
                    .map_err(|error| error.to_string())?;
                self.presentation_dirty = true;
            }
            (EditorSessionMode::Project, None) => {
                return Err("startup session is missing project document".to_string());
            }
        }

        self.set_status_line(status_message);
        Ok(())
    }

    fn update_welcome_project_name(&mut self, value: &str) {
        self.startup_session.draft.project_name = value.to_string();
        self.refresh_welcome_snapshot();
    }

    fn update_welcome_location(&mut self, value: &str) {
        self.startup_session.draft.location = value.to_string();
        self.refresh_welcome_snapshot();
    }

    fn create_project_from_welcome(&mut self) {
        match self
            .editor_manager
            .create_project_and_open(self.startup_session.draft.clone())
            .map_err(|error| error.to_string())
            .and_then(|session| self.apply_startup_session(session))
        {
            Ok(()) => {}
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.refresh_welcome_snapshot();
                self.set_status_line(error);
            }
        }
    }

    fn open_existing_project_from_welcome(&mut self) {
        let result = self
            .startup_session
            .draft
            .validate_for_open_existing()
            .map_err(|error| error.to_string())
            .and_then(|root| {
                self.editor_manager
                    .open_project_and_remember(root)
                    .map_err(|error| error.to_string())
            })
            .and_then(|session| self.apply_startup_session(session));
        if let Err(error) = result {
            self.startup_session.status_message = error.clone();
            self.refresh_welcome_snapshot();
            self.set_status_line(error);
        }
    }

    fn open_recent_project(&mut self, path: &str) {
        let result = self
            .editor_manager
            .open_project_and_remember(path)
            .map_err(|error| error.to_string())
            .and_then(|session| self.apply_startup_session(session));
        if let Err(error) = result {
            self.startup_session.status_message = error.clone();
            if let Ok(recent_projects) = self.editor_manager.recent_projects_snapshot() {
                self.startup_session.recent_projects = recent_projects;
            }
            self.refresh_welcome_snapshot();
            self.set_status_line(error);
        }
    }

    fn remove_recent_project(&mut self, path: &str) {
        match self
            .editor_manager
            .forget_recent_project(path)
            .map_err(|error| error.to_string())
            .and_then(|_| {
                self.editor_manager
                    .recent_projects_snapshot()
                    .map_err(|error| error.to_string())
            }) {
            Ok(recent_projects) => {
                self.startup_session.recent_projects = recent_projects;
                self.startup_session.status_message = format!("Removed recent project {path}");
                self.refresh_welcome_snapshot();
                self.set_status_line(format!("Removed recent project {path}"));
            }
            Err(error) => {
                self.startup_session.status_message = error.clone();
                self.refresh_welcome_snapshot();
                self.set_status_line(error);
            }
        }
    }

    fn activate_host_page(&mut self, page_id: &str) {
        self.apply_dispatch_result(callback_dispatch::dispatch_layout_command(
            &self.runtime,
            LayoutCommand::ActivateMainPage {
                page_id: MainPageId::new(page_id),
            },
        ));
    }

    fn toggle_drawer_tab(&mut self, slot: &str, tab_id: &str) {
        let Ok(slot) = parse_drawer_slot(slot) else {
            self.set_status_line(format!("Unknown drawer slot {slot}"));
            return;
        };
        let layout = self.runtime.current_layout();
        let Some(drawer) = layout.drawers.get(&slot).cloned() else {
            return;
        };
        let instance_id = ViewInstanceId::new(tab_id);
        let is_active = drawer
            .tab_stack
            .active_tab
            .as_ref()
            .is_some_and(|active| active == &instance_id);

        if is_active && drawer.mode != ActivityDrawerMode::Collapsed {
            self.apply_dispatch_result(callback_dispatch::dispatch_layout_command(
                &self.runtime,
                LayoutCommand::SetDrawerMode {
                    slot,
                    mode: ActivityDrawerMode::Collapsed,
                },
            ));
        } else {
            self.apply_dispatch_result(callback_dispatch::dispatch_layout_command(
                &self.runtime,
                LayoutCommand::ActivateDrawerTab {
                    slot,
                    instance_id: instance_id.clone(),
                },
            ));
            if drawer.mode == ActivityDrawerMode::Collapsed {
                self.apply_dispatch_result(callback_dispatch::dispatch_layout_command(
                    &self.runtime,
                    LayoutCommand::SetDrawerMode {
                        slot,
                        mode: ActivityDrawerMode::Pinned,
                    },
                ));
            }
        }
    }

    fn activate_document_tab(&mut self, tab_id: &str) {
        self.apply_dispatch_result(callback_dispatch::dispatch_layout_command(
            &self.runtime,
            LayoutCommand::FocusView {
                instance_id: ViewInstanceId::new(tab_id),
            },
        ));
    }

    fn close_tab(&mut self, tab_id: &str) {
        match callback_dispatch::dispatch_layout_command(
            &self.runtime,
            LayoutCommand::CloseView {
                instance_id: ViewInstanceId::new(tab_id),
            },
        ) {
            Ok(effects) => {
                self.apply_dispatch_effects(effects);
                self.set_status_line("Closed tab");
            }
            Err(error) => self.set_status_line(error),
        }
    }

    fn drop_tab(&mut self, tab_id: &str, target_group: &str, pointer_x: f32, pointer_y: f32) {
        self.recompute_if_dirty();

        let layout = self.runtime.current_layout();
        let chrome = self.build_chrome();
        let model = WorkbenchViewModel::build(&chrome);
        let Some(resolved) = self.shell_geometry.as_ref().and_then(|geometry| {
            resolve_tab_drop(
                &layout,
                &model,
                geometry,
                &self.chrome_metrics,
                tab_id,
                target_group,
                pointer_x,
                pointer_y,
            )
        }) else {
            self.set_status_line(format!("Unsupported drop target {target_group}"));
            return;
        };

        match callback_dispatch::dispatch_layout_command(
            &self.runtime,
            LayoutCommand::AttachView {
                instance_id: ViewInstanceId::new(tab_id),
                target: resolved.host.clone(),
                anchor: resolved.anchor.clone(),
            },
        ) {
            Ok(effects) => {
                self.apply_dispatch_effects(effects);
                if let crate::ViewHost::Drawer(slot) = resolved.host {
                    self.apply_dispatch_result(callback_dispatch::dispatch_layout_command(
                        &self.runtime,
                        LayoutCommand::SetDrawerMode {
                            slot,
                            mode: ActivityDrawerMode::Pinned,
                        },
                    ));
                }
                self.set_status_line(format!(
                    "Moved {} to {}",
                    tab_id,
                    drop_group_label(target_group)
                ));
            }
            Err(error) => self.set_status_line(error),
        }
    }

    fn begin_drawer_resize(&mut self, target_group: &str, x: f32, y: f32) {
        self.recompute_if_dirty();

        let Ok(region) = parse_shell_region_group(target_group) else {
            self.set_status_line(format!("Unsupported drawer resize target {target_group}"));
            return;
        };
        let Some(geometry) = self.shell_geometry.as_ref() else {
            return;
        };
        let frame = geometry.region_frame(region);
        let base_preferred = match region {
            ShellRegionId::Bottom => frame.height,
            ShellRegionId::Left | ShellRegionId::Right | ShellRegionId::Document => frame.width,
        };
        if base_preferred <= 0.0 {
            return;
        }

        self.active_drawer_resize = Some(ActiveDrawerResize {
            region,
            start_x: x,
            start_y: y,
            base_preferred,
        });
        self.update_drawer_resize(x, y);
    }

    fn update_drawer_resize(&mut self, x: f32, y: f32) {
        let Some(active) = self.active_drawer_resize else {
            return;
        };
        let preferred = match active.region {
            ShellRegionId::Left => active.base_preferred + (x - active.start_x),
            ShellRegionId::Right => active.base_preferred - (x - active.start_x),
            ShellRegionId::Bottom => active.base_preferred - (y - active.start_y),
            ShellRegionId::Document => active.base_preferred,
        }
        .max(0.0);

        self.transient_region_preferred
            .insert(active.region, preferred);
        self.mark_layout_dirty();
        self.recompute_if_dirty();
    }

    fn finish_drawer_resize(&mut self, x: f32, y: f32) {
        self.update_drawer_resize(x, y);

        let Some(active) = self.active_drawer_resize.take() else {
            return;
        };
        let preferred = self
            .transient_region_preferred
            .get(&active.region)
            .copied()
            .unwrap_or(active.base_preferred);
        self.transient_region_preferred.remove(&active.region);

        match dispatch_resize_to_group(
            &self.runtime,
            shell_region_group_key(active.region),
            preferred,
        ) {
            Ok(effects) => {
                self.apply_dispatch_effects(effects);
                if !self.layout_dirty {
                    self.presentation_dirty = true;
                }
            }
            Err(error) => self.set_status_line(error),
        }

        self.recompute_if_dirty();
    }

    fn select_hierarchy_node(&mut self, node_id: &str) {
        match node_id.parse::<zircon_scene::NodeId>() {
            Ok(node_id) => self.apply_dispatch_result(
                callback_dispatch::dispatch_hierarchy_selection(&self.runtime, node_id),
            ),
            Err(error) => self.set_status_line(format!("Invalid node id {node_id}: {error}")),
        }
    }

    fn apply_inspector_changes(&mut self) {
        let Some(inspector) = self.runtime.editor_snapshot().inspector else {
            self.set_status_line("Nothing selected");
            return;
        };
        let parent_value = if inspector.parent.trim().is_empty() {
            UiBindingValue::Null
        } else {
            UiBindingValue::string(inspector.parent.clone())
        };
        let event = EditorInspectorEvent {
            subject_path: "entity://selected".to_string(),
            changes: vec![
                InspectorFieldChange::new("name", UiBindingValue::string(inspector.name)),
                InspectorFieldChange::new("parent", parent_value),
                InspectorFieldChange::new(
                    "transform.translation.x",
                    UiBindingValue::string(inspector.translation[0].clone()),
                ),
                InspectorFieldChange::new(
                    "transform.translation.y",
                    UiBindingValue::string(inspector.translation[1].clone()),
                ),
                InspectorFieldChange::new(
                    "transform.translation.z",
                    UiBindingValue::string(inspector.translation[2].clone()),
                ),
            ],
        };
        self.apply_dispatch_result(callback_dispatch::dispatch_inspector_apply(
            &self.runtime,
            event,
        ));
    }

    fn viewport_moved(&mut self, x: f32, y: f32) {
        self.viewport_cursor = Vec2::new(x, y);
        self.input_manager
            .submit_event(InputEvent::CursorMoved { x, y });
        self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::PointerMoved { x, y },
        ));
    }

    fn viewport_left_pressed(&mut self, x: f32, y: f32) {
        self.viewport_cursor = Vec2::new(x, y);
        self.input_manager
            .submit_event(InputEvent::ButtonPressed(InputButton::MouseLeft));
        self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::LeftPressed { x, y },
        ));
    }

    fn viewport_left_released(&mut self) {
        self.input_manager
            .submit_event(InputEvent::ButtonReleased(InputButton::MouseLeft));
        self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::LeftReleased,
        ));
    }

    fn viewport_right_pressed(&mut self, x: f32, y: f32) {
        self.viewport_cursor = Vec2::new(x, y);
        self.input_manager
            .submit_event(InputEvent::ButtonPressed(InputButton::MouseRight));
        self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::RightPressed { x, y },
        ));
    }

    fn viewport_right_released(&mut self) {
        self.input_manager
            .submit_event(InputEvent::ButtonReleased(InputButton::MouseRight));
        self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::RightReleased,
        ));
    }

    fn viewport_middle_pressed(&mut self, x: f32, y: f32) {
        self.viewport_cursor = Vec2::new(x, y);
        self.input_manager
            .submit_event(InputEvent::ButtonPressed(InputButton::MouseMiddle));
        self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::MiddlePressed { x, y },
        ));
    }

    fn viewport_middle_released(&mut self) {
        self.input_manager
            .submit_event(InputEvent::ButtonReleased(InputButton::MouseMiddle));
        self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::MiddleReleased,
        ));
    }

    fn viewport_scrolled(&mut self, delta: f32) {
        self.input_manager
            .submit_event(InputEvent::WheelScrolled { delta });
        self.apply_dispatch_result(callback_dispatch::dispatch_viewport_event(
            &self.runtime,
            EditorViewportEvent::Scrolled { delta },
        ));
    }

    fn refresh_project_assets(&mut self) -> Result<(), String> {
        let mut changes = Vec::new();
        while let Ok(change) = self.asset_change_events.try_recv() {
            changes.push(change);
        }
        let mut editor_changes = Vec::new();
        while let Ok(change) = self.editor_asset_change_events.try_recv() {
            editor_changes.push(change);
        }
        if changes.is_empty() && editor_changes.is_empty() {
            return Ok(());
        }

        self.sync_asset_workspace();
        let scene_changed = self.asset_server.current_project().is_some_and(|project| {
            changes
                .iter()
                .any(|change| change.uri == project.default_scene_uri)
        });
        if scene_changed {
            self.reload_default_scene()?;
        }
        self.mark_render_and_presentation_dirty();
        Ok(())
    }

    fn reload_default_scene(&mut self) -> Result<(), String> {
        let project_info = self
            .asset_server
            .current_project()
            .ok_or_else(|| "no directory project is currently open".to_string())?;
        let mut project =
            ProjectManager::open(&project_info.root_path).map_err(|error| error.to_string())?;
        project
            .scan_and_import()
            .map_err(|error| error.to_string())?;
        let scene_uri = ResourceLocator::parse(&project_info.default_scene_uri)
            .map_err(|error| error.to_string())?;
        let world =
            Scene::load_scene_from_uri(&project, &scene_uri).map_err(|error| error.to_string())?;
        let level = self
            .editor_manager
            .create_runtime_level(world)
            .map_err(|error| error.to_string())?;
        self.runtime.replace_world(level, project_info.root_path);
        Ok(())
    }

    fn import_model_into_project(&mut self) -> Result<(), String> {
        let chrome = self.build_chrome();
        let project = self
            .asset_server
            .current_project()
            .ok_or_else(|| "Open a project before importing models".to_string())?;
        EditorProjectDocument::ensure_runtime_assets(&project.root_path)
            .map_err(|error| error.to_string())?;

        let source = canonical_model_source_path(&chrome.mesh_import_path)?;
        let paths =
            ProjectPaths::from_root(&project.root_path).map_err(|error| error.to_string())?;
        let (model_uri, display_path) = stage_model_source(&paths, &source)?;

        self.asset_server
            .import_asset(&model_uri.to_string())
            .map_err(|error| error.to_string())?;
        let material_id = self.default_project_material_id()?;
        self.sync_asset_workspace();
        let model_id =
            resolve_ready_handle::<ModelMarker>(self.resource_server.as_ref(), &model_uri)?;
        if self
            .runtime
            .import_mesh_asset(model_id, material_id, display_path)?
        {
            self.mark_render_and_presentation_dirty();
        } else {
            self.presentation_dirty = true;
        }
        Ok(())
    }

    fn default_project_material_id(&self) -> Result<ResourceHandle<MaterialMarker>, String> {
        let material_uri = ResourceLocator::parse("res://materials/default.material.toml")
            .map_err(|error| error.to_string())?;
        self.asset_server
            .import_asset(&material_uri.to_string())
            .map_err(|error| error.to_string())?;
        resolve_ready_handle::<MaterialMarker>(self.resource_server.as_ref(), &material_uri)
    }

    fn sync_asset_workspace(&mut self) {
        self.runtime
            .sync_asset_catalog(self.editor_asset_server.catalog_snapshot());
        self.runtime
            .sync_asset_resources(self.resource_server.list_resources());
        self.refresh_selected_asset_details();
        self.refresh_visible_asset_previews();
        self.presentation_dirty = true;
    }

    fn refresh_selected_asset_details(&mut self) {
        let selected_uuid = self.runtime.editor_snapshot().asset_activity.selected_asset_uuid;
        self.runtime.sync_asset_details(
            selected_uuid
                .as_deref()
                .and_then(|uuid| self.editor_asset_server.asset_details(uuid)),
        );
    }

    fn refresh_visible_asset_previews(&mut self) {
        if self.asset_server.current_project().is_none() {
            return;
        }

        let chrome = self.build_chrome();
        let mut visible = BTreeSet::new();

        if asset_surface_visible(&chrome, ViewContentKind::Assets) {
            visible.extend(
                chrome
                    .asset_activity
                    .visible_assets
                    .iter()
                    .map(|asset| asset.uuid.clone()),
            );
            if let Some(uuid) = chrome.asset_activity.selection.uuid.clone() {
                visible.insert(uuid);
            }
        }

        if asset_surface_visible(&chrome, ViewContentKind::AssetBrowser) {
            visible.extend(
                chrome
                    .asset_browser
                    .visible_assets
                    .iter()
                    .map(|asset| asset.uuid.clone()),
            );
            if let Some(uuid) = chrome.asset_browser.selection.uuid.clone() {
                visible.insert(uuid);
            }
        }

        for uuid in visible {
            let _ = self
                .editor_asset_server
                .request_preview_refresh(&uuid, true);
        }
    }

    fn select_asset_folder(&mut self, folder_id: &str) {
        self.apply_dispatch_result(callback_dispatch::dispatch_asset_folder_selection(
            &self.runtime,
            folder_id.to_string(),
        ));
    }

    fn select_asset_item(&mut self, asset_uuid: &str) {
        self.apply_dispatch_result(callback_dispatch::dispatch_asset_item_selection(
            &self.runtime,
            asset_uuid.to_string(),
        ));
    }

    fn update_asset_search(&mut self, query: &str) {
        self.apply_dispatch_result(callback_dispatch::dispatch_asset_search(
            &self.runtime,
            query.to_string(),
        ));
    }

    fn update_asset_kind_filter(&mut self, kind: &str) {
        self.apply_dispatch_result(callback_dispatch::dispatch_asset_kind_filter(
            &self.runtime,
            Some(kind.to_string()),
        ));
    }

    fn update_asset_view_mode(&mut self, surface_mode: &str, mode: &str) {
        let Some(view_mode) = parse_asset_view_mode(mode) else {
            self.set_status_line(format!("Unknown asset view mode {mode}"));
            return;
        };

        let surface = match parse_asset_surface(surface_mode) {
            Some(surface) => surface,
            None => {
                self.set_status_line(format!("Unknown asset surface {surface_mode}"));
                return;
            }
        };

        self.apply_dispatch_result(callback_dispatch::dispatch_asset_view_mode(
            &self.runtime,
            surface,
            view_mode,
        ));
    }

    fn update_asset_utility_tab(&mut self, surface_mode: &str, tab: &str) {
        let Some(utility_tab) = parse_asset_utility_tab(tab) else {
            self.set_status_line(format!("Unknown asset utility tab {tab}"));
            return;
        };

        let surface = match parse_asset_surface(surface_mode) {
            Some(surface) => surface,
            None => {
                self.set_status_line(format!("Unknown asset surface {surface_mode}"));
                return;
            }
        };

        self.apply_dispatch_result(callback_dispatch::dispatch_asset_utility_tab(
            &self.runtime,
            surface,
            utility_tab,
        ));
    }

    fn activate_asset_reference(&mut self, asset_uuid: &str) {
        self.apply_dispatch_result(callback_dispatch::dispatch_asset_activate_reference(
            &self.runtime,
            asset_uuid.to_string(),
        ));
    }

    fn open_asset_browser(&mut self) {
        self.apply_dispatch_result(callback_dispatch::dispatch_open_asset_browser(
            &self.runtime,
        ));
    }

    fn locate_selected_asset(&mut self) {
        self.apply_dispatch_result(callback_dispatch::dispatch_locate_selected_asset(
            &self.runtime,
        ));
    }
}

fn asset_surface_visible(chrome: &crate::EditorChromeSnapshot, kind: ViewContentKind) -> bool {
    let Some(page) = chrome.workbench.main_pages.iter().find(|page| match page {
        crate::MainPageSnapshot::Workbench { id, .. }
        | crate::MainPageSnapshot::Exclusive { id, .. } => id == &chrome.workbench.active_main_page,
    }) else {
        return false;
    };

    match page {
        crate::MainPageSnapshot::Workbench { workspace, .. } => {
            let drawer_visible = chrome.workbench.drawers.values().any(|drawer| {
                drawer.visible
                    && drawer.mode != ActivityDrawerMode::Collapsed
                    && drawer
                        .active_tab
                        .as_ref()
                        .and_then(|active| {
                            drawer.tabs.iter().find(|tab| &tab.instance_id == active)
                        })
                        .or_else(|| drawer.tabs.first())
                        .is_some_and(|tab| tab.content_kind == kind)
            });
            drawer_visible
                || active_workspace_tab(workspace).is_some_and(|tab| tab.content_kind == kind)
        }
        crate::MainPageSnapshot::Exclusive { view, .. } => view.content_kind == kind,
    }
}

fn active_workspace_tab(
    workspace: &crate::DocumentWorkspaceSnapshot,
) -> Option<&crate::ViewTabSnapshot> {
    match workspace {
        crate::DocumentWorkspaceSnapshot::Split { first, second, .. } => {
            active_workspace_tab(first).or_else(|| active_workspace_tab(second))
        }
        crate::DocumentWorkspaceSnapshot::Tabs { tabs, active_tab } => active_tab
            .as_ref()
            .and_then(|active| tabs.iter().find(|tab| &tab.instance_id == active))
            .or_else(|| tabs.first()),
    }
}

fn parse_asset_surface(surface_mode: &str) -> Option<EditorAssetSurface> {
    match surface_mode {
        "activity" => Some(EditorAssetSurface::Activity),
        "browser" => Some(EditorAssetSurface::Browser),
        _ => None,
    }
}

fn parse_asset_view_mode(mode: &str) -> Option<EditorAssetViewMode> {
    match mode {
        "list" => Some(EditorAssetViewMode::List),
        "thumbnail" => Some(EditorAssetViewMode::Thumbnail),
        _ => None,
    }
}

fn parse_asset_utility_tab(tab: &str) -> Option<EditorAssetUtilityTab> {
    match tab {
        "preview" => Some(EditorAssetUtilityTab::Preview),
        "references" => Some(EditorAssetUtilityTab::References),
        "metadata" => Some(EditorAssetUtilityTab::Metadata),
        "plugins" => Some(EditorAssetUtilityTab::Plugins),
        _ => None,
    }
}

fn wire_callbacks(ui: &WorkbenchShell, host: &Rc<RefCell<SlintEditorHost>>) {
    let weak = Rc::downgrade(host);
    ui.on_menu_action(move |action| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().handle_menu_action(action.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_activate_host_page(move |page_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().activate_host_page(page_id.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_toggle_drawer_tab(move |slot, tab_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .toggle_drawer_tab(slot.as_str(), tab_id.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_activate_document_tab(move |tab_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().activate_document_tab(tab_id.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_close_tab(move |tab_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().close_tab(tab_id.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_drop_tab(move |tab_id, target_group, pointer_x, pointer_y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .drop_tab(tab_id.as_str(), target_group.as_str(), pointer_x, pointer_y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_begin_drawer_resize(move |target_group, x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .begin_drawer_resize(target_group.as_str(), x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_update_drawer_resize(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().update_drawer_resize(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_finish_drawer_resize(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().finish_drawer_resize(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_hierarchy_select(move |node_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().select_hierarchy_node(node_id.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_inspector_name_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow().runtime.update_name_field(value.to_string());
            host.borrow_mut().presentation_dirty = true;
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_inspector_parent_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow().runtime.update_parent_field(value.to_string());
            host.borrow_mut().presentation_dirty = true;
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_inspector_x_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow().runtime.update_translation_field(0, value.to_string());
            host.borrow_mut().presentation_dirty = true;
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_inspector_y_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow().runtime.update_translation_field(1, value.to_string());
            host.borrow_mut().presentation_dirty = true;
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_inspector_z_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow().runtime.update_translation_field(2, value.to_string());
            host.borrow_mut().presentation_dirty = true;
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_inspector_apply(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().apply_inspector_changes();
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_delete_selected(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().handle_menu_action("DeleteSelected");
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_mesh_import_path_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow().runtime.set_mesh_import_path(value.to_string());
            host.borrow_mut().presentation_dirty = true;
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_import_model(move || {
        if let Some(host) = weak.upgrade() {
            if let Err(error) = host.borrow_mut().import_model_into_project() {
                host.borrow_mut().set_status_line(error);
            }
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_asset_select_folder(move |folder_id| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().select_asset_folder(folder_id.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_asset_select_item(move |asset_uuid| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().select_asset_item(asset_uuid.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_asset_search_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().update_asset_search(value.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_asset_kind_filter_changed(move |kind| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().update_asset_kind_filter(kind.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_asset_view_mode_changed(move |surface_mode, mode| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .update_asset_view_mode(surface_mode.as_str(), mode.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_asset_utility_tab_changed(move |surface_mode, tab| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .update_asset_utility_tab(surface_mode.as_str(), tab.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_asset_activate_reference(move |asset_uuid| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut()
                .activate_asset_reference(asset_uuid.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_open_asset_browser(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().open_asset_browser();
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_locate_selected_asset(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().locate_selected_asset();
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_project_name_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().update_welcome_project_name(value.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_location_edited(move |value| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().update_welcome_location(value.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_create_project(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().create_project_from_welcome();
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_open_existing_project(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().open_existing_project_from_welcome();
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_open_recent_project(move |path| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().open_recent_project(path.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_welcome_remove_recent_project(move |path| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().remove_recent_project(path.as_str());
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_viewport_pointer_moved(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().viewport_moved(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_viewport_left_pressed(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().viewport_left_pressed(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_viewport_left_released(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().viewport_left_released();
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_viewport_right_pressed(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().viewport_right_pressed(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_viewport_right_released(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().viewport_right_released();
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_viewport_middle_pressed(move |x, y| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().viewport_middle_pressed(x, y);
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_viewport_middle_released(move || {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().viewport_middle_released();
        }
    });

    let weak = Rc::downgrade(host);
    ui.on_viewport_scrolled(move |delta| {
        if let Some(host) = weak.upgrade() {
            host.borrow_mut().viewport_scrolled(delta);
        }
    });
}

fn install_rendering_bridge(ui: &WorkbenchShell, host: &Rc<RefCell<SlintEditorHost>>) {
    let viewport = host.borrow().viewport.clone();
    let _ = ui
        .window()
        .set_rendering_notifier(move |state, api| match (state, api) {
            (
                slint::RenderingState::RenderingSetup,
                slint::GraphicsAPI::WGPU27 { device, queue, .. },
            ) => {
                let _ = viewport.attach_renderer(device.clone(), queue.clone());
            }
            (slint::RenderingState::RenderingTeardown, _) => viewport.detach_renderer(),
            _ => {}
        });
}

fn viewport_size_from_frame(geometry: &WorkbenchShellGeometry) -> Option<UVec2> {
    let width = geometry.viewport_content_frame.width.max(0.0).round() as u32;
    let height = geometry.viewport_content_frame.height.max(0.0).round() as u32;
    if width == 0 || height == 0 {
        None
    } else {
        Some(UVec2::new(width, height))
    }
}

fn parse_shell_region_group(target_group: &str) -> Result<ShellRegionId, String> {
    match target_group {
        "left" => Ok(ShellRegionId::Left),
        "right" => Ok(ShellRegionId::Right),
        "bottom" => Ok(ShellRegionId::Bottom),
        _ => Err(format!("Unsupported drawer resize target {target_group}")),
    }
}

fn shell_region_group_key(region: ShellRegionId) -> &'static str {
    match region {
        ShellRegionId::Left => "left",
        ShellRegionId::Right => "right",
        ShellRegionId::Bottom => "bottom",
        ShellRegionId::Document => "document",
    }
}

fn parse_drawer_slot(slot: &str) -> Result<ActivityDrawerSlot, String> {
    match slot {
        "left_top" => Ok(ActivityDrawerSlot::LeftTop),
        "left_bottom" => Ok(ActivityDrawerSlot::LeftBottom),
        "right_top" => Ok(ActivityDrawerSlot::RightTop),
        "right_bottom" => Ok(ActivityDrawerSlot::RightBottom),
        "bottom_left" => Ok(ActivityDrawerSlot::BottomLeft),
        "bottom_right" => Ok(ActivityDrawerSlot::BottomRight),
        _ => Err(format!("unknown drawer slot {slot}")),
    }
}

fn stage_model_source(
    paths: &ProjectPaths,
    source: &Path,
) -> Result<(ResourceLocator, String), String> {
    if let Ok(relative) = source.strip_prefix(paths.assets_root()) {
        let uri = asset_uri_from_relative_path(relative)?;
        return Ok((uri, source.to_string_lossy().into_owned()));
    }

    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if extension == "gltf" {
        return Err(
            "External .gltf import is not supported yet; copy the model folder into Project/assets or use .glb".to_string(),
        );
    }

    let destination = paths.assets_root().join("models").join(
        source
            .file_name()
            .ok_or_else(|| format!("model path has no file name: {}", source.display()))?,
    );
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    if source != destination {
        fs::copy(source, &destination).map_err(|error| {
            format!(
                "failed to copy model {} into project assets: {error}",
                source.display()
            )
        })?;
        if extension == "obj" {
            let sibling_mtl = source.with_extension("mtl");
            if sibling_mtl.exists() {
                let _ = fs::copy(sibling_mtl, destination.with_extension("mtl"));
            }
        }
    }

    Ok((
        asset_uri_from_relative_path(
            Path::new("models").join(destination.file_name().ok_or_else(|| {
                format!("model path has no file name: {}", destination.display())
            })?),
        )?,
        destination.to_string_lossy().into_owned(),
    ))
}

fn asset_uri_from_relative_path(relative: impl AsRef<Path>) -> Result<ResourceLocator, String> {
    let normalized = relative
        .as_ref()
        .components()
        .map(|component| component.as_os_str().to_string_lossy())
        .collect::<Vec<_>>()
        .join("/");
    ResourceLocator::parse(&format!("res://{normalized}")).map_err(|error| error.to_string())
}

fn build_startup_state(
    editor_manager: &EditorManager,
    session: &EditorStartupSessionDocument,
    viewport_size: UVec2,
) -> Result<EditorState, Box<dyn Error>> {
    let welcome = session.welcome_pane_snapshot(false);
    match session.mode {
        EditorSessionMode::Project => {
            let document = session
                .project
                .clone()
                .ok_or_else(|| "startup session is missing project document".to_string())?;
            editor_manager.apply_project_workspace(document.editor_workspace.clone())?;
            let level = editor_manager.create_runtime_level(document.world)?;
            let mut state = EditorState::project(
                level,
                viewport_size,
                document.root_path.to_string_lossy().into_owned(),
            );
            state.set_welcome_snapshot(welcome);
            state.set_status_line(session.status_message.clone());
            Ok(state)
        }
        EditorSessionMode::Welcome => {
            let mut state = EditorState::welcome(viewport_size, welcome);
            state.set_status_line(session.status_message.clone());
            Ok(state)
        }
    }
}
