use slint::Model;
use std::collections::BTreeMap;

use super::floating_windows::collect_floating_windows;
use super::pane_projection::document_pane;
use crate::host::slint_host::shell_pointer::WorkbenchShellPointerRoute;
use crate::{
    default_preview_fixture, host::slint_host::tab_drag::workbench_shell_pointer_route_group_key,
    EditorUiCompatibilityHarness, FloatingWindowLayout, MainPageId, TabStackLayout, ViewHost,
    ViewInstance, ViewInstanceId, WorkbenchViewModel,
};
use zircon_scene::{
    DisplayMode, GridMode, ProjectionMode, SceneViewportTool, TransformSpace, ViewOrientation,
};

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
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &crate::WorkbenchShellGeometry::default(),
        &ui_asset_panes,
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
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &crate::WorkbenchShellGeometry::default(),
        &ui_asset_panes,
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
    let floating_windows = collect_floating_windows(&model, &chrome, &geometry, &ui_asset_panes);

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
    let floating_windows = collect_floating_windows(&model, &chrome, &geometry, &ui_asset_panes);
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
