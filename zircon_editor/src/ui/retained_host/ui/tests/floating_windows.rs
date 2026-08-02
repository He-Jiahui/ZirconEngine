use super::*;

#[test]
fn floating_windows_project_tabs_and_active_pane_for_host_presentation() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.game"),
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
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(game_instance.instance_id.clone()),
        frame: crate::ui::workbench::autolayout::ShellFrame::default(),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &WorkbenchShellGeometry::default(),
        &ui_asset_panes,
        &BTreeMap::new(),
        None,
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
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.game"),
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
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(ViewInstanceId::new("editor.missing#1")),
        frame: crate::ui::workbench::autolayout::ShellFrame::default(),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &WorkbenchShellGeometry::default(),
        &ui_asset_panes,
        &BTreeMap::new(),
        None,
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
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    let game_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.game#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.game"),
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
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![
                scene_instance.instance_id.clone(),
                game_instance.instance_id.clone(),
            ],
            active_tab: Some(game_instance.instance_id.clone()),
        }),
        focused_view: Some(game_instance.instance_id.clone()),
        frame: crate::ui::workbench::autolayout::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let geometry = crate::ui::workbench::autolayout::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ui::workbench::autolayout::ShellSizePx::new(1440.0, 900.0),
        1.0,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        None,
    );
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &ui_asset_panes,
        &BTreeMap::new(),
        None,
        &floating_window_projection_bundle,
    );

    let snapshot =
        EditorUiCompatibilityHarness::capture_floating_window_overlay_snapshot(&floating_windows);

    assert!(
        snapshot
            .frame_entries
            .contains(&"floating-window/window:preview=420,180,360,240".to_string())
    );
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.attach=floating-window/window:preview".to_string()
    ));
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.left=floating-window-edge/window:preview/left".to_string()
    ));
    assert!(
        snapshot.route_key_entries.contains(
            &"floating-window/window:preview.right=floating-window-edge/window:preview/right"
                .to_string()
        )
    );
    assert!(snapshot.route_key_entries.contains(
        &"floating-window/window:preview.top=floating-window-edge/window:preview/top".to_string()
    ));
    assert!(
        snapshot.route_key_entries.contains(
            &"floating-window/window:preview.bottom=floating-window-edge/window:preview/bottom"
                .to_string()
        )
    );
    assert!(
        snapshot
            .attribute_entries
            .contains(&"floating-window/window:preview.title=Preview Popout".to_string())
    );
    assert!(
        snapshot.attribute_entries.contains(
            &"floating-window/window:preview.focus_target_id=editor.game#float".to_string()
        )
    );
    assert!(
        snapshot.attribute_entries.contains(
            &"floating-window/window:preview.active_pane.id=editor.game#float".to_string()
        )
    );
    assert!(
        snapshot
            .attribute_entries
            .contains(&"floating-window/window:preview.active_pane.kind=Game".to_string())
    );
}

#[test]
fn floating_window_overlay_route_keys_match_shared_shell_pointer_route_normalization() {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id.clone()),
        frame: crate::ui::workbench::autolayout::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let geometry = crate::ui::workbench::autolayout::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ui::workbench::autolayout::ShellSizePx::new(1440.0, 900.0),
        1.0,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        None,
    );
    let ui_asset_panes = BTreeMap::new();
    let floating_window_projection_bundle = build_floating_window_projection_bundle(
        &model,
        None,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        &[],
    );
    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &ui_asset_panes,
        &BTreeMap::new(),
        None,
        &floating_window_projection_bundle,
    );
    let window = &floating_windows[0];

    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindow(
            window_id.clone()
        )),
        Some(window.target_group.to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Left,
        }),
        Some(window.left_edge_target_group.to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Right,
        }),
        Some(window.right_edge_target_group.to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Top,
        }),
        Some(window.top_edge_target_group.to_string())
    );
    assert_eq!(
        host_shell_pointer_route_group_key(&HostShellPointerRoute::FloatingWindowEdge {
            window_id,
            edge: DockEdge::Bottom,
        }),
        Some(window.bottom_edge_target_group.to_string())
    );
}

#[test]
fn collect_floating_windows_does_not_fall_back_to_legacy_geometry_when_projection_bundle_is_explicitly_provided()
 {
    let mut fixture = default_preview_fixture();
    let window_id = MainPageId::new("window:preview");
    let scene_instance = ViewInstance {
        instance_id: ViewInstanceId::new("editor.scene#float"),
        descriptor_id: crate::ui::workbench::view::ViewDescriptorId::new("editor.scene"),
        title: "Scene".to_string(),
        serializable_payload: serde_json::json!({ "path": "crate://scene/floating.scene" }),
        dirty: false,
        host: ViewHost::FloatingWindow(window_id.clone(), vec![]),
    };
    fixture.instances.push(scene_instance.clone());
    fixture.layout.floating_windows.push(FloatingWindowLayout {
        window_id: window_id.clone(),
        title: "Preview Popout".to_string(),
        workspace: DocumentNode::Tabs(TabStackLayout {
            tabs: vec![scene_instance.instance_id.clone()],
            active_tab: Some(scene_instance.instance_id.clone()),
        }),
        focused_view: Some(scene_instance.instance_id.clone()),
        frame: crate::ui::workbench::autolayout::ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    });

    let chrome = fixture.build_chrome();
    let model = WorkbenchViewModel::build(
        &crate::core::commands::EditorCommandRegistry::default_workbench(),
        &chrome,
    );
    let mut geometry = crate::ui::workbench::autolayout::compute_workbench_shell_geometry(
        &model,
        &chrome,
        &fixture.layout,
        &fixture.descriptors,
        crate::ui::workbench::autolayout::ShellSizePx::new(1440.0, 900.0),
        1.0,
        &crate::ui::workbench::autolayout::WorkbenchChromeMetrics::default(),
        None,
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        crate::ui::workbench::autolayout::ShellFrame::new(96.0, 72.0, 144.0, 88.0),
    );

    let floating_windows = collect_floating_windows(
        &model,
        &chrome,
        &geometry,
        &BTreeMap::new(),
        &BTreeMap::new(),
        None,
        &FloatingWindowProjectionBundle::default(),
    );

    assert_eq!(floating_windows.len(), 1);
    assert_eq!(floating_windows[0].frame.x, 0.0);
    assert_eq!(floating_windows[0].frame.y, 0.0);
    assert_eq!(floating_windows[0].frame.width, 0.0);
    assert_eq!(floating_windows[0].frame.height, 0.0);
}
