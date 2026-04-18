use slint::{ComponentHandle, Model};
use std::collections::BTreeMap;

use super::apply_presentation;
use super::floating_windows::collect_floating_windows;
use super::pane_projection::document_pane;
use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchTemplateBridge;
use crate::ui::slint_host::floating_window_projection::{
    build_floating_window_projection_bundle, FloatingWindowProjectionBundle,
};
use crate::ui::slint_host::shell_pointer::WorkbenchShellPointerRoute;
use crate::{
    default_preview_fixture, ui::slint_host::tab_drag::workbench_shell_pointer_route_group_key,
    EditorChromeSnapshot, EditorUiCompatibilityHarness, FloatingWindowLayout, MainPageId,
    TabStackLayout, ViewHost, ViewInstance, ViewInstanceId, WorkbenchViewModel,
};
use zircon_scene::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};
use zircon_ui::UiSize;

fn root_shell_fixture() -> (
    crate::PreviewFixture,
    EditorChromeSnapshot,
    WorkbenchViewModel,
    BTreeMap<String, crate::UiAssetEditorPanePresentation>,
) {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    (fixture, chrome, model, BTreeMap::new())
}

#[test]
fn apply_presentation_uses_shared_root_projection_frames_when_drawers_are_collapsed() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let projection_frames = bridge.root_shell_frames();
    let geometry = crate::WorkbenchShellGeometry {
        center_band_frame: crate::ShellFrame::new(9.0, 19.0, 333.0, 444.0),
        status_bar_frame: crate::ShellFrame::new(15.0, 520.0, 640.0, 18.0),
        region_frames: [
            (crate::ShellRegionId::Left, crate::ShellFrame::default()),
            (
                crate::ShellRegionId::Document,
                crate::ShellFrame::new(21.0, 37.0, 410.0, 250.0),
            ),
            (crate::ShellRegionId::Right, crate::ShellFrame::default()),
            (crate::ShellRegionId::Bottom, crate::ShellFrame::default()),
        ]
        .into_iter()
        .collect(),
        ..crate::WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let center_band = ui.get_center_band_frame();
    assert_eq!(center_band.x, 0.0);
    assert_eq!(center_band.y, 40.0);
    assert_eq!(center_band.width, 1280.0);
    assert_eq!(center_band.height, 656.0);

    let document_region = ui.get_document_region_frame();
    assert_eq!(document_region.x, 56.0);
    assert_eq!(document_region.y, 40.0);
    assert_eq!(document_region.width, 1224.0);
    assert_eq!(document_region.height, 656.0);

    let status_bar = ui.get_status_bar_frame();
    assert_eq!(status_bar.x, 0.0);
    assert_eq!(status_bar.y, 696.0);
    assert_eq!(status_bar.width, 1280.0);
    assert_eq!(status_bar.height, 24.0);

    let viewport_content = ui.get_viewport_content_frame();
    assert_eq!(viewport_content.x, 56.0);
    assert_eq!(viewport_content.y, 100.0);
    assert_eq!(viewport_content.width, 1224.0);
    assert_eq!(viewport_content.height, 596.0);
}

#[test]
fn apply_presentation_prefers_shared_root_projection_for_visible_drawer_document_region() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let projection_frames = bridge.root_shell_frames();
    let shell_frame = projection_frames
        .shell_frame
        .expect("root shell projection frame should exist");
    let body_frame = projection_frames
        .workbench_body_frame
        .expect("workbench body projection frame should exist");
    let metrics = crate::WorkbenchChromeMetrics::default();
    let left_geometry = crate::ShellFrame::new(180.0, 91.0, 312.0, 440.0);
    let right_geometry = crate::ShellFrame::new(1024.0, 117.0, 256.0, 440.0);
    let bottom_geometry = crate::ShellFrame::new(48.0, 712.0, 1232.0, 180.0);
    let expected_document_frame = crate::ShellFrame::new(
        shell_frame.x + left_geometry.width + metrics.separator_thickness,
        body_frame.y,
        body_frame.width
            - left_geometry.width
            - right_geometry.width
            - metrics.separator_thickness * 2.0,
        body_frame.height - bottom_geometry.height - metrics.separator_thickness,
    );
    let geometry_document_frame = crate::ShellFrame::new(734.0, 91.0, 222.0, 109.0);
    let expected_viewport_frame = crate::ShellFrame::new(
        expected_document_frame.x,
        expected_document_frame.y + metrics.viewport_toolbar_height,
        expected_document_frame.width,
        expected_document_frame.height - metrics.viewport_toolbar_height,
    );
    let geometry_viewport_frame = crate::ShellFrame::new(888.0, 144.0, 155.0, 77.0);
    let geometry = crate::WorkbenchShellGeometry {
        center_band_frame: crate::ShellFrame::new(5.0, 17.0, 400.0, 500.0),
        status_bar_frame: crate::ShellFrame::new(11.0, 520.0, 700.0, 18.0),
        region_frames: [
            (crate::ShellRegionId::Left, left_geometry),
            (crate::ShellRegionId::Document, geometry_document_frame),
            (crate::ShellRegionId::Right, right_geometry),
            (crate::ShellRegionId::Bottom, bottom_geometry),
        ]
        .into_iter()
        .collect(),
        viewport_content_frame: geometry_viewport_frame,
        ..crate::WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let center_band = ui.get_center_band_frame();
    assert_eq!(center_band.x, 0.0);
    assert_eq!(center_band.y, 40.0);
    assert_eq!(center_band.width, 1280.0);
    assert_eq!(center_band.height, 656.0);

    let document_region = ui.get_document_region_frame();
    assert_eq!(document_region.x, expected_document_frame.x);
    assert_eq!(document_region.y, expected_document_frame.y);
    assert_eq!(document_region.width, expected_document_frame.width);
    assert_eq!(document_region.height, expected_document_frame.height);

    let status_bar = ui.get_status_bar_frame();
    assert_eq!(status_bar.x, 0.0);
    assert_eq!(status_bar.y, 696.0);
    assert_eq!(status_bar.width, 1280.0);
    assert_eq!(status_bar.height, 24.0);

    let viewport_content = ui.get_viewport_content_frame();
    assert_eq!(viewport_content.x, expected_viewport_frame.x);
    assert_eq!(viewport_content.y, expected_viewport_frame.y);
    assert_eq!(viewport_content.width, expected_viewport_frame.width);
    assert_eq!(viewport_content.height, expected_viewport_frame.height);
}

#[test]
fn apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_positions() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    let projection_frames = bridge.root_shell_frames();
    let shell_frame = projection_frames
        .shell_frame
        .expect("root shell projection frame should exist");
    let body_frame = projection_frames
        .workbench_body_frame
        .expect("workbench body projection frame should exist");
    let metrics = crate::WorkbenchChromeMetrics::default();
    let left_geometry = crate::ShellFrame::new(180.0, 91.0, 312.0, 519.0);
    let right_geometry = crate::ShellFrame::new(1024.0, 117.0, 256.0, 401.0);
    let bottom_geometry = crate::ShellFrame::new(48.0, 712.0, 777.0, 180.0);
    let expected_center_height =
        body_frame.height - bottom_geometry.height - metrics.separator_thickness;
    let geometry = crate::WorkbenchShellGeometry {
        center_band_frame: crate::ShellFrame::new(5.0, 17.0, 400.0, 500.0),
        status_bar_frame: crate::ShellFrame::new(11.0, 520.0, 700.0, 18.0),
        region_frames: [
            (crate::ShellRegionId::Left, left_geometry),
            (
                crate::ShellRegionId::Document,
                crate::ShellFrame::new(493.0, 91.0, 531.0, 440.0),
            ),
            (crate::ShellRegionId::Right, right_geometry),
            (crate::ShellRegionId::Bottom, bottom_geometry),
        ]
        .into_iter()
        .collect(),
        ..crate::WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    let left_region = ui.get_left_region_frame();
    assert_eq!(left_region.x, shell_frame.x);
    assert_eq!(left_region.y, body_frame.y);
    assert_eq!(left_region.width, left_geometry.width);
    assert_eq!(left_region.height, expected_center_height);

    let right_region = ui.get_right_region_frame();
    assert_eq!(
        right_region.x,
        shell_frame.x + shell_frame.width - right_geometry.width
    );
    assert_eq!(right_region.y, body_frame.y);
    assert_eq!(right_region.width, right_geometry.width);
    assert_eq!(right_region.height, expected_center_height);

    let bottom_region = ui.get_bottom_region_frame();
    assert_eq!(bottom_region.x, shell_frame.x);
    assert_eq!(
        bottom_region.y,
        body_frame.y + body_frame.height - bottom_geometry.height
    );
    assert_eq!(bottom_region.width, body_frame.width);
    assert_eq!(bottom_region.height, bottom_geometry.height);
}

#[test]
fn apply_presentation_prefers_shared_root_projection_for_visible_drawer_region_extents() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let mut bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &crate::WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let geometry = crate::WorkbenchShellGeometry {
        center_band_frame: crate::ShellFrame::new(5.0, 17.0, 400.0, 500.0),
        status_bar_frame: crate::ShellFrame::new(11.0, 520.0, 700.0, 18.0),
        region_frames: [
            (
                crate::ShellRegionId::Left,
                crate::ShellFrame::new(180.0, 91.0, 180.0, 519.0),
            ),
            (
                crate::ShellRegionId::Document,
                crate::ShellFrame::new(493.0, 91.0, 531.0, 440.0),
            ),
            (
                crate::ShellRegionId::Right,
                crate::ShellFrame::new(1024.0, 117.0, 144.0, 401.0),
            ),
            (
                crate::ShellRegionId::Bottom,
                crate::ShellFrame::new(48.0, 712.0, 777.0, 120.0),
            ),
        ]
        .into_iter()
        .collect(),
        ..crate::WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    assert_eq!(
        ui.get_left_region_frame(),
        crate::ui::slint_host::FrameRect {
            x: projection_frames.left_drawer_shell_frame.unwrap().x,
            y: projection_frames.left_drawer_shell_frame.unwrap().y,
            width: projection_frames.left_drawer_shell_frame.unwrap().width,
            height: projection_frames.left_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        ui.get_right_region_frame(),
        crate::ui::slint_host::FrameRect {
            x: projection_frames.right_drawer_shell_frame.unwrap().x,
            y: projection_frames.right_drawer_shell_frame.unwrap().y,
            width: projection_frames.right_drawer_shell_frame.unwrap().width,
            height: projection_frames.right_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        ui.get_bottom_region_frame(),
        crate::ui::slint_host::FrameRect {
            x: projection_frames.bottom_drawer_shell_frame.unwrap().x,
            y: projection_frames.bottom_drawer_shell_frame.unwrap().y,
            width: projection_frames.bottom_drawer_shell_frame.unwrap().width,
            height: projection_frames.bottom_drawer_shell_frame.unwrap().height,
        }
    );
}

#[test]
fn apply_presentation_prefers_shared_visible_drawer_projection_when_legacy_geometry_is_zeroed() {
    i_slint_backend_testing::init_no_event_loop();

    let (_fixture, chrome, model, ui_asset_panes) = root_shell_fixture();
    let ui =
        crate::ui::slint_host::UiHostWindow::new().expect("workbench shell should instantiate");
    ui.show()
        .expect("workbench shell should show in the test backend");
    ui.window().set_size(slint::PhysicalSize::new(1280, 720));

    let mut bridge = BuiltinWorkbenchTemplateBridge::new(UiSize::new(1280.0, 720.0)).unwrap();
    bridge
        .recompute_layout_with_workbench_model(
            UiSize::new(1280.0, 720.0),
            &model,
            &crate::WorkbenchChromeMetrics::default(),
        )
        .unwrap();
    let projection_frames = bridge.root_shell_frames();
    let geometry = crate::WorkbenchShellGeometry {
        center_band_frame: crate::ShellFrame::new(5.0, 17.0, 400.0, 500.0),
        status_bar_frame: crate::ShellFrame::new(11.0, 520.0, 700.0, 18.0),
        region_frames: [
            (crate::ShellRegionId::Left, crate::ShellFrame::default()),
            (
                crate::ShellRegionId::Document,
                crate::ShellFrame::new(21.0, 37.0, 410.0, 250.0),
            ),
            (crate::ShellRegionId::Right, crate::ShellFrame::default()),
            (crate::ShellRegionId::Bottom, crate::ShellFrame::default()),
        ]
        .into_iter()
        .collect(),
        viewport_content_frame: crate::ShellFrame::new(66.0, 120.0, 320.0, 180.0),
        ..crate::WorkbenchShellGeometry::default()
    };
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );

    apply_presentation(
        &ui,
        &model,
        &chrome,
        &geometry,
        &[],
        None,
        &ui_asset_panes,
        Some(&projection_frames),
        &floating_window_projection_bundle,
    );

    assert_eq!(
        ui.get_left_region_frame(),
        crate::ui::slint_host::FrameRect {
            x: projection_frames.left_drawer_shell_frame.unwrap().x,
            y: projection_frames.left_drawer_shell_frame.unwrap().y,
            width: projection_frames.left_drawer_shell_frame.unwrap().width,
            height: projection_frames.left_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        ui.get_right_region_frame(),
        crate::ui::slint_host::FrameRect {
            x: projection_frames.right_drawer_shell_frame.unwrap().x,
            y: projection_frames.right_drawer_shell_frame.unwrap().y,
            width: projection_frames.right_drawer_shell_frame.unwrap().width,
            height: projection_frames.right_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        ui.get_bottom_region_frame(),
        crate::ui::slint_host::FrameRect {
            x: projection_frames.bottom_drawer_shell_frame.unwrap().x,
            y: projection_frames.bottom_drawer_shell_frame.unwrap().y,
            width: projection_frames.bottom_drawer_shell_frame.unwrap().width,
            height: projection_frames.bottom_drawer_shell_frame.unwrap().height,
        }
    );
    assert_eq!(
        ui.get_document_region_frame(),
        crate::ui::slint_host::FrameRect {
            x: 313.0,
            y: 40.0,
            width: 658.0,
            height: 491.0,
        }
    );
    assert_eq!(
        ui.get_viewport_content_frame(),
        crate::ui::slint_host::FrameRect {
            x: 313.0,
            y: 68.0,
            width: 658.0,
            height: 463.0,
        }
    );
}

#[test]
fn scene_document_pane_uses_viewport_dimensions_and_enables_toolbar() {
    let fixture = default_preview_fixture();
    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let ui_asset_panes = BTreeMap::new();

    let pane = document_pane(&model, &chrome, &ui_asset_panes);

    assert_eq!(pane.kind, "Scene");
    assert_eq!(pane.subtitle, "1280 x 720");
    assert!(pane.show_toolbar);
}

#[test]
fn scene_document_pane_projects_viewport_toolbar_state() {
    let mut fixture = default_preview_fixture();
    fixture.editor.scene_viewport_settings.tool = SceneViewportTool::Scale;
    fixture.editor.scene_viewport_settings.transform_space = TransformSpace::Global;
    fixture.editor.scene_viewport_settings.projection_mode = ProjectionMode::Orthographic;
    fixture.editor.scene_viewport_settings.view_orientation = ViewOrientation::NegZ;
    fixture.editor.scene_viewport_settings.display_mode = DisplayMode::WireOverlay;
    fixture.editor.scene_viewport_settings.grid_mode = GridMode::VisibleAndSnap;
    fixture.editor.scene_viewport_settings.gizmos_enabled = false;
    fixture.editor.scene_viewport_settings.preview_lighting = false;
    fixture.editor.scene_viewport_settings.preview_skybox = false;
    fixture.editor.scene_viewport_settings.translate_step = 2.5;
    fixture.editor.scene_viewport_settings.rotate_step_deg = 30.0;
    fixture.editor.scene_viewport_settings.scale_step = 0.25;

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let ui_asset_panes = BTreeMap::new();
    let pane = document_pane(&model, &chrome, &ui_asset_panes);

    assert_eq!(pane.kind, "Scene");
    assert_eq!(pane.viewport.tool, "Scale");
    assert_eq!(pane.viewport.transform_space, "Global");
    assert_eq!(pane.viewport.projection_mode, "Orthographic");
    assert_eq!(pane.viewport.view_orientation, "NegZ");
    assert_eq!(pane.viewport.display_mode, "WireOverlay");
    assert_eq!(pane.viewport.grid_mode, "VisibleAndSnap");
    assert!(!pane.viewport.gizmos_enabled);
    assert!(!pane.viewport.preview_lighting);
    assert!(!pane.viewport.preview_skybox);
    assert_eq!(pane.viewport.translate_snap, 2.5);
    assert_eq!(pane.viewport.rotate_snap_deg, 30.0);
    assert_eq!(pane.viewport.scale_snap, 0.25);
    assert_eq!(pane.viewport.translate_snap_label, "T 2.5");
    assert_eq!(pane.viewport.rotate_snap_label, "R 30");
    assert_eq!(pane.viewport.scale_snap_label, "S 0.25");
}

#[test]
fn floating_windows_project_tabs_and_active_pane_for_host_presentation() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ViewDescriptorId::new("editor.game"),
        title: "Game".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.play" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.instances.push(game_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: crate::DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(game_instance.instance_id.clone()),
        frame: crate::ShellFrame::default(),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &crate::WorkbenchShellGeometry::default(),
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &crate::WorkbenchShellGeometry::default(),
        &ui_asset_panes,
        &floating_window_projection_bundle,
    );

    assert_eq!(floating_windows.len(), 1);
    assert_eq!(floating_windows[0].window_id, "window:preview");
    assert_eq!(floating_windows[0].title, "Preview Popout");
    assert_eq!(
        floating_windows[0]
            .tabs
            .iter()
            .map(|tab| (tab.title.to_string(), tab.active))
            .collect::<Vec<_>>(),
        vec![("Scene".to_string(), false), ("Game".to_string(), true)]
    );
    assert_eq!(floating_windows[0].focus_target_id, "editor.game#float");
    assert_eq!(floating_windows[0].active_pane.title, "Game");
    assert_eq!(floating_windows[0].active_pane.kind, "Game");
}

#[test]
fn floating_windows_ignore_stale_focused_view_when_projecting_focus_target() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ViewDescriptorId::new("editor.game"),
        title: "Game".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.play" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.instances.push(game_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: crate::DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(ViewInstanceId::new("editor.missing#1")),
        frame: crate::ShellFrame::default(),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &crate::WorkbenchShellGeometry::default(),
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &crate::WorkbenchShellGeometry::default(),
        &ui_asset_panes,
        &floating_window_projection_bundle,
    );

    assert_eq!(floating_windows.len(), 1);
    assert_eq!(floating_windows[0].focus_target_id, "editor.game#float");
    assert_eq!(floating_windows[0].active_pane.id, "editor.game#float");
    assert_eq!(floating_windows[0].active_pane.kind, "Game");
}

#[test]
fn floating_window_overlay_snapshot_captures_shared_frame_and_route_keys() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ViewDescriptorId::new("editor.game"),
        title: "Game".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.play" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.instances.push(game_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: crate::DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(game_instance.instance_id.clone()),
        frame: crate::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = crate::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ShellSizePx::new(1440.0, 900.0),
        &crate::WorkbenchChromeMetrics::default(),
        None,
    );
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &ui_asset_panes,
        &floating_window_projection_bundle,
    );

    let snapshot =
        EditorUiCompatibilityHarness::capture_floating_window_overlay_snapshot(&floating_windows);

    assert!(snapshot
        .frame_entries
        .contains(&"floating-window/window:preview=420,180,360,240".to_string()));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.attach=floating-window/window:preview".to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.left=floating-window-edge/window:preview/left".to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.right=floating-window-edge/window:preview/right"
            .to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.top=floating-window-edge/window:preview/top".to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.bottom=floating-window-edge/window:preview/bottom"
            .to_string()
    ));
    assert!(snapshot
        .attribute_entries
        .contains(&"floating-window/window:preview.title=Preview Popout".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"floating-window/window:preview.focus_target_id=editor.game#float".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"floating-window/window:preview.active_pane.id=editor.game#float".to_string()));
    assert!(snapshot
        .attribute_entries
        .contains(&"floating-window/window:preview.active_pane.kind=Game".to_string()));
}

#[test]
fn floating_window_overlay_route_keys_match_shared_shell_pointer_route_normalization() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: crate::DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id.clone()),
        frame: crate::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let geometry = crate::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ShellSizePx::new(1440.0, 900.0),
        &crate::WorkbenchChromeMetrics::default(),
        None,
    );
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &ui_asset_panes,
        &floating_window_projection_bundle,
    );
    let window = &floating_windows[0];

    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindow(
            window_id.clone()
        )),
        Some(window.target_group.to_string())
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: crate::DockEdge::Left,
        }),
        Some(window.left_edge_target_group.to_string())
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: crate::DockEdge::Right,
        }),
        Some(window.right_edge_target_group.to_string())
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: crate::DockEdge::Top,
        }),
        Some(window.top_edge_target_group.to_string())
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindowEdge {
            window_id,
            edge: crate::DockEdge::Bottom,
        }),
        Some(window.bottom_edge_target_group.to_string())
    );
}

#[test]
fn collect_floating_windows_does_not_fall_back_to_legacy_geometry_when_projection_bundle_is_explicitly_provided(
) {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: crate::DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id.clone()),
        frame: crate::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(&chrome);
    let mut geometry = crate::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ShellSizePx::new(1440.0, 900.0),
        &crate::WorkbenchChromeMetrics::default(),
        None,
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        crate::ShellFrame::new(96.0, 72.0, 144.0, 88.0),
    );

    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &BTreeMap::new(),
        &FloatingWindowProjectionBundle::default(),
    );

    assert_eq!(floating_windows.len(), 1);
    assert_eq!(floating_windows[0].frame.x, 0.0);
    assert_eq!(floating_windows[0].frame.y, 0.0);
    assert_eq!(floating_windows[0].frame.width, 0.0);
    assert_eq!(floating_windows[0].frame.height, 0.0);
}
