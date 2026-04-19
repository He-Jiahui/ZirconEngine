use std::cell::RefCell;
use std::fs;
use std::path::PathBuf;
use std::rc::Rc;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ui::slint_host::{PaneSurfaceHostContext, WorkbenchHostContext};
use slint::{ComponentHandle, PhysicalSize};
use zircon_core::CoreRuntime;
use zircon_framework::render::{DisplayMode, ViewOrientation};
use zircon_math::UVec2;
use zircon_runtime::foundation::{
    module_descriptor as foundation_module_descriptor, FOUNDATION_MODULE_NAME,
};
use zircon_runtime::scene::DefaultLevelManager;
mod floating_window_projection;

use super::*;
use crate::{
    module, ActivityDrawerMode, ActivityDrawerSlot, EditorAssetEvent, EditorDraftEvent,
    EditorEvent, EditorEventRuntime, EditorManager, EditorSessionMode, EditorState,
    EditorViewportEvent, LayoutCommand, MainPageId, MenuAction, RecentProjectEntry,
    RecentProjectValidation, ShellFrame, ViewDescriptorId, ViewInstanceId, EDITOR_MANAGER_NAME,
};

struct ChildWindowHostHarness {
    _core: CoreRuntime,
    config_path: PathBuf,
    host: Rc<RefCell<SlintEditorHost>>,
    root_ui: UiHostWindow,
}

fn pane_surface_host(ui: &UiHostWindow) -> PaneSurfaceHostContext<'_> {
    ui.global::<PaneSurfaceHostContext>()
}

fn host_context(ui: &UiHostWindow) -> WorkbenchHostContext<'_> {
    ui.global::<WorkbenchHostContext>()
}

impl ChildWindowHostHarness {
    fn new(prefix: &str) -> Self {
        i_slint_backend_testing::init_no_event_loop();

        let config_path = unique_temp_path(prefix);
        std::env::set_var("ZIRCON_EDITOR_CONFIG_PATH", &config_path);
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
        std::env::remove_var("ZIRCON_EDITOR_CONFIG_PATH");

        let root_ui = UiHostWindow::new().expect("root workbench shell should instantiate");
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

    fn detach_view_to_child_window(&self, instance_id: &str, window_id: &str) -> UiHostWindow {
        self.detach_views_to_child_window(&[instance_id], window_id)
    }

    fn detach_views_to_child_window(&self, instance_ids: &[&str], window_id: &str) -> UiHostWindow {
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

    fn activate_workbench_page(&self) {
        let mut host = self.host.borrow_mut();
        host.runtime.set_session_mode(EditorSessionMode::Project);
        host.editor_manager
            .dismiss_welcome_page()
            .expect("welcome page should dismiss");
        host.mark_layout_dirty();
        host.refresh_ui();
        host.recompute_if_dirty();
    }

    fn activate_drawer_tab(&self, slot: ActivityDrawerSlot, instance_id: &str) {
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

    fn stage_missing_recent_project(&self, path: &str, display_name: &str) {
        self.host
            .borrow()
            .editor_manager
            .update_recent_project(path, display_name)
            .expect("recent project should be staged");
        let mut host = self.host.borrow_mut();
        host.startup_session.recent_projects = vec![RecentProjectEntry {
            display_name: display_name.to_string(),
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

    pane_surface_host(&child).invoke_viewport_pointer_event(0, 1, 24.0, 32.0, 0.0);

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

    pane_surface_host(&child)
        .invoke_asset_control_clicked("activity".into(), "OpenAssetBrowser".into());

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

    pane_surface_host(&child)
        .invoke_inspector_control_changed("NameField".into(), "Draft Cube".into());

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
    invoke: impl FnOnce(&UiHostWindow),
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
            pane_surface_host(child).invoke_asset_tree_pointer_scrolled(
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
            pane_surface_host(child).invoke_asset_content_pointer_scrolled(
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
            pane_surface_host(child).invoke_asset_reference_pointer_scrolled(
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

    host_context(&harness.root_ui).invoke_menu_pointer_clicked(20.0, 12.0);
    host_context(&harness.root_ui).invoke_menu_pointer_clicked(60.0, 126.0);

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

    let (click_x, click_y) = {
        let host = harness.host.borrow();
        let window_button = host.menu_pointer_layout.button_frames[4];
        (
            window_button.x + window_button.width * 0.5,
            window_button.y + window_button.height * 0.5,
        )
    };

    let baseline = harness.journal_len();
    host_context(&harness.root_ui).invoke_menu_pointer_clicked(click_x, click_y);

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

    host_context(&harness.root_ui).invoke_menu_pointer_scrolled(
        popup_scroll_x,
        popup_scroll_y,
        96.0,
    );

    {
        let host = harness.host.borrow();
        assert_eq!(host.menu_pointer_state.open_menu_index, Some(4));
        assert!(host.menu_pointer_state.popup_scroll_offset > 0.0);
    }

    host_context(&harness.root_ui).invoke_menu_pointer_clicked(dismiss_x, dismiss_y);

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

    pane_surface_host(&harness.root_ui).invoke_viewport_toolbar_pointer_clicked(
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
fn root_viewport_toolbar_pointer_click_prefers_shared_projection_surface_width_over_stale_document_geometry(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_root_viewport_toolbar_projection_width");
    let (point_x, point_y) = {
        let mut host = harness.host.borrow_mut();
        let geometry = host
            .shell_geometry
            .as_mut()
            .expect("root host should have computed shell geometry");
        geometry
            .region_frames
            .insert(ShellRegionId::Left, ShellFrame::default());
        geometry
            .region_frames
            .insert(ShellRegionId::Right, ShellFrame::default());
        geometry
            .region_frames
            .insert(ShellRegionId::Bottom, ShellFrame::default());
        let document = geometry.region_frame(ShellRegionId::Document);
        geometry.region_frames.insert(
            ShellRegionId::Document,
            ShellFrame::new(document.x, document.y, 800.0, document.height),
        );

        let surface_size = host.viewport_toolbar_surface_size("editor.scene#1");
        assert!(
            surface_size.width > 1000.0,
            "shared projection width should outrank stale document geometry"
        );
        host.viewport_toolbar_bridge
            .recompute_layout(surface_size)
            .expect("viewport toolbar projection should recompute");
        let control_frame = host
            .viewport_toolbar_bridge
            .control_frame_for_action("align.neg_z")
            .expect("align.neg_z should map to a projected control frame");
        (
            control_frame.x + control_frame.width * 0.75,
            control_frame.y + control_frame.height * 0.5,
        )
    };
    let baseline = harness.journal_len();

    pane_surface_host(&harness.root_ui).invoke_viewport_toolbar_pointer_clicked(
        "editor.scene#1".into(),
        "align.neg_z".into(),
        0.0,
        0.0,
        0.0,
        0.0,
        point_x,
        point_y,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Viewport(EditorViewportEvent::AlignView {
            orientation: ViewOrientation::NegZ,
        })]
    );
}

#[test]
fn root_viewport_toolbar_surface_size_prefers_shared_projection_width_when_document_geometry_is_oversized(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_root_viewport_toolbar_projection_oversized");
    let mut host = harness.host.borrow_mut();
    let expected_width = host
        .template_bridge
        .control_frame("PaneSurfaceRoot")
        .expect("pane surface root should map to a projected control frame")
        .width;
    let geometry = host
        .shell_geometry
        .as_mut()
        .expect("root host should have computed shell geometry");
    geometry
        .region_frames
        .insert(ShellRegionId::Left, ShellFrame::default());
    geometry
        .region_frames
        .insert(ShellRegionId::Right, ShellFrame::default());
    geometry
        .region_frames
        .insert(ShellRegionId::Bottom, ShellFrame::default());
    let document = geometry.region_frame(ShellRegionId::Document);
    geometry.region_frames.insert(
        ShellRegionId::Document,
        ShellFrame::new(
            document.x,
            document.y,
            expected_width + 480.0,
            document.height,
        ),
    );

    assert_eq!(
        host.viewport_toolbar_surface_size("editor.scene#1"),
        UiSize::new(expected_width, 28.0),
        "shared projection width should remain authoritative even when legacy document geometry is wider"
    );
}

#[test]
fn root_document_tab_pointer_click_prefers_shared_projection_surface_width_over_stale_document_geometry(
) {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_document_tab_projection_width");
    harness.activate_workbench_page();
    let expected_instance = ViewInstanceId::new("editor.scene#1");
    let (tab_index, tab_x, point_x, point_y, tab_width) = {
        let mut host = harness.host.borrow_mut();
        let chrome = host.runtime.chrome_snapshot();
        let model = WorkbenchViewModel::build(&chrome);
        let tab_index = model
            .document_tabs
            .iter()
            .position(|tab| tab.instance_id == expected_instance)
            .expect("scene view should exist in document tabs");
        let shared_tabs_frame = host
            .template_bridge
            .control_frame("DocumentTabsRoot")
            .expect("document tabs root should map to a projected control frame");
        assert!(
            shared_tabs_frame.width > 1000.0,
            "shared projection width should outrank stale document geometry"
        );
        {
            let geometry = host
                .shell_geometry
                .as_mut()
                .expect("root host should have computed shell geometry");
            geometry
                .region_frames
                .insert(ShellRegionId::Left, ShellFrame::default());
            geometry
                .region_frames
                .insert(ShellRegionId::Right, ShellFrame::default());
            geometry
                .region_frames
                .insert(ShellRegionId::Bottom, ShellFrame::default());
            let document = geometry.region_frame(ShellRegionId::Document);
            geometry.region_frames.insert(
                ShellRegionId::Document,
                ShellFrame::new(document.x, document.y, 800.0, document.height),
            );
            let geometry = geometry.clone();
            let floating_window_projection_bundle =
                crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle::default();
            host.sync_document_tab_pointer_layout(
                &model,
                &geometry,
                &floating_window_projection_bundle,
            );
        }

        let tab_width = 140.0;
        let tab_x = shared_tabs_frame.width - tab_width - 24.0;
        (tab_index as i32, tab_x, tab_x + 32.0, 14.0, tab_width)
    };
    let baseline = harness.journal_len();

    host_context(&harness.root_ui).invoke_document_tab_pointer_clicked(
        "main".into(),
        tab_index,
        tab_x,
        tab_width,
        point_x,
        point_y,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(LayoutCommand::FocusView {
            instance_id: expected_instance,
        })]
    );
}

#[test]
fn root_host_page_pointer_click_prefers_shared_projection_shell_width_over_metric_strip_estimate() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_host_page_projection_width");
    let (tab_x, point_x, point_y, tab_width) = {
        let mut host = harness.host.borrow_mut();
        let chrome = host.runtime.chrome_snapshot();
        let model = WorkbenchViewModel::build(&chrome);
        let shared_shell_frame = host
            .template_bridge
            .control_frame("UiHostWindowRoot")
            .expect("workbench shell root should map to a projected control frame");
        assert!(
            shared_shell_frame.width > 1000.0,
            "shared shell projection width should outrank host-page metric estimates"
        );
        host.sync_host_page_pointer_layout(&model);

        let tab_width = 220.0;
        let tab_x = shared_shell_frame.width - tab_width - 24.0;
        (tab_x, tab_x + tab_width * 0.5, 12.0, tab_width)
    };
    let baseline = harness.journal_len();

    host_context(&harness.root_ui)
        .invoke_host_page_pointer_clicked(0, tab_x, tab_width, point_x, point_y);

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(LayoutCommand::ActivateMainPage {
            page_id: MainPageId::workbench(),
        })]
    );
}

#[test]
fn root_activity_rail_pointer_click_prefers_shared_projection_surface_when_left_region_geometry_is_stale(
) {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_activity_rail_projection_width");
    harness.activate_workbench_page();
    let (point_x, point_y) = {
        let mut host = harness.host.borrow_mut();
        let chrome = host.runtime.chrome_snapshot();
        let model = WorkbenchViewModel::build(&chrome);
        let shared_activity_rail = host
            .template_bridge
            .control_frame("ActivityRailRoot")
            .expect("activity rail root should map to a projected control frame");
        assert!(
            shared_activity_rail.width > 0.0,
            "shared projection activity rail should exist"
        );
        {
            let geometry = host
                .shell_geometry
                .as_mut()
                .expect("root host should have computed shell geometry");
            geometry
                .region_frames
                .insert(ShellRegionId::Left, ShellFrame::default());
            geometry
                .region_frames
                .insert(ShellRegionId::Right, ShellFrame::default());
            geometry
                .region_frames
                .insert(ShellRegionId::Bottom, ShellFrame::default());
            let geometry = geometry.clone();
            host.sync_activity_rail_pointer_layout(&model, &geometry);
        }

        (shared_activity_rail.width * 0.5, 20.0)
    };
    let baseline = harness.journal_len();

    host_context(&harness.root_ui).invoke_activity_rail_pointer_clicked(
        "left".into(),
        point_x,
        point_y,
    );

    assert_eq!(
        harness.delta_events_since(baseline),
        vec![EditorEvent::Layout(LayoutCommand::SetDrawerMode {
            slot: ActivityDrawerSlot::LeftTop,
            mode: ActivityDrawerMode::Collapsed,
        })]
    );
}

#[test]
fn root_resize_capture_prefers_shared_left_drawer_shell_extent_over_stale_region_geometry() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_resize_projection_extent");
    harness.activate_workbench_page();

    let mut host = harness.host.borrow_mut();
    let expected_width = host
        .template_bridge
        .control_frame("LeftDrawerShellRoot")
        .expect("left drawer shell root should map to a projected control frame")
        .width;
    let splitter = host
        .shell_geometry
        .as_ref()
        .expect("root host should have computed shell geometry")
        .splitter_frame(ShellRegionId::Left);
    let geometry = host
        .shell_geometry
        .as_mut()
        .expect("root host should have computed shell geometry");
    let left = geometry.region_frame(ShellRegionId::Left);
    geometry.region_frames.insert(
        ShellRegionId::Left,
        ShellFrame::new(left.x, left.y, 80.0, left.height),
    );

    host.workbench_resize_pointer_event(
        0,
        splitter.x + splitter.width * 0.5,
        splitter.y + splitter.height * 0.5,
    );

    assert_eq!(
        host.active_drawer_resize
            .as_ref()
            .map(|active| active.base_preferred),
        Some(expected_width),
        "resize capture should start from the shared drawer shell extent instead of stale legacy geometry"
    );
}

#[test]
fn root_host_viewport_size_matches_presented_viewport_content_frame_when_drawers_are_collapsed() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_viewport_size_alignment");
    let viewport_frame = harness.root_ui.get_host_layout().viewport_content_frame;
    let host = harness.host.borrow();

    assert_eq!(
        host.viewport_size,
        UVec2::new(
            viewport_frame.width.max(0.0).round() as u32,
            viewport_frame.height.max(0.0).round() as u32,
        )
    );
}

#[test]
fn root_host_recomputes_builtin_template_bridge_with_visible_drawer_shell_and_header_frames() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_visible_drawer_template_frames");
    harness.activate_workbench_page();

    let host = harness.host.borrow();
    let body_frame = host
        .template_bridge
        .control_frame("WorkbenchBody")
        .expect("workbench body control frame should exist");
    let expected_center_height = body_frame.height - 164.0 - 1.0;
    assert_eq!(
        host.template_bridge.control_frame("LeftDrawerShellRoot"),
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y,
            312.0,
            expected_center_height
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("LeftDrawerHeaderRoot"),
        Some(UiFrame::new(body_frame.x + 35.0, body_frame.y, 277.0, 25.0,))
    );
    assert_eq!(
        host.template_bridge.control_frame("LeftDrawerContentRoot"),
        Some(UiFrame::new(
            body_frame.x + 35.0,
            body_frame.y + 26.0,
            277.0,
            expected_center_height - 26.0,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("RightDrawerShellRoot"),
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y,
            308.0,
            expected_center_height,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("RightDrawerHeaderRoot"),
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y,
            273.0,
            25.0,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("RightDrawerContentRoot"),
        Some(UiFrame::new(
            body_frame.x + body_frame.width - 308.0,
            body_frame.y + 26.0,
            273.0,
            expected_center_height - 26.0,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("BottomDrawerShellRoot"),
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y + body_frame.height - 164.0,
            body_frame.width,
            164.0,
        ))
    );
    assert_eq!(
        host.template_bridge.control_frame("BottomDrawerHeaderRoot"),
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y + body_frame.height - 164.0,
            body_frame.width,
            25.0,
        ))
    );
    assert_eq!(
        host.template_bridge
            .control_frame("BottomDrawerContentRoot"),
        Some(UiFrame::new(
            body_frame.x,
            body_frame.y + body_frame.height - 138.0,
            body_frame.width,
            138.0,
        ))
    );
}

#[test]
fn root_welcome_recent_pointer_click_uses_projection_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_welcome_recent_projection");
    harness.stage_missing_recent_project("E:/Missing/RecentProject", "RecentProject");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_welcome_recent_pointer_clicked(160.0, 204.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.welcome_recent_pointer_size.width > 0.0);
    assert!(host.welcome_recent_pointer_size.height > 0.0);
    assert!(host
        .runtime
        .chrome_snapshot()
        .welcome
        .recent_projects
        .is_empty());
    assert_eq!(
        host.runtime.editor_snapshot().status_line,
        "Removed recent project E:/Missing/RecentProject"
    );
}

#[test]
fn root_welcome_recent_pointer_move_prefers_cached_size_over_projection_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_welcome_recent_cached_move");
    harness.stage_missing_recent_project("E:/Missing/RecentProject", "RecentProject");
    {
        let mut host = harness.host.borrow_mut();
        host.welcome_recent_pointer_size = UiSize::new(321.0, 222.0);
    }

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_welcome_recent_pointer_moved(160.0, 204.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(host.welcome_recent_pointer_size, UiSize::new(321.0, 222.0));
}

#[test]
fn root_welcome_recent_pointer_scroll_prefers_cached_size_over_projection_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_welcome_recent_cached_scroll");
    harness.stage_missing_recent_project("E:/Missing/RecentProject", "RecentProject");
    {
        let mut host = harness.host.borrow_mut();
        host.welcome_recent_pointer_size = UiSize::new(321.0, 222.0);
    }

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_welcome_recent_pointer_scrolled(160.0, 204.0, 24.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(host.welcome_recent_pointer_size, UiSize::new(321.0, 222.0));
}

#[test]
fn root_hierarchy_pointer_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_hierarchy_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_hierarchy_pointer_moved(80.0, 40.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.hierarchy_pointer_size.width > 0.0);
    assert!(host.hierarchy_pointer_size.height > 0.0);
    assert_eq!(host.hierarchy_pointer_state.hovered_item_index, Some(1));
}

#[test]
fn root_hierarchy_pointer_move_prefers_shared_drawer_content_projection_over_stale_left_region_geometry(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_root_hierarchy_content_projection_width");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.hierarchy#1");

    let expected_size = {
        let mut host = harness.host.borrow_mut();
        let shared_content = host
            .template_bridge
            .control_frame("LeftDrawerContentRoot")
            .expect("left drawer content root should map to a projected control frame");
        assert!(
            shared_content.width > 200.0 && shared_content.height > 100.0,
            "shared left drawer content frame should be larger than the stale fallback"
        );
        let geometry = host
            .shell_geometry
            .as_mut()
            .expect("root host should have computed shell geometry");
        let left = geometry.region_frame(ShellRegionId::Left);
        geometry.region_frames.insert(
            ShellRegionId::Left,
            ShellFrame::new(left.x, left.y, 120.0, 80.0),
        );
        UiSize::new(shared_content.width, shared_content.height)
    };

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_hierarchy_pointer_moved(80.0, 40.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(
        host.hierarchy_pointer_size, expected_size,
        "shared drawer content projection should own hierarchy pointer sizing when root callback width/height are missing"
    );
}

#[test]
fn root_console_pointer_scroll_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_console_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::BottomLeft, "editor.console#1");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_console_pointer_scrolled(24.0, 24.0, 48.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.console_scroll_surface.size().width > 0.0);
    assert!(host.console_scroll_surface.size().height > 0.0);
}

#[test]
fn root_console_pointer_scroll_prefers_shared_drawer_content_projection_over_stale_bottom_region_geometry(
) {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_root_console_content_projection_height");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::BottomLeft, "editor.console#1");

    let expected_size = {
        let mut host = harness.host.borrow_mut();
        let shared_content = host
            .template_bridge
            .control_frame("BottomDrawerContentRoot")
            .expect("bottom drawer content root should map to a projected control frame");
        assert!(
            shared_content.width > 400.0 && shared_content.height > 60.0,
            "shared bottom drawer content frame should be larger than the stale fallback"
        );
        let geometry = host
            .shell_geometry
            .as_mut()
            .expect("root host should have computed shell geometry");
        let bottom = geometry.region_frame(ShellRegionId::Bottom);
        geometry.region_frames.insert(
            ShellRegionId::Bottom,
            ShellFrame::new(bottom.x, bottom.y, 260.0, 44.0),
        );
        UiSize::new(shared_content.width, shared_content.height)
    };

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_console_pointer_scrolled(24.0, 24.0, 48.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert_eq!(
        host.console_scroll_surface.size(),
        expected_size,
        "shared drawer content projection should own console scroll sizing when root callback width/height are missing"
    );
}

#[test]
fn root_inspector_pointer_scroll_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_inspector_projection");
    harness.activate_workbench_page();

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_inspector_pointer_scrolled(24.0, 24.0, 48.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.inspector_scroll_surface.size().width > 0.0);
    assert!(host.inspector_scroll_surface.size().height > 0.0);
}

#[test]
fn root_asset_browser_details_scroll_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_asset_browser_projection");
    let _asset_browser = harness.open_view("editor.asset_browser");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_browser_asset_details_pointer_scrolled(24.0, 24.0, 48.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.browser_asset_details_scroll_surface.size().width > 0.0);
    assert!(host.browser_asset_details_scroll_surface.size().height > 0.0);
}

#[test]
fn root_activity_asset_tree_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_activity_asset_tree_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.assets#1");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_asset_tree_pointer_moved("activity".into(), 48.0, 72.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.activity_asset_pointer.tree_size.width > 0.0);
    assert!(host.activity_asset_pointer.tree_size.height > 0.0);
}

#[test]
fn root_browser_asset_tree_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_browser_asset_tree_projection");
    let _asset_browser = harness.open_view("editor.asset_browser");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_asset_tree_pointer_moved("browser".into(), 48.0, 72.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.browser_asset_pointer.tree_size.width > 0.0);
    assert!(host.browser_asset_pointer.tree_size.height > 0.0);
}

#[test]
fn root_activity_asset_content_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_root_activity_asset_content_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.assets#1");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_asset_content_pointer_moved("activity".into(), 96.0, 96.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.activity_asset_pointer.content_size.width > 0.0);
    assert!(host.activity_asset_pointer.content_size.height > 0.0);
}

#[test]
fn root_browser_asset_content_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_root_browser_asset_content_projection");
    let _asset_browser = harness.open_view("editor.asset_browser");

    harness
        .root_ui
        .global::<PaneSurfaceHostContext>()
        .invoke_asset_content_pointer_moved("browser".into(), 96.0, 96.0, 0.0, 0.0);

    let host = harness.host.borrow();
    assert!(host.browser_asset_pointer.content_size.width > 0.0);
    assert!(host.browser_asset_pointer.content_size.height > 0.0);
}

#[test]
fn root_activity_asset_reference_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_root_activity_asset_reference_projection");
    harness.activate_workbench_page();
    harness.activate_drawer_tab(ActivityDrawerSlot::LeftTop, "editor.assets#1");

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_moved(
        "activity".into(),
        "references".into(),
        96.0,
        160.0,
        0.0,
        0.0,
    );

    let host = harness.host.borrow();
    assert!(host.activity_asset_pointer.references.size.width > 0.0);
    assert!(host.activity_asset_pointer.references.size.height > 0.0);
}

#[test]
fn root_browser_asset_reference_move_uses_region_frame_fallback_in_real_host() {
    let _guard = lock_env();

    let harness =
        ChildWindowHostHarness::new("zircon_slint_root_browser_asset_reference_projection");
    let _asset_browser = harness.open_view("editor.asset_browser");

    pane_surface_host(&harness.root_ui).invoke_asset_reference_pointer_moved(
        "browser".into(),
        "references".into(),
        96.0,
        160.0,
        0.0,
        0.0,
    );

    let host = harness.host.borrow();
    assert!(host.browser_asset_pointer.references.size.width > 0.0);
    assert!(host.browser_asset_pointer.references.size.height > 0.0);
}

#[test]
fn child_window_document_tab_pointer_event_dispatches_focus_view_and_tracks_window_focus() {
    let _guard = lock_env();

    let harness = ChildWindowHostHarness::new("zircon_slint_child_window_tab_dispatch");
    let child = harness.detach_view_to_child_window("editor.assets#1", "window:assets");
    let baseline = harness.journal_len();

    host_context(&child).invoke_document_tab_pointer_clicked(
        "window:assets".into(),
        0,
        8.0,
        120.0,
        40.0,
        16.0,
    );

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

    host_context(&child).invoke_document_tab_close_pointer_clicked(
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
    let bounds = child.get_host_shell().native_window_bounds;
    let baseline = harness.journal_len();

    host_context(&child).invoke_floating_window_header_pointer_clicked(
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
