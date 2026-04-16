use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use slint::{ComponentHandle, PhysicalSize};
use zircon_core::CoreRuntime;
use zircon_manager::MANAGER_MODULE_NAME;
use zircon_math::UVec2;
use zircon_scene::{DefaultLevelManager, DisplayMode};

use super::*;
use crate::{
    module, EditorAssetEvent, EditorDraftEvent, EditorEvent, EditorEventRuntime, EditorManager,
    EditorState, EditorViewportEvent, LayoutCommand, MainPageId, MenuAction, ViewDescriptorId,
    ViewInstanceId, EDITOR_MANAGER_NAME,
};

struct ChildWindowHostHarness {
    _core: CoreRuntime,
    config_path: PathBuf,
    host: Rc<RefCell<SlintEditorHost>>,
    root_ui: WorkbenchShell,
}

impl ChildWindowHostHarness {
    fn new(prefix: &str) -> Self {
        i_slint_backend_testing::init_no_event_loop();

        let config_path = unique_temp_path(prefix);
        std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", &config_path);
        let core = CoreRuntime::new();
        core.register_module(zircon_manager::module_descriptor())
            .unwrap();
        core.register_module(zircon_asset::module_descriptor())
            .unwrap();
        core.register_module(module::module_descriptor()).unwrap();
        core.activate_module(MANAGER_MODULE_NAME).unwrap();
        core.activate_module(zircon_asset::ASSET_MODULE_NAME)
            .unwrap();
        core.activate_module(module::EDITOR_MODULE_NAME).unwrap();
        std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");

        let root_ui = WorkbenchShell::new().expect("root workbench shell should instantiate");
        root_ui
            .show()
            .expect("root workbench shell should show in the test backend");

        let state = EditorState::with_default_selection(
            DefaultLevelManager::default().create_default_level(),
            UVec2::new(1280, 720),
        );
        let manager = core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .unwrap();
        let host = Rc::new(RefCell::new(
            SlintEditorHost::new_for_test(core.handle(), root_ui.clone_strong())
                .map(|mut host| {
                    host.runtime = EditorEventRuntime::new(state, manager);
                    host.sync_asset_workspace();
                    host
                })
                .expect("slint editor host should build with test viewport controller"),
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

    fn detach_view_to_child_window(&self, instance_id: &str, window_id: &str) -> WorkbenchShell {
        self.detach_views_to_child_window(&[instance_id], window_id)
    }

    fn detach_views_to_child_window(
        &self,
        instance_ids: &[&str],
        window_id: &str,
    ) -> WorkbenchShell {
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

    fn journal_len(&self) -> usize {
        self.host.borrow().runtime.journal().records().len()
    }

    fn delta_events_since(&self, baseline: usize) -> Vec<EditorEvent> {
        self.host.borrow().runtime.journal().records()[baseline..]
            .iter()
            .map(|record| record.event.clone())
            .collect()
    }

    fn open_view(&self, descriptor_id: &str) -> ViewInstanceId {
        let manager = self
            ._core
            .resolve_manager::<EditorManager>(EDITOR_MANAGER_NAME)
            .expect("editor manager");
        let instance_id = manager
            .open_view(ViewDescriptorId::new(descriptor_id), None)
            .expect("view should open");
        let mut host = self.host.borrow_mut();
        host.refresh_ui();
        host.recompute_if_dirty();
        instance_id
    }

    fn dispatch_menu_action(&self, action: &str) {
        let mut host = self.host.borrow_mut();
        let effects = callback_dispatch::dispatch_menu_action(&host.runtime, action)
            .expect("menu action dispatch should succeed");
        host.apply_dispatch_effects(effects);
        host.recompute_if_dirty();
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

fn env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn lock_env() -> std::sync::MutexGuard<'static, ()> {
    env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner())
}

fn unique_temp_path(prefix: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("{prefix}_{unique}.json"))
}

#[test]
fn child_window_viewport_pointer_event_focuses_source_window_before_runtime_dispatch() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_child_window_viewport_dispatch");
    let child = harness.detach_view_to_child_window("editor.scene#1", "window:scene");
    let baseline = harness.journal_len();

    child.invoke_viewport_pointer_event(0, 1, 24.0, 32.0, 0.0);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![
            EditorEvent::Layout(LayoutCommand::FocusView {
                instance_id: ViewInstanceId::new("editor.scene#1"),
            }),
            EditorEvent::Viewport(EditorViewportEvent::LeftPressed { x: 24.0, y: 32.0 }),
        ]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:scene"))
    );
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_asset_control_focuses_source_window_before_runtime_dispatch() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_child_window_asset_dispatch");
    let child = harness.detach_view_to_child_window("editor.assets#1", "window:assets");
    let baseline = harness.journal_len();

    child.invoke_asset_control_clicked("activity".into(), "OpenAssetBrowser".into());

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![
            EditorEvent::Layout(LayoutCommand::FocusView {
                instance_id: ViewInstanceId::new("editor.assets#1"),
            }),
            EditorEvent::Asset(EditorAssetEvent::OpenAssetBrowser),
        ]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:assets"))
    );
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_inspector_control_focuses_source_window_before_runtime_dispatch() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_child_window_inspector_dispatch");
    let child = harness.detach_view_to_child_window("editor.inspector#1", "window:inspector");
    let baseline = harness.journal_len();

    child.invoke_inspector_control_changed("NameField".into(), "Draft Cube".into());

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![
            EditorEvent::Layout(LayoutCommand::FocusView {
                instance_id: ViewInstanceId::new("editor.inspector#1"),
            }),
            EditorEvent::Draft(EditorDraftEvent::SetInspectorField {
                subject_path: "entity://selected".to_string(),
                field_id: "name".to_string(),
                value: "Draft Cube".to_string(),
            }),
        ]
    );
    assert_eq!(
        harness
            .host
            .borrow()
            .runtime
            .editor_snapshot()
            .inspector
            .as_ref()
            .map(|inspector| inspector.name.as_str()),
        Some("Draft Cube")
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:inspector"))
    );
    assert_eq!(host.callback_source_window, None);
}

fn assert_child_window_focus_tracks_asset_scroll(
    event_name: &str,
    invoke: impl FnOnce(&WorkbenchShell),
) {
    let harness = ChildWindowHostHarness::new(event_name);
    let child = harness.detach_view_to_child_window("editor.assets#1", "window:assets");
    let baseline = harness.journal_len();

    invoke(&child);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(LayoutCommand::FocusView {
            instance_id: ViewInstanceId::new("editor.assets#1"),
        })]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:assets"))
    );
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_asset_tree_scroll_focuses_source_window_before_shared_scroll_dispatch() {
    let _guard = lock_env();

    assert_child_window_focus_tracks_asset_scroll(
        "zircon_slint_child_window_asset_tree_scroll",
        |child| {
            child.invoke_asset_tree_pointer_scrolled(
                "activity".into(),
                32.0,
                84.0,
                48.0,
                280.0,
                360.0,
            );
        },
    );
}

#[test]
fn child_window_asset_content_scroll_focuses_source_window_before_shared_scroll_dispatch() {
    let _guard = lock_env();

    assert_child_window_focus_tracks_asset_scroll(
        "zircon_slint_child_window_asset_content_scroll",
        |child| {
            child.invoke_asset_content_pointer_scrolled(
                "activity".into(),
                72.0,
                120.0,
                48.0,
                320.0,
                360.0,
            );
        },
    );
}

#[test]
fn child_window_asset_reference_scroll_focuses_source_window_before_shared_scroll_dispatch() {
    let _guard = lock_env();

    assert_child_window_focus_tracks_asset_scroll(
        "zircon_slint_child_window_asset_reference_scroll",
        |child| {
            child.invoke_asset_reference_pointer_scrolled(
                "activity".into(),
                "references".into(),
                72.0,
                160.0,
                48.0,
                320.0,
                240.0,
            );
        },
    );
}

#[test]
fn root_menu_pointer_click_dispatches_shared_menu_action_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_menu_dispatch");
    let baseline = harness.journal_len();

    harness.root_ui.invoke_menu_pointer_clicked(20.0, 12.0);
    harness.root_ui.invoke_menu_pointer_clicked(60.0, 126.0);

    let host = harness.host.borrow();
    assert_eq!(host.menu_pointer_state.open_menu_index, None);
    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::WorkbenchMenu(MenuAction::ResetLayout)]
    );
}

#[test]
fn root_menu_popup_scroll_and_dismiss_flow_through_shared_pointer_bridge_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_menu_popup_scroll");
    harness
        .root_ui
        .window()
        .set_size(PhysicalSize::new(1280, 220));
    {
        let mut host = harness.host.borrow_mut();
        host.sync_shell_size();
        host.refresh_ui();
    }
    for index in 0..10 {
        harness.dispatch_menu_action(&format!("SavePreset.alpha-{index:02}"));
    }

    let window_button = harness.root_ui.get_window_menu_button_frame();
    let click_x = window_button.x + window_button.width * 0.5;
    let click_y = window_button.y + window_button.height * 0.5;

    let baseline = harness.journal_len();
    harness
        .root_ui
        .invoke_menu_pointer_clicked(click_x, click_y);

    let (popup_scroll_x, popup_scroll_y, dismiss_x, dismiss_y) = {
        let host = harness.host.borrow();
        assert_eq!(host.menu_pointer_state.open_menu_index, Some(4));
        assert!(
            host.menu_pointer_layout.preset_names.len() >= 10,
            "window menu should include saved presets before scroll"
        );
        assert!(
            host.menu_pointer_layout.window_popup_height
                < 72.0 + host.menu_pointer_layout.preset_names.len() as f32 * 30.0,
            "window popup should overflow before scroll"
        );
        let button = host.menu_pointer_layout.button_frames[4];
        let popup_y = button.y + button.height + 3.0;
        (
            button.x + 18.0,
            popup_y + 18.0,
            host.menu_pointer_layout.shell_frame.width - 24.0,
            host.menu_pointer_layout.shell_frame.height - 24.0,
        )
    };

    harness
        .root_ui
        .invoke_menu_pointer_scrolled(popup_scroll_x, popup_scroll_y, 96.0);

    {
        let host = harness.host.borrow();
        assert_eq!(host.menu_pointer_state.open_menu_index, Some(4));
        assert!(host.menu_pointer_state.popup_scroll_offset > 0.0);
    }

    harness
        .root_ui
        .invoke_menu_pointer_clicked(dismiss_x, dismiss_y);

    let host = harness.host.borrow();
    assert_eq!(host.menu_pointer_state.open_menu_index, None);
    assert!(host.menu_pointer_state.popup_scroll_offset > 0.0);
    assert!(harness.delta_events_since(baseline).is_empty());
}

#[test]
fn root_viewport_toolbar_pointer_click_uses_projection_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_viewport_toolbar_projection");
    let baseline = harness.journal_len();

    harness.root_ui.invoke_viewport_toolbar_pointer_clicked(
        "editor.scene#1".into(),
        "display.cycle".into(),
        0.0,
        0.0,
        0.0,
        0.0,
        300.0,
        10.0,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Viewport(EditorViewportEvent::SetDisplayMode {
            mode: DisplayMode::WireOverlay,
        })]
    );
}

#[test]
fn child_window_document_tab_pointer_event_dispatches_focus_view_and_tracks_window_focus() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_child_window_tab_dispatch");
    let child = harness.detach_view_to_child_window("editor.assets#1", "window:assets");
    let baseline = harness.journal_len();

    child.invoke_document_tab_pointer_clicked("window:assets".into(), 0, 8.0, 120.0, 40.0, 16.0);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(LayoutCommand::FocusView {
            instance_id: ViewInstanceId::new("editor.assets#1"),
        })]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:assets"))
    );
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_document_tab_close_pointer_event_dispatches_close_view_and_keeps_window_focus() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_child_window_tab_close_dispatch");
    let asset_browser = harness.open_view("editor.asset_browser");
    let child = harness.detach_view_to_child_window(asset_browser.0.as_str(), "window:browser");
    let baseline = harness.journal_len();

    child.invoke_document_tab_close_pointer_clicked(
        "window:browser".into(),
        0,
        8.0,
        120.0,
        112.0,
        16.0,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(LayoutCommand::CloseView {
            instance_id: asset_browser,
        })]
    );

    let host = harness.host.borrow();
    assert_eq!(host.callback_source_window, None);
}

#[test]
fn child_window_header_pointer_event_dispatches_focus_view_and_tracks_window_focus() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_child_window_header_dispatch");
    let child = harness.detach_view_to_child_window("editor.scene#1", "window:scene");
    let bounds = child.get_native_window_bounds();
    let baseline = harness.journal_len();

    child.invoke_floating_window_header_pointer_clicked(
        bounds.x + bounds.width - 40.0,
        bounds.y + 20.0,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(LayoutCommand::FocusView {
            instance_id: ViewInstanceId::new("editor.scene#1"),
        })]
    );

    let host = harness.host.borrow();
    assert_eq!(
        host.last_focused_callback_window,
        Some(MainPageId::new("window:scene"))
    );
    assert_eq!(host.callback_source_window, None);
}
