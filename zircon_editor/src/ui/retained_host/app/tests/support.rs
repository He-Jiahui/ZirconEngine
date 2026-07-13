pub(super) use std::cell::RefCell;
pub(super) use std::fs;
pub(super) use std::path::PathBuf;
pub(super) use std::rc::Rc;
pub(super) use std::time::{SystemTime, UNIX_EPOCH};

pub(super) use crate::core::editor_event::{
    ActivityDrawerMode as EventActivityDrawerMode, ActivityDrawerSlot as EventActivityDrawerSlot,
    EditorAssetEvent, EditorEvent, EditorEventTransient, EditorViewportEvent,
    LayoutCommand as EventLayoutCommand, MainPageId as EventMainPageId, MenuAction,
    ViewInstanceId as EventViewInstanceId,
};
pub(super) use crate::core::project::{RecentProjectEntry, RecentProjectValidation};
pub(super) use crate::scene::viewport::{DisplayMode, ViewOrientation};
pub(super) use crate::ui::host::module::{self, EDITOR_MANAGER_NAME};
use crate::ui::host::EditorHostEventController;
pub(super) use crate::ui::host::EditorManager;
pub(super) use crate::ui::retained_host::primitives::PhysicalSize;
pub(super) use crate::ui::retained_host::{PaneSurfaceHostContext, UiHostContext};
pub(super) use crate::ui::workbench::autolayout::ShellFrame;
pub(super) use crate::ui::workbench::layout::{
    ActivityDrawerMode, ActivityDrawerSlot, LayoutCommand, MainPageId,
};
pub(super) use crate::ui::workbench::startup::EditorSessionMode;
pub(super) use crate::ui::workbench::state::EditorState;
pub(super) use crate::ui::workbench::view::{ViewDescriptorId, ViewInstanceId};
pub(super) use winit::event::{ElementState, KeyEvent};
pub(super) use winit::keyboard::{
    Key, KeyCode, KeyLocation, ModifiersState, NamedKey, PhysicalKey,
};
pub(super) use zircon_runtime::core::CoreRuntime;
pub(super) use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
pub(super) use zircon_runtime::scene::DefaultLevelManager;
pub(super) use zircon_runtime_interface::math::UVec2;

pub(super) use super::super::*;

pub(super) struct ChildWindowHostHarness {
    pub(super) _core: CoreRuntime,
    pub(super) config_path: PathBuf,
    pub(super) host: Rc<RefCell<RetainedEditorHost>>,
    pub(super) root_ui: UiHostWindow,
}

pub(super) fn pane_surface_host(ui: &UiHostWindow) -> PaneSurfaceHostContext<'_> {
    ui.global::<PaneSurfaceHostContext>()
}

pub(super) fn host_context(ui: &UiHostWindow) -> UiHostContext<'_> {
    ui.global::<UiHostContext>()
}

impl ChildWindowHostHarness {
    pub(super) fn new(prefix: &str) -> Self {
        let config_path = unique_temp_path(prefix);
        std::env::set_var("ZIRCON_CONFIG_PATH", &config_path);
        let core = CoreRuntime::new();
        core.register_module(foundation_module_descriptor())
            .unwrap();
        core.register_module(zircon_runtime::asset::module_descriptor())
            .unwrap();
        core.register_module(module::module_descriptor()).unwrap();
        core.activate_module(FOUNDATION_MODULE_NAME).unwrap();
        core.activate_module(zircon_runtime::asset::ASSET_MODULE_NAME)
            .unwrap();
        core.activate_module(module::EDITOR_MODULE_NAME).unwrap();
        std::env::remove_var("ZIRCON_CONFIG_PATH");

        let root_ui = UiHostWindow::new().expect("root workbench shell should instantiate");
        root_ui
            .show()
            .expect("root workbench shell should show in the test backend");

        let mut state = EditorState::with_default_selection(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
        );
        state.mark_project_open();
        let manager = core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        let host = Rc::new(RefCell::new(
            RetainedEditorHost::new_for_test(core.handle(), root_ui.clone_strong())
                .map(|mut host| {
                    host.runtime = EditorHostEventController::new(state, manager);
                    host.sync_asset_workspace();
                    host
                })
                .expect("retained editor host should build with test viewport controller"),
        ));
        wire_callbacks(&root_ui, &host);
        host.borrow_mut().self_handle = Some(Rc::downgrade(&host));
        host.borrow_mut().refresh_ui();

        Self {
            _core: core,
            config_path,
            host,
            root_ui,
        }
    }

    pub(super) fn detach_view_to_child_window(
        &self,
        instance_id: &str,
        window_id: &str,
    ) -> UiHostWindow {
        self.detach_views_to_child_window(&[instance_id], window_id)
    }

    pub(super) fn detach_views_to_child_window(
        &self,
        instance_ids: &[&str],
        window_id: &str,
    ) -> UiHostWindow {
        let window_id = MainPageId::new(window_id);
        {
            let mut host = self.host.borrow_mut();
            for instance_id in instance_ids {
                let result = callback_dispatch::dispatch_layout_command(
                    &host.runtime,
                    LayoutCommand::DetachViewToWindow {
                        instance_id: ViewInstanceId::new(*instance_id),
                        new_window: window_id.clone(),
                    },
                );
                host.apply_dispatch_result(result);
            }
            host.recompute_if_dirty();
        }

        self.host
            .borrow()
            .native_window_presenters
            .window(&window_id)
            .expect("detached view should create a child native window presenter")
    }

    pub(super) fn journal_len(&self) -> usize {
        self.host.borrow().runtime.journal().records().len()
    }

    pub(super) fn delta_events_since(&self, baseline: usize) -> Vec<EditorEvent> {
        self.host.borrow().runtime.journal().records()[baseline..]
            .iter()
            .map(|record| record.event.clone())
            .collect()
    }

    pub(super) fn open_view(&self, descriptor_id: &str) -> ViewInstanceId {
        let manager = self
            ._core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .expect("editor manager");
        let instance_id = manager
            .open_view(ViewDescriptorId::new(descriptor_id), None)
            .expect("view should open");
        let mut host = self.host.borrow_mut();
        host.mark_layout_dirty();
        host.refresh_ui();
        host.recompute_if_dirty();
        instance_id
    }

    pub(super) fn dispatch_menu_action(&self, action: &str) {
        let mut host = self.host.borrow_mut();
        let effects = callback_dispatch::dispatch_menu_action(&host.runtime, action)
            .expect("menu action dispatch should succeed");
        host.apply_dispatch_effects(effects);
        host.recompute_if_dirty();
    }

    pub(super) fn activate_workbench_page(&self) {
        let mut host = self.host.borrow_mut();
        host.runtime.set_session_mode(EditorSessionMode::Project);
        host.editor_manager
            .dismiss_welcome_page()
            .expect("welcome page should dismiss");
        host.mark_layout_dirty();
        host.refresh_ui();
        host.recompute_if_dirty();
    }

    pub(super) fn activate_drawer_tab(&self, slot: ActivityDrawerSlot, instance_id: &str) {
        let mut host = self.host.borrow_mut();
        let effects = callback_dispatch::dispatch_layout_command(
            &host.runtime,
            LayoutCommand::ActivateDrawerTab {
                slot,
                instance_id: ViewInstanceId::new(instance_id),
            },
        )
        .expect("drawer tab activation should succeed");
        host.apply_dispatch_effects(effects);
        let effects = callback_dispatch::dispatch_layout_command(
            &host.runtime,
            LayoutCommand::SetDrawerMode {
                slot,
                mode: ActivityDrawerMode::Pinned,
            },
        )
        .expect("drawer mode update should succeed");
        host.apply_dispatch_effects(effects);
        host.refresh_ui();
        host.recompute_if_dirty();
    }

    pub(super) fn stage_missing_recent_project(&self, path: &str, display_name: &str) {
        let mut host = self.host.borrow_mut();
        host.startup_session.recent_projects = vec![RecentProjectEntry {
            summary: zircon_runtime_interface::project::ProjectManifestSummary {
                name: display_name.to_string(),
                engine_version_req: None,
                default_scene: "res://scenes/main.scene.toml".to_string(),
                format_version: 2,
            },
            path: path.to_string(),
            last_opened_unix_ms: 1,
            validation: RecentProjectValidation::Missing,
        }];
        host.startup_session.status_message = "Choose a recent project or create a new one.".into();
        host.refresh_welcome_snapshot();
        host.refresh_ui();
    }
}

impl Drop for ChildWindowHostHarness {
    fn drop(&mut self) {
        let _ = self
            .host
            .borrow_mut()
            .native_window_presenters
            .sync_targets(&[], |_ui, _target| {}, |_ui, _target| {});
        let _ = self.root_ui.hide();
        let _ = fs::remove_file(&self.config_path);
    }
}

pub(super) fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

pub(super) fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

pub(super) fn key_event(
    logical_key: Key,
    physical_key: PhysicalKey,
    text: Option<&str>,
    state: ElementState,
) -> KeyEvent {
    KeyEvent {
        physical_key,
        logical_key: logical_key.clone(),
        text: text.map(Into::into),
        location: KeyLocation::Standard,
        state,
        repeat: false,
        text_with_all_modifiers: text.map(Into::into),
        key_without_modifiers: logical_key,
    }
}

pub(super) fn workbench_control_bool(
    host: &RetainedEditorHost,
    control_id: &str,
    property: &str,
) -> bool {
    host.workbench_window_bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .and_then(|metadata| metadata.attributes.get(property))
                .and_then(toml::Value::as_bool)
        })
        .unwrap_or(false)
}

pub(super) fn workbench_control_visibility(
    host: &RetainedEditorHost,
    control_id: &str,
) -> Option<zircon_runtime_interface::ui::tree::UiVisibility> {
    host.workbench_window_bridge
        .surface()
        .tree
        .nodes
        .values()
        .find_map(|node| {
            node.template_metadata
                .as_ref()
                .filter(|metadata| metadata.control_id.as_deref() == Some(control_id))
                .map(|_| node.visibility)
        })
}
