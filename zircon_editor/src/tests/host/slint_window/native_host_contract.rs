use std::cell::RefCell;
use std::rc::Rc;

use crate::ui::slint_host::{
    build_pane_template_surface_frame, callback_dispatch::BuiltinViewportToolbarTemplateBridge,
    to_host_contract_component_showcase_pane_from_host_pane_with_runtime, FrameRect,
    HostChromeControlFrameData, HostChromeTabData, HostDocumentDockSurfaceData, HostMenuChromeData,
    HostMenuChromeItemData, HostMenuChromeMenuData, HostMenuStateData, HostResizeLayerData,
    HostSideDockSurfaceData, HostWindowLayoutData, PaneData, PaneSurfaceHostContext, SceneNodeData,
    SceneViewportChromeData, TabData, TemplateNodeFrameData, TemplatePaneNodeData, UiHostContext,
    UiHostWindow,
};
use crate::ui::template_runtime::EditorUiHostRuntime;
use slint::{Model, ModelRc, PhysicalSize, VecModel};
use zircon_runtime_interface::ui::layout::UiSize;

#[test]
fn native_host_pointer_click_routes_document_tab_with_document_region_origin() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        tab_frames: model_rc(vec![chrome_tab(
            "document.scene",
            "Scene",
            12.0,
            4.0,
            84.0,
            24.0,
        )]),
        tabs: model_rc(vec![tab_data("document.scene", "Scene")]),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>()
            .on_document_tab_pointer_clicked(move |surface_key, index, tab_x, tab_width, x, y| {
                clicks
                    .borrow_mut()
                    .push((surface_key.to_string(), index, tab_x, tab_width, x, y));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(60.0 + 24.0, 58.0 + 12.0);

    assert!(result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [("document".to_string(), 0, 12.0, 84.0, 24.0, 12.0)],
        "root document tabs should be hit-tested in document-region global coordinates"
    );
}

#[test]
fn native_host_pointer_click_routes_host_page_tabs_with_tab_local_point() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.page_chrome.tab_frames = model_rc(vec![chrome_tab(
        "HostPageTab0",
        "Workbench",
        68.0,
        29.0,
        116.0,
        28.0,
    )]);
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<UiHostContext>().on_host_page_pointer_clicked(
            move |index, tab_x, tab_width, x, y| {
                clicks.borrow_mut().push((index, tab_x, tab_width, x, y));
            },
        );
    }

    let result = ui.dispatch_native_primary_press_for_test(80.0, 41.0);

    assert!(result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [(0, 68.0, 116.0, 12.0, 12.0)],
        "host page tab pointer bridge expects tab-local click coordinates, not global shell coordinates"
    );
}

#[test]
fn native_host_pointer_click_routes_pane_template_button_actions() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_node_with_action(
                "ProjectPrimaryAction",
                "Button",
                "Create",
                "project.create",
                12.0,
                14.0,
                92.0,
                26.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_clicked(move |control_id, action_id| {
                clicks
                    .borrow_mut()
                    .push((control_id.to_string(), action_id.to_string()));
            });
    }

    let result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);

    assert!(result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [("ProjectPrimaryAction".to_string(), "project.create".to_string())],
        "native host clicks should use template-node dispatch metadata before pane fallback routing"
    );
}

#[test]
fn native_host_pointer_click_routes_binding_only_template_buttons() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_node_with_binding(
                "ApplyDraft",
                "Button",
                "Apply Draft",
                "InspectorPaneBody/ApplyDraft",
                12.0,
                14.0,
                92.0,
                22.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_clicked(move |control_id, action_id| {
                clicks
                    .borrow_mut()
                    .push((control_id.to_string(), action_id.to_string()));
            });
    }

    let result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);

    assert!(result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [(
            "ApplyDraft".to_string(),
            "InspectorPaneBody/ApplyDraft".to_string()
        )],
        "native template button hit-testing should route projected binding metadata when no literal action_id exists"
    );
}

#[test]
fn native_host_welcome_material_text_field_accepts_keyboard_input() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(40.0, 58.0, 340.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 340.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 340.0, 145.0),
        pane: welcome_pane_with_nodes(vec![welcome_text_node(
            "WelcomeProjectNameField",
            "ProjectNameEdited",
            "Zircon",
            20.0,
            18.0,
            240.0,
            32.0,
        )]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let changes = Rc::new(RefCell::new(Vec::new()));
    {
        let changes = changes.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_welcome_control_changed(move |control_id, value| {
                changes
                    .borrow_mut()
                    .push((control_id.to_string(), value.to_string()));
            });
    }

    let focus_result =
        ui.dispatch_native_primary_press_for_test(40.0 + 20.0 + 8.0, 58.0 + 32.0 + 18.0 + 8.0);
    let insert_result = ui.dispatch_native_text_input_for_test("X");
    let backspace_result = ui.dispatch_native_backspace_for_test();
    let input_frame = host_frame(40.0 + 20.0, 58.0 + 32.0 + 18.0, 240.0, 32.0);

    assert!(focus_result.request_redraw());
    assert!(insert_result.request_redraw());
    assert!(backspace_result.request_redraw());
    assert!(!insert_result.requires_frame_update());
    assert_eq!(insert_result.damage_region(), Some(input_frame.clone()));
    assert!(!backspace_result.requires_frame_update());
    assert_eq!(backspace_result.damage_region(), Some(input_frame));
    assert_eq!(
        changes.borrow().as_slice(),
        [
            ("ProjectNameEdited".to_string(), "ZirconX".to_string()),
            ("ProjectNameEdited".to_string(), "Zircon".to_string())
        ],
        "focused Material text fields should forward keyboard edits through the welcome .ui.toml binding route"
    );
}

#[test]
fn native_host_generic_template_text_field_routes_builtin_change_binding() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_input_node(
                "NameField",
                "Draft Cube",
                "InspectorView/NameField",
                12.0,
                14.0,
                160.0,
                24.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);
    let before_snapshot = ui
        .window()
        .take_snapshot()
        .expect("pre-edit text field snapshot should render");

    let edits = Rc::new(RefCell::new(Vec::new()));
    {
        let edits = edits.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_edited(move |control_id, binding_id, value| {
                edits.borrow_mut().push((
                    control_id.to_string(),
                    binding_id.to_string(),
                    value.to_string(),
                ));
            });
    }

    let focus_result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    let insert_result = ui.dispatch_native_text_input_for_test("X");
    let after_snapshot = ui
        .window()
        .take_snapshot()
        .expect("focused text field snapshot should render the edited value");
    let input_frame = host_frame(60.0 + 12.0, 58.0 + 32.0 + 14.0, 160.0, 24.0);

    assert!(focus_result.request_redraw());
    assert!(insert_result.request_redraw());
    assert!(!focus_result.requires_frame_update());
    assert_eq!(focus_result.damage_region(), Some(input_frame.clone()));
    assert!(!insert_result.requires_frame_update());
    assert_eq!(insert_result.damage_region(), Some(input_frame.clone()));
    assert_eq!(
        ui.get_host_presentation()
            .text_input_focus
            .value_text
            .as_str(),
        "Draft CubeX"
    );
    assert!(
        changed_pixel_count(
            after_snapshot.width(),
            before_snapshot.as_bytes(),
            after_snapshot.as_bytes(),
            input_frame.x as u32,
            input_frame.y as u32,
            input_frame.width as u32,
            input_frame.height as u32,
        ) > 0,
        "local text edit focus should repaint visible input glyphs without waiting for a full presentation rebuild"
    );
    assert_eq!(
        edits.borrow().as_slice(),
        [(
            "NameField".to_string(),
            "InspectorView/NameField".to_string(),
            "Draft CubeX".to_string()
        )],
        "generic template text fields should keep the shared edit binding id and changed value on the native route"
    );
}

#[test]
fn native_host_generic_template_text_field_routes_commit_binding_on_enter() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_input_node_with_commit(
                "NameField",
                "Draft Cube",
                "InspectorView/NameField",
                "InspectorView/ApplyBatchButton",
                12.0,
                14.0,
                160.0,
                24.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let edits = Rc::new(RefCell::new(Vec::new()));
    {
        let edits = edits.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_edited(move |control_id, binding_id, value| {
                edits.borrow_mut().push((
                    control_id.to_string(),
                    binding_id.to_string(),
                    value.to_string(),
                ));
            });
    }

    ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    ui.dispatch_native_text_input_for_test("X");
    let commit_result = ui.dispatch_native_enter_for_test();

    assert!(commit_result.request_redraw());
    assert_eq!(
        edits.borrow().as_slice(),
        [
            (
                "NameField".to_string(),
                "InspectorView/NameField".to_string(),
                "Draft CubeX".to_string()
            ),
            (
                "NameField".to_string(),
                "InspectorView/ApplyBatchButton".to_string(),
                "Draft CubeX".to_string()
            )
        ],
        "Enter should dispatch the focused text field commit binding instead of reusing the edit binding"
    );
}

#[test]
fn native_host_binding_only_template_text_field_accepts_keyboard_input() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_input_node_with_binding(
                "NameField",
                "Draft Cube",
                "InspectorView/NameField",
                12.0,
                14.0,
                160.0,
                24.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let edits = Rc::new(RefCell::new(Vec::new()));
    {
        let edits = edits.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_edited(move |control_id, binding_id, value| {
                edits.borrow_mut().push((
                    control_id.to_string(),
                    binding_id.to_string(),
                    value.to_string(),
                ));
            });
    }

    let focus_result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);
    let insert_result = ui.dispatch_native_text_input_for_test("X");
    let input_frame = host_frame(60.0 + 12.0, 58.0 + 32.0 + 14.0, 160.0, 24.0);

    assert!(focus_result.request_redraw());
    assert!(insert_result.request_redraw());
    assert!(!insert_result.requires_frame_update());
    assert_eq!(insert_result.damage_region(), Some(input_frame));
    assert_eq!(
        edits.borrow().as_slice(),
        [(
            "NameField".to_string(),
            "InspectorView/NameField".to_string(),
            "Draft CubeX".to_string()
        )],
        "binding-only input nodes still need a single native text edit target"
    );
}

#[test]
fn native_host_resize_splitter_forwards_move_and_release_after_capture() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.resize_layer = HostResizeLayerData {
        left_splitter_frame: host_frame(120.0, 58.0, 8.0, 138.0),
        ..HostResizeLayerData::default()
    };
    ui.set_host_presentation(presentation);

    let resize_events = Rc::new(RefCell::new(Vec::new()));
    {
        let resize_events = resize_events.clone();
        ui.global::<UiHostContext>()
            .on_host_resize_pointer_event(move |kind, x, y| {
                resize_events.borrow_mut().push((kind, x, y));
            });
    }

    let press = ui.dispatch_native_primary_press_for_test(124.0, 80.0);
    let move_result = ui.dispatch_native_pointer_move_for_test(180.0, 82.0);
    let release = ui.dispatch_native_primary_release_for_test(180.0, 82.0);

    assert!(press.request_redraw());
    assert!(move_result.request_redraw());
    assert!(release.request_redraw());
    assert_eq!(
        resize_events.borrow().as_slice(),
        [(0, 124.0, 80.0), (1, 180.0, 82.0), (2, 180.0, 82.0)],
        "native resize should stay captured until pointer up"
    );
}

#[test]
fn native_host_welcome_material_button_routes_welcome_callback() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(40.0, 58.0, 340.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 340.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 340.0, 145.0),
        pane: welcome_pane_with_nodes(vec![welcome_button_node(
            "WelcomeCreateProjectButton",
            "CreateProject",
            "Create Project",
            20.0,
            64.0,
            132.0,
            30.0,
        )]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_welcome_control_clicked(move |control_id| {
                clicks.borrow_mut().push(control_id.to_string());
            });
    }

    let result =
        ui.dispatch_native_primary_press_for_test(40.0 + 20.0 + 8.0, 58.0 + 32.0 + 64.0 + 8.0);

    assert!(result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        ["CreateProject".to_string()],
        "welcome Material buttons should route to the welcome surface command bridge"
    );
}

#[test]
fn native_host_document_tab_drag_releases_capture_and_forwards_drop() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(420, 260));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(420.0, 260.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(40.0, 58.0, 340.0, 178.0),
        header_frame: host_frame(0.0, 0.0, 340.0, 31.0),
        tab_frames: model_rc(vec![chrome_tab(
            "document.scene",
            "Scene",
            12.0,
            4.0,
            84.0,
            24.0,
        )]),
        tabs: model_rc(vec![tab_data("document.scene", "Scene")]),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let drag_events = Rc::new(RefCell::new(Vec::new()));
    {
        let drag_events = drag_events.clone();
        ui.global::<UiHostContext>()
            .on_host_drag_pointer_event(move |kind, x, y| {
                drag_events.borrow_mut().push((kind, x, y));
            });
    }

    ui.dispatch_native_primary_press_for_test(40.0 + 24.0, 58.0 + 12.0);
    ui.dispatch_native_pointer_move_for_test(40.0 + 132.0, 58.0 + 74.0);
    ui.dispatch_native_primary_release_for_test(40.0 + 132.0, 58.0 + 74.0);

    let drag_state = ui.global::<UiHostContext>().get_drag_state();
    assert_eq!(
        drag_events.borrow().as_slice(),
        [(0, 172.0, 132.0), (2, 172.0, 132.0)],
        "native document-tab drags should enter and leave the shared Slate drag path"
    );
    assert!(
        drag_state.drag_tab_id.is_empty() && !drag_state.drag_active,
        "drag capture must be cleared on primary release so native repaint cannot leave stale drag state behind"
    );
}

#[test]
fn native_host_pointer_click_routes_projected_material_showcase_button() {
    i_slint_backend_testing::init_no_event_loop();
    let _guard = crate::tests::support::env_lock()
        .lock()
        .unwrap_or_else(|poison| poison.into_inner());

    let mut runtime = EditorUiHostRuntime::default();
    runtime
        .load_builtin_host_templates()
        .expect("builtin host templates should load for showcase native host test");
    let showcase_pane = component_showcase_pane_with_runtime_projection(&runtime, 900.0, 760.0);
    let button = template_node_by_control_id(&showcase_pane, "ButtonDemo");
    assert_eq!(button.dispatch_kind.as_str(), "showcase");
    assert_eq!(
        button.action_id.as_str(),
        "UiComponentShowcase/ButtonCommit"
    );
    assert!(
        button.frame.width > 0.0 && button.frame.height > 0.0,
        "projected Material ButtonDemo should carry a concrete arranged frame"
    );

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(980, 840));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(980.0, 840.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(980.0, 840.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(40.0, 52.0, 920.0, 760.0),
        header_frame: host_frame(0.0, 0.0, 920.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 900.0, 720.0),
        pane: showcase_pane.clone(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let activations = Rc::new(RefCell::new(Vec::new()));
    {
        let activations = activations.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_component_showcase_control_activated(move |control_id, action_id| {
                activations
                    .borrow_mut()
                    .push((control_id.to_string(), action_id.to_string()));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(
        40.0 + button.frame.x + button.frame.width * 0.5,
        52.0 + 32.0 + button.frame.y + button.frame.height * 0.5,
    );

    assert!(result.request_redraw());
    assert_eq!(
        activations.borrow().as_slice(),
        [(
            "ButtonDemo".to_string(),
            "UiComponentShowcase/ButtonCommit".to_string()
        )],
        "native host should route real .ui.toml Material component hits through showcase dispatch metadata"
    );
}

#[test]
fn native_host_pointer_click_ignores_template_buttons_without_dispatch_metadata() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: pane_with_nodes(
            "Project",
            vec![template_node(
                "MissingDispatch",
                "Button",
                "Decorative",
                12.0,
                14.0,
                92.0,
                22.0,
            )],
        ),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let clicks = clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_surface_control_clicked(move |control_id, action_id| {
                clicks
                    .borrow_mut()
                    .push((control_id.to_string(), action_id.to_string()));
            });
    }

    let result =
        ui.dispatch_native_primary_press_for_test(60.0 + 12.0 + 8.0, 58.0 + 32.0 + 14.0 + 8.0);

    assert!(result.request_redraw());
    assert_eq!(
        clicks.borrow().as_slice(),
        [],
        "visual buttons without action, binding, or dispatch kind must not consume native clicks as empty actions"
    );
}

#[test]
fn native_host_pointer_click_routes_viewport_toolbar_buttons_before_viewport_body() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(720, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 620.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 620.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 620.0, 105.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    let tool_frame = viewport_toolbar_control_frame(&presentation, "tool.move");
    ui.set_host_presentation(presentation);

    let toolbar_clicks = Rc::new(RefCell::new(Vec::new()));
    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let toolbar_clicks = toolbar_clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_toolbar_pointer_clicked(
                move |surface_key,
                      control_id,
                      control_x,
                      control_y,
                      control_width,
                      control_height,
                      point_x,
                      point_y| {
                    toolbar_clicks.borrow_mut().push((
                        surface_key.to_string(),
                        control_id.to_string(),
                        control_x,
                        control_y,
                        control_width,
                        control_height,
                        point_x,
                        point_y,
                    ));
                },
            );
    }
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_pointer_event(move |kind, button, x, y, delta| {
                viewport_events
                    .borrow_mut()
                    .push((kind, button, x, y, delta));
            });
    }

    let result = ui.dispatch_native_primary_press_for_test(
        60.0 + tool_frame.x + tool_frame.width * 0.5,
        58.0 + 32.0 + tool_frame.y + tool_frame.height * 0.5,
    );

    assert!(result.request_redraw());
    assert_eq!(viewport_events.borrow().as_slice(), []);
    let clicks = toolbar_clicks.borrow();
    assert_eq!(clicks.len(), 1);
    assert_eq!(clicks[0].0, "document");
    assert_eq!(clicks[0].1, "tool.move");
    assert_eq!(clicks[0].2, tool_frame.x);
    assert_eq!(clicks[0].3, tool_frame.y);
    assert_eq!(clicks[0].4, tool_frame.width);
    assert_eq!(clicks[0].5, tool_frame.height);
    assert_eq!(clicks[0].6, tool_frame.x + tool_frame.width * 0.5);
    assert_eq!(clicks[0].7, tool_frame.y + tool_frame.height * 0.5);
}

#[test]
fn native_host_viewport_toolbar_only_dispatches_primary_press() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(720, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(720.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 620.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 620.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 620.0, 105.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    let display_frame = viewport_toolbar_control_frame(&presentation, "display.cycle");
    ui.set_host_presentation(presentation);

    let toolbar_clicks = Rc::new(RefCell::new(Vec::new()));
    {
        let toolbar_clicks = toolbar_clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_toolbar_pointer_clicked(
                move |_surface_key,
                      control_id,
                      _control_x,
                      _control_y,
                      _control_width,
                      _control_height,
                      _point_x,
                      _point_y| {
                    toolbar_clicks.borrow_mut().push(control_id.to_string());
                },
            );
    }

    let display_x = 60.0 + display_frame.x + display_frame.width * 0.5;
    let toolbar_y = 58.0 + 32.0 + display_frame.y + display_frame.height * 0.5;
    let press = ui.dispatch_native_primary_press_for_test(display_x, toolbar_y);
    let release = ui.dispatch_native_primary_release_for_test(display_x, toolbar_y);
    let secondary = ui.dispatch_native_secondary_press_for_test(display_x, toolbar_y);
    let middle = ui.dispatch_native_middle_press_for_test(display_x, toolbar_y);

    assert!(press.request_redraw());
    assert!(!release.request_redraw());
    assert!(!secondary.request_redraw());
    assert!(!middle.request_redraw());
    assert_eq!(toolbar_clicks.borrow().as_slice(), ["display.cycle"]);
}

#[test]
fn native_host_pointer_click_routes_late_viewport_toolbar_controls() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(900, 240));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(900.0, 240.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(900.0, 240.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 800.0, 158.0),
        header_frame: host_frame(0.0, 0.0, 800.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 800.0, 125.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    let frame_selection_frame = viewport_toolbar_control_frame(&presentation, "frame.selection");
    ui.set_host_presentation(presentation);

    let toolbar_clicks = Rc::new(RefCell::new(Vec::new()));
    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let toolbar_clicks = toolbar_clicks.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_toolbar_pointer_clicked(
                move |_surface_key,
                      control_id,
                      control_x,
                      _control_y,
                      control_width,
                      _control_height,
                      point_x,
                      _point_y| {
                    toolbar_clicks.borrow_mut().push((
                        control_id.to_string(),
                        control_x,
                        control_width,
                        point_x,
                    ));
                },
            );
    }
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_pointer_event(move |kind, button, x, y, delta| {
                viewport_events
                    .borrow_mut()
                    .push((kind, button, x, y, delta));
            });
    }

    let frame_selection_x = 60.0 + frame_selection_frame.x + frame_selection_frame.width * 0.5;
    let toolbar_y = 58.0 + 32.0 + frame_selection_frame.y + frame_selection_frame.height * 0.5;
    let result = ui.dispatch_native_primary_press_for_test(frame_selection_x, toolbar_y);

    assert!(result.request_redraw());
    assert_eq!(viewport_events.borrow().as_slice(), []);
    assert_eq!(
        toolbar_clicks.borrow().as_slice(),
        [(
            "frame.selection".to_string(),
            frame_selection_frame.x,
            frame_selection_frame.width,
            frame_selection_frame.x + frame_selection_frame.width * 0.5,
        )]
    );
}

#[test]
fn native_host_pointer_move_routes_viewport_without_native_repaint() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_pointer_event(move |kind, button, x, y, delta| {
                viewport_events
                    .borrow_mut()
                    .push((kind, button, x, y, delta));
            });
    }

    let result = ui.dispatch_native_pointer_move_for_test(60.0 + 40.0, 58.0 + 32.0 + 28.0 + 12.0);

    assert!(
        !result.request_redraw(),
        "viewport moves update runtime input state; native repaint waits for the next viewport image"
    );
    assert_eq!(
        viewport_events.borrow().as_slice(),
        [(1, 0, 40.0, 12.0, 0.0)],
        "viewport move facts should still reach the shared pointer bridge"
    );
}

#[test]
fn native_host_viewport_button_and_scroll_wait_for_viewport_image_repaint() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(60.0, 58.0, 280.0, 138.0),
        header_frame: host_frame(0.0, 0.0, 280.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 280.0, 105.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);
    let rebuild_count_after_projection = ui.presentation_rebuild_count_for_test();

    let viewport_events = Rc::new(RefCell::new(Vec::new()));
    {
        let viewport_events = viewport_events.clone();
        ui.global::<PaneSurfaceHostContext>()
            .on_viewport_pointer_event(move |kind, button, x, y, delta| {
                viewport_events
                    .borrow_mut()
                    .push((kind, button, x, y, delta));
            });
    }

    let x = 60.0 + 40.0;
    let y = 58.0 + 32.0 + 28.0 + 12.0;
    let press = ui.dispatch_native_primary_press_for_test(x, y);
    let release = ui.dispatch_native_primary_release_for_test(x, y);
    let scroll = ui.dispatch_native_pointer_scroll_for_test(x, y, -120.0);

    assert!(
        !press.request_redraw(),
        "viewport press updates runtime input; native repaint waits for the next viewport image"
    );
    assert!(
        !release.request_redraw(),
        "viewport release should not force a stale native repaint"
    );
    assert!(
        !scroll.request_redraw(),
        "viewport scroll should not repaint the old viewport image before the renderer updates it"
    );
    assert_eq!(
        ui.presentation_rebuild_count_for_test(),
        rebuild_count_after_projection,
        "viewport pointer events must not rebuild projected presentation state"
    );
    assert_eq!(
        viewport_events.borrow().as_slice(),
        [
            (0, 1, 40.0, 12.0, 0.0),
            (2, 1, 40.0, 12.0, 0.0),
            (3, 0, 40.0, 12.0, -120.0),
        ],
        "viewport press/release/scroll facts should still reach the shared pointer bridge"
    );
}

#[test]
fn native_host_hierarchy_move_updates_visible_hover_state() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(20.0, 40.0, 300.0, 150.0),
        header_frame: host_frame(0.0, 0.0, 300.0, 24.0),
        content_frame: host_frame(0.0, 25.0, 300.0, 124.0),
        pane: hierarchy_pane(vec![
            scene_node("entity://root", "Root", 0, false),
            scene_node("entity://child", "Child", 1, false),
        ]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let before = ui
        .window()
        .take_snapshot()
        .expect("pre-hover hierarchy snapshot should render");
    {
        let ui = ui.clone_strong();
        ui.global::<PaneSurfaceHostContext>()
            .on_hierarchy_pointer_moved(move |_x, _y, _width, _height| {
                ui.global::<PaneSurfaceHostContext>()
                    .set_hovered_hierarchy_index(1);
            });
    }

    let result = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 42.0);
    let after = ui
        .window()
        .take_snapshot()
        .expect("post-hover hierarchy snapshot should render");

    assert!(result.request_redraw());
    assert!(
        !result.requires_frame_update(),
        "native hover should repaint the pane region without forcing a full frame update"
    );
    assert_eq!(
        result.damage_region(),
        Some(host_frame(28.0, 96.0, 284.0, 22.0)),
        "hierarchy hover should damage the changed row instead of the full host frame"
    );
    assert!(
        changed_pixel_count(
            after.width(),
            before.as_bytes(),
            after.as_bytes(),
            28,
            94,
            284,
            26,
        ) > 80,
        "native hierarchy hover state should be visible in the rust-owned host painter"
    );
    let repeated = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 42.0);
    assert!(
        !repeated.request_redraw(),
        "repeating the same hierarchy hover target should be a pointer fast path"
    );
}

#[test]
fn native_host_repeated_hierarchy_hover_moves_do_not_rebuild_presentation() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(20.0, 40.0, 300.0, 150.0),
        header_frame: host_frame(0.0, 0.0, 300.0, 24.0),
        content_frame: host_frame(0.0, 25.0, 300.0, 124.0),
        pane: hierarchy_pane(vec![
            scene_node("entity://root", "Root", 0, false),
            scene_node("entity://child", "Child", 1, false),
        ]),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);
    let rebuild_count_after_projection = ui.presentation_rebuild_count_for_test();
    {
        let ui = ui.clone_strong();
        ui.global::<PaneSurfaceHostContext>()
            .on_hierarchy_pointer_moved(move |_x, _y, _width, _height| {
                ui.global::<PaneSurfaceHostContext>()
                    .set_hovered_hierarchy_index(1);
            });
    }

    let hover_x = 20.0 + 20.0;
    let hover_y = 40.0 + 25.0 + 42.0;
    let first = ui.dispatch_native_pointer_move_for_test(hover_x, hover_y);

    assert!(first.request_redraw());
    assert!(
        !first.requires_frame_update(),
        "first hierarchy hover should use local paint damage"
    );
    assert_eq!(
        ui.presentation_rebuild_count_for_test(),
        rebuild_count_after_projection,
        "pointer-only hover must not rebuild the projected presentation"
    );

    for _ in 0..100 {
        let repeated = ui.dispatch_native_pointer_move_for_test(hover_x, hover_y);
        assert!(
            !repeated.request_redraw(),
            "same-target hierarchy hover should stay on the pointer fast path"
        );
        assert!(
            !repeated.requires_frame_update(),
            "same-target hierarchy hover must not request a full frame update"
        );
    }
    assert_eq!(
        ui.presentation_rebuild_count_for_test(),
        rebuild_count_after_projection,
        "100 same-target hover moves must not rebuild presentation state"
    );
}

#[test]
fn native_host_asset_tree_move_updates_visible_hover_state() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));
    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.left_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.right_dock = HostSideDockSurfaceData::default();
    presentation.host_scene_data.bottom_dock = Default::default();
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        surface_key: "document".into(),
        region_frame: host_frame(20.0, 40.0, 300.0, 150.0),
        header_frame: host_frame(0.0, 0.0, 300.0, 24.0),
        content_frame: host_frame(0.0, 25.0, 300.0, 124.0),
        pane: asset_tree_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    ui.set_host_presentation(presentation);

    let before = ui
        .window()
        .take_snapshot()
        .expect("pre-hover asset tree snapshot should render");
    {
        let ui = ui.clone_strong();
        ui.global::<PaneSurfaceHostContext>()
            .on_asset_tree_pointer_moved(move |_mode, _x, _y, _width, _height| {
                ui.global::<PaneSurfaceHostContext>()
                    .set_activity_asset_tree_hovered_index(0);
            });
    }

    let result = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 57.0 + 12.0);
    let after = ui
        .window()
        .take_snapshot()
        .expect("post-hover asset tree snapshot should render");

    assert!(result.request_redraw());
    assert!(
        !result.requires_frame_update(),
        "native asset hover should repaint the pane region without forcing a full frame update"
    );
    assert!(
        changed_pixel_count(
            after.width(),
            before.as_bytes(),
            after.as_bytes(),
            28,
            122,
            220,
            28,
        ) > 80,
        "native asset tree hover state should be visible in the rust-owned host painter"
    );
    let repeated = ui.dispatch_native_pointer_move_for_test(20.0 + 20.0, 40.0 + 25.0 + 57.0 + 12.0);
    assert!(
        !repeated.request_redraw(),
        "repeating the same asset-tree hover target should not repaint"
    );
}

#[test]
fn rust_owned_host_painter_draws_open_menu_popup_above_pane_surfaces() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));

    let mut closed = ui.get_host_presentation();
    closed.host_layout = host_window_layout_for_test(360.0, 220.0);
    closed.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    closed.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 8.0, 2.0, 56.0, 22.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            label: "File".into(),
            popup_width_px: 144.0,
            popup_height_px: 66.0,
            items: model_rc(vec![
                HostMenuChromeItemData {
                    label: "Open".into(),
                    action_id: "OpenProject".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
                HostMenuChromeItemData {
                    label: "Reset Layout".into(),
                    action_id: "ResetLayout".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                },
            ]),
            popup_nodes: model_rc(vec![
                template_node("MenuPopupPanel", "Panel", "", 0.0, 0.0, 144.0, 66.0),
                template_node("MenuPopupItemRow0", "Panel", "Open", 6.0, 6.0, 132.0, 26.0),
                template_node(
                    "MenuPopupItemRow1",
                    "Panel",
                    "Reset",
                    6.0,
                    34.0,
                    132.0,
                    26.0,
                ),
            ]),
        }]),
        ..HostMenuChromeData::default()
    };
    closed.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(0.0, 26.0, 360.0, 170.0),
        header_frame: host_frame(0.0, 0.0, 360.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 360.0, 137.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    closed.menu_state = HostMenuStateData {
        open_menu_index: -1,
        ..HostMenuStateData::default()
    };
    ui.set_host_presentation(closed.clone());
    let closed_snapshot = ui
        .window()
        .take_snapshot()
        .expect("closed menu snapshot should render");

    let mut open = closed;
    open.menu_state = HostMenuStateData {
        open_menu_index: 0,
        ..HostMenuStateData::default()
    };
    let open_menu_state = open.menu_state.clone();
    ui.set_host_presentation(open);
    ui.global::<UiHostContext>().set_menu_state(open_menu_state);
    let open_snapshot = ui
        .window()
        .take_snapshot()
        .expect("open menu snapshot should render");

    assert!(
        changed_pixel_count(
            open_snapshot.width(),
            closed_snapshot.as_bytes(),
            open_snapshot.as_bytes(),
            8,
            27,
            144,
            66,
        ) > 200,
        "open menu popup should paint over the document/viewport surface below the menu bar"
    );
}

#[test]
fn rust_owned_host_painter_draws_open_nested_menu_popup() {
    i_slint_backend_testing::init_no_event_loop();

    let ui = UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in test backend");
    ui.window().set_size(PhysicalSize::new(360, 220));

    let mut presentation = ui.get_host_presentation();
    presentation.host_layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.layout = host_window_layout_for_test(360.0, 220.0);
    presentation.host_scene_data.menu_chrome = HostMenuChromeData {
        top_bar_height_px: 25.0,
        menu_frames: model_rc(vec![control_frame("MenuSlot0", 8.0, 2.0, 56.0, 22.0)]),
        menus: model_rc(vec![HostMenuChromeMenuData {
            label: "Tools".into(),
            popup_width_px: 144.0,
            popup_height_px: 38.0,
            items: model_rc(vec![HostMenuChromeItemData {
                label: "Weather".into(),
                shortcut: ">".into(),
                enabled: true,
                children: model_rc(vec![HostMenuChromeItemData {
                    label: "Refresh Clouds".into(),
                    action_id: "Weather.CloudLayer.Refresh".into(),
                    enabled: true,
                    ..HostMenuChromeItemData::default()
                }]),
                ..HostMenuChromeItemData::default()
            }]),
            popup_nodes: model_rc(vec![
                template_node("MenuPopupPanel", "Panel", "", 0.0, 0.0, 144.0, 38.0),
                template_node(
                    "MenuPopupItemRow0",
                    "Panel",
                    "Weather",
                    6.0,
                    6.0,
                    132.0,
                    26.0,
                ),
            ]),
        }]),
        ..HostMenuChromeData::default()
    };
    presentation.host_scene_data.document_dock = HostDocumentDockSurfaceData {
        region_frame: host_frame(0.0, 26.0, 360.0, 170.0),
        header_frame: host_frame(0.0, 0.0, 360.0, 31.0),
        content_frame: host_frame(0.0, 32.0, 360.0, 137.0),
        pane: scene_pane(),
        ..HostDocumentDockSurfaceData::default()
    };
    presentation.menu_state = HostMenuStateData {
        open_menu_index: 0,
        ..HostMenuStateData::default()
    };
    let root_menu_state = presentation.menu_state.clone();
    ui.set_host_presentation(presentation.clone());
    ui.global::<UiHostContext>().set_menu_state(root_menu_state);
    let root_only = ui
        .window()
        .take_snapshot()
        .expect("root menu snapshot should render");

    presentation.menu_state = HostMenuStateData {
        open_menu_index: 0,
        open_submenu_path: vec![0],
        hovered_menu_item_path: vec![0, 0],
        hovered_menu_item_index: 1,
        ..HostMenuStateData::default()
    };
    let nested_menu_state = presentation.menu_state.clone();
    ui.set_host_presentation(presentation);
    ui.global::<UiHostContext>()
        .set_menu_state(nested_menu_state);
    let nested = ui
        .window()
        .take_snapshot()
        .expect("nested menu snapshot should render");

    assert!(
        changed_pixel_count(
            nested.width(),
            root_only.as_bytes(),
            nested.as_bytes(),
            148,
            33,
            150,
            42,
        ) > 140,
        "opening a submenu branch should paint a visible child popup beside the root menu"
    );
}

fn host_frame(x: f32, y: f32, width: f32, height: f32) -> FrameRect {
    FrameRect {
        x,
        y,
        width,
        height,
    }
}

fn host_window_layout_for_test(width: f32, height: f32) -> HostWindowLayoutData {
    HostWindowLayoutData {
        center_band_frame: host_frame(0.0, 58.0, width, height - 82.0),
        status_bar_frame: host_frame(0.0, height - 24.0, width, 24.0),
        left_region_frame: FrameRect::default(),
        document_region_frame: host_frame(60.0, 58.0, width - 80.0, height - 82.0),
        viewport_content_frame: host_frame(60.0, 118.0, width - 80.0, height - 142.0),
        ..HostWindowLayoutData::default()
    }
}

fn scene_pane() -> PaneData {
    PaneData {
        kind: "Scene".into(),
        title: "Scene".into(),
        show_toolbar: true,
        viewport: SceneViewportChromeData {
            tool: "Move".into(),
            transform_space: "Global".into(),
            display_mode: "Lit".into(),
            grid_mode: "Grid".into(),
            toolbar_surface_frame: Some(viewport_toolbar_surface_frame_for_test()),
            ..SceneViewportChromeData::default()
        },
        ..PaneData::default()
    }
}

fn viewport_toolbar_surface_frame_for_test() -> zircon_runtime_interface::ui::surface::UiSurfaceFrame
{
    let mut bridge = BuiltinViewportToolbarTemplateBridge::new()
        .expect("viewport toolbar template bridge should load in native host tests");
    bridge
        .recompute_layout(UiSize::new(1200.0, 28.0))
        .expect("viewport toolbar template should compute test layout");
    bridge.surface_frame_for_projection_controls(
        "document",
        UiSize::new(1200.0, 28.0),
        |projection_control_id| {
            Some(viewport_toolbar_hit_control_id_for_test(
                projection_control_id,
            ))
        },
    )
}

fn viewport_toolbar_hit_control_id_for_test(projection_control_id: &str) -> String {
    match projection_control_id {
        "SetTool" => "tool.move",
        "SetTransformSpace" => "space.global",
        "SetProjectionMode" => "projection.perspective",
        "AlignView" => "align.neg_z",
        "SetDisplayMode" => "display.cycle",
        "SetGridMode" => "grid.cycle",
        "SetTranslateSnap" => "snap.translate",
        "SetRotateSnapDegrees" => "snap.rotate",
        "SetScaleSnap" => "snap.scale",
        "SetPreviewLighting" => "toggle.lighting",
        "SetPreviewSkybox" => "toggle.skybox",
        "SetGizmosEnabled" => "toggle.gizmos",
        "FrameSelection" => "frame.selection",
        "EnterPlayMode" => "EnterPlayMode",
        "ExitPlayMode" => "ExitPlayMode",
        _ => projection_control_id,
    }
    .to_string()
}

fn viewport_toolbar_control_frame(
    presentation: &crate::ui::slint_host::HostWindowPresentationData,
    control_id: &str,
) -> FrameRect {
    let toolbar_frame = presentation
        .host_scene_data
        .document_dock
        .pane
        .viewport
        .toolbar_surface_frame
        .as_ref()
        .expect("scene pane should carry a viewport toolbar surface frame");
    let arranged = toolbar_frame
        .arranged_tree
        .nodes
        .iter()
        .find(|node| node.control_id.as_deref() == Some(control_id))
        .unwrap_or_else(|| panic!("missing viewport toolbar control frame for {control_id}"));
    host_frame(
        arranged.frame.x,
        arranged.frame.y,
        arranged.frame.width,
        arranged.frame.height,
    )
}

fn pane_with_nodes(kind: &str, nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: kind.into(),
        title: kind.into(),
        ..PaneData::default()
    };
    pane.project_overview.nodes = model_rc(nodes);
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(1000.0, 1000.0));
    pane
}

fn welcome_pane_with_nodes(nodes: Vec<TemplatePaneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: "Welcome".into(),
        title: "Welcome".into(),
        ..PaneData::default()
    };
    pane.welcome.nodes = model_rc(nodes);
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(1000.0, 1000.0));
    pane
}

fn component_showcase_pane_with_runtime_projection(
    runtime: &EditorUiHostRuntime,
    width: f32,
    height: f32,
) -> PaneData {
    use crate::ui::layouts::windows::workbench_host_window as host_window;
    use crate::ui::workbench::view::{
        PaneBodySpec, PaneInteractionMode, PanePayloadKind, PaneRouteNamespace,
    };

    let fixture = crate::ui::workbench::fixture::default_preview_fixture();
    let chrome = fixture.build_chrome();
    let body_spec = PaneBodySpec::new(
        "editor.window.ui_component_showcase",
        PanePayloadKind::UiComponentShowcaseV1,
        PaneRouteNamespace::UiComponentShowcase,
        PaneInteractionMode::TemplateOnly,
    );
    let body = host_window::build_pane_body_presentation(
        &body_spec,
        &host_window::PanePayloadBuildContext::new(&chrome),
    );
    let workbench_pane = host_window::PaneData {
        id: "component-showcase".into(),
        slot: "component-showcase-slot".into(),
        kind: "UiComponentShowcase".into(),
        title: "UI Component Showcase".into(),
        icon_key: "ui-components".into(),
        subtitle: "Runtime components".into(),
        info: "".into(),
        show_empty: false,
        empty_title: "".into(),
        empty_body: "".into(),
        primary_action_label: "".into(),
        primary_action_id: "".into(),
        secondary_action_label: "".into(),
        secondary_action_id: "".into(),
        secondary_hint: "".into(),
        show_toolbar: false,
        viewport: crate::ui::layouts::views::blank_viewport_chrome(),
        native_body: host_window::PaneNativeBodyData {
            hierarchy: host_window::HierarchyPaneViewData::default(),
            inspector: host_window::InspectorPaneViewData::default(),
            console: host_window::ConsolePaneViewData::default(),
            assets_activity: host_window::AssetsActivityPaneViewData::default(),
            asset_browser: host_window::AssetBrowserPaneViewData::default(),
            project_overview: host_window::ProjectOverviewPaneViewData::default(),
            module_plugins: host_window::ModulePluginsPaneViewData::default(),
            build_export: host_window::BuildExportPaneViewData::default(),
            ui_asset: crate::ui::asset_editor::UiAssetEditorPanePresentation::default(),
            animation: host_window::AnimationEditorPaneViewData::default(),
        },
        pane_presentation: Some(host_window::PanePresentation::new(
            host_window::PaneShellPresentation::new(
                "UI Component Showcase",
                "ui-components",
                "Runtime components",
                "",
                None,
                false,
                crate::ui::layouts::views::blank_viewport_chrome(),
            ),
            body,
        )),
    };
    let projected = to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
        &workbench_pane,
        host_window::PaneContentSize::new(width, height),
        runtime,
    );

    let mut pane = PaneData {
        kind: "UiComponentShowcase".into(),
        title: "UI Component Showcase".into(),
        project_overview: projected,
        ..PaneData::default()
    };
    pane.body_surface_frame = build_pane_template_surface_frame(&pane, UiSize::new(width, height));
    pane
}

fn template_node_by_control_id(pane: &PaneData, control_id: &str) -> TemplatePaneNodeData {
    (0..pane.project_overview.nodes.row_count())
        .filter_map(|row| pane.project_overview.nodes.row_data(row))
        .find(|node| node.control_id.as_str() == control_id)
        .unwrap_or_else(|| panic!("projected pane should expose {control_id}"))
}

fn hierarchy_pane(nodes: Vec<SceneNodeData>) -> PaneData {
    let mut pane = PaneData {
        kind: "Hierarchy".into(),
        title: "Hierarchy".into(),
        ..PaneData::default()
    };
    pane.hierarchy.hierarchy_nodes = model_rc(nodes);
    pane
}

fn asset_tree_pane() -> PaneData {
    let mut pane = PaneData {
        kind: "Assets".into(),
        title: "Assets".into(),
        ..PaneData::default()
    };
    pane.assets_activity.nodes = model_rc(vec![template_node(
        "AssetsActivityTreeRowPanel",
        "Panel",
        "Assets",
        8.0,
        57.0,
        220.0,
        28.0,
    )]);
    pane
}

fn scene_node(id: &str, name: &str, depth: i32, selected: bool) -> SceneNodeData {
    SceneNodeData {
        id: id.into(),
        name: name.into(),
        depth,
        selected,
    }
}

fn template_node_with_action(
    control_id: &str,
    role: &str,
    text: &str,
    action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        action_id: action_id.into(),
        dispatch_kind: "click".into(),
        button_variant: "primary".into(),
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

fn template_node_with_binding(
    control_id: &str,
    role: &str,
    text: &str,
    binding_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        binding_id: binding_id.into(),
        button_variant: "primary".into(),
        ..template_node(control_id, role, text, x, y, width, height)
    }
}

fn welcome_text_node(
    control_id: &str,
    edit_action_id: &str,
    value_text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_role: "input-field".into(),
        dispatch_kind: "welcome_text".into(),
        action_id: edit_action_id.into(),
        edit_action_id: edit_action_id.into(),
        value_text: value_text.into(),
        surface_variant: "inset".into(),
        ..template_node(control_id, "LineEdit", value_text, x, y, width, height)
    }
}

fn template_input_node(
    control_id: &str,
    value_text: &str,
    edit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_role: "input-field".into(),
        edit_action_id: edit_action_id.into(),
        value_text: value_text.into(),
        surface_variant: "inset".into(),
        ..template_node(control_id, "InputField", value_text, x, y, width, height)
    }
}

fn template_input_node_with_binding(
    control_id: &str,
    value_text: &str,
    binding_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        component_role: "input-field".into(),
        binding_id: binding_id.into(),
        value_text: value_text.into(),
        surface_variant: "inset".into(),
        ..template_node(control_id, "InputField", value_text, x, y, width, height)
    }
}

fn template_input_node_with_commit(
    control_id: &str,
    value_text: &str,
    edit_action_id: &str,
    commit_action_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        commit_action_id: commit_action_id.into(),
        ..template_input_node(control_id, value_text, edit_action_id, x, y, width, height)
    }
}

fn welcome_button_node(
    control_id: &str,
    action_id: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        dispatch_kind: "welcome".into(),
        action_id: action_id.into(),
        button_variant: "primary".into(),
        ..template_node(control_id, "Button", text, x, y, width, height)
    }
}

fn template_node(
    control_id: &str,
    role: &str,
    text: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> TemplatePaneNodeData {
    TemplatePaneNodeData {
        node_id: format!("{control_id}.node").into(),
        control_id: control_id.into(),
        role: role.into(),
        text: text.into(),
        surface_variant: "panel".into(),
        border_width: 1.0,
        frame: TemplateNodeFrameData {
            x,
            y,
            width,
            height,
        },
        ..TemplatePaneNodeData::default()
    }
}

fn control_frame(
    control_id: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HostChromeControlFrameData {
    HostChromeControlFrameData {
        control_id: control_id.into(),
        frame: host_frame(x, y, width, height),
    }
}

fn chrome_tab(
    control_id: &str,
    title: &str,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
) -> HostChromeTabData {
    HostChromeTabData {
        control_id: control_id.into(),
        tab: tab_data(control_id, title),
        frame: host_frame(x, y, width, height),
        close_frame: host_frame(x + width - 20.0, y + 4.0, 16.0, 16.0),
    }
}

fn tab_data(id: &str, title: &str) -> TabData {
    TabData {
        id: id.into(),
        title: title.into(),
        active: true,
        closeable: true,
        ..TabData::default()
    }
}

fn model_rc<T: Clone + 'static>(values: Vec<T>) -> ModelRc<T> {
    ModelRc::from(Rc::new(VecModel::from(values)))
}

fn changed_pixel_count(
    width: u32,
    left: &[u8],
    right: &[u8],
    x: u32,
    y: u32,
    region_width: u32,
    region_height: u32,
) -> usize {
    let x1 = x.saturating_add(region_width).min(width);
    let y1 = y
        .saturating_add(region_height)
        .min((left.len() / 4 / width as usize) as u32)
        .min((right.len() / 4 / width as usize) as u32);
    (y..y1)
        .flat_map(|row| (x..x1).map(move |column| (column, row)))
        .filter(|(column, row)| {
            pixel(width, left, *column, *row) != pixel(width, right, *column, *row)
        })
        .count()
}

fn pixel(width: u32, bytes: &[u8], x: u32, y: u32) -> [u8; 4] {
    let offset = ((y as usize * width as usize) + x as usize) * 4;
    [
        bytes[offset],
        bytes[offset + 1],
        bytes[offset + 2],
        bytes[offset + 3],
    ]
}
