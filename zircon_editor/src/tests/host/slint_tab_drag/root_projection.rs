use super::support::*;

#[test]
fn shared_shell_pointer_route_reports_document_edge_before_document_group() {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 50.0, 1440.0, 830.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (ShellRegionId::Left, ShellFrame::new(0.0, 50.0, 0.0, 738.0)),
            (
                ShellRegionId::Document,
                ShellFrame::new(0.0, 50.0, 1440.0, 738.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1440.0, 50.0, 0.0, 738.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
            ),
        ]),
        splitter_frames: BTreeMap::new(),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(0.0, 0.0, 0.0, 0.0),
    };
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        UiSize::new(1440.0, 900.0),
        &geometry,
        false,
        &[],
        &[],
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(12.0, 240.0)),
        Some(HostShellPointerRoute::DocumentEdge(DockEdge::Left))
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(12.0, 240.0)),
        Some(HostDragTargetGroup::Document)
    );
}

#[test]
fn shared_shell_pointer_route_uses_shared_root_projection_document_bounds_when_drawers_are_collapsed(
) {
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 50.0, 1440.0, 650.0),
        status_bar_frame: ShellFrame::new(0.0, 700.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (ShellRegionId::Left, ShellFrame::new(0.0, 50.0, 0.0, 650.0)),
            (
                ShellRegionId::Document,
                ShellFrame::new(21.0, 50.0, 1419.0, 650.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1440.0, 50.0, 0.0, 650.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(0.0, 700.0, 1440.0, 0.0),
            ),
        ]),
        splitter_frames: BTreeMap::new(),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::default(),
    };
    let root_projection = BuiltinHostRootShellFrames {
        host_body_frame: Some(UiFrame::new(0.0, 40.0, 1440.0, 656.0)),
        document_host_frame: Some(UiFrame::new(56.0, 40.0, 1384.0, 656.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 696.0, 1440.0, 24.0)),
        ..BuiltinHostRootShellFrames::default()
    };
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_root_shell_frames(
        UiSize::new(1440.0, 720.0),
        &geometry,
        false,
        &[],
        Some(&root_projection),
        None,
    );

    assert_eq!(bridge.drag_route_at(UiPoint::new(40.0, 240.0)), None);
    assert_eq!(
        bridge.drag_route_at(UiPoint::new(80.0, 240.0)),
        Some(HostShellPointerRoute::DocumentEdge(DockEdge::Left))
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(80.0, 240.0)),
        Some(HostDragTargetGroup::Document)
    );
    assert_eq!(
        bridge.drag_route_at(UiPoint::new(160.0, 240.0)),
        Some(HostShellPointerRoute::DragTarget(
            HostDragTargetGroup::Document,
        ))
    );
}

#[test]
fn resolve_host_tab_drop_route_uses_shared_root_projection_tab_strip_when_drawers_are_collapsed() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: BTreeMap::from([
            (
                ActivityDrawerSlot::RightTop,
                drawer(
                    ActivityDrawerSlot::RightTop,
                    &["editor.inspector#1"],
                    Some("editor.inspector#1"),
                    ActivityDrawerMode::Pinned,
                    true,
                ),
            ),
            (
                ActivityDrawerSlot::RightBottom,
                drawer(
                    ActivityDrawerSlot::RightBottom,
                    &["editor.project#1", "editor.console#1"],
                    Some("editor.console#1"),
                    ActivityDrawerMode::Pinned,
                    true,
                ),
            ),
        ]),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        BTreeMap::from([
            (
                ActivityDrawerSlot::RightTop,
                tool_window_stack(
                    ActivityDrawerSlot::RightTop,
                    &[pane_tab("editor.inspector#1", "Inspector", true)],
                    Some("editor.inspector#1"),
                    true,
                ),
            ),
            (
                ActivityDrawerSlot::RightBottom,
                tool_window_stack(
                    ActivityDrawerSlot::RightBottom,
                    &[
                        pane_tab("editor.project#1", "Project", false),
                        pane_tab("editor.console#1", "Console", true),
                    ],
                    Some("editor.console#1"),
                    true,
                ),
            ),
        ]),
        Vec::new(),
        Vec::new(),
    );
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    let root_projection = BuiltinHostRootShellFrames {
        host_body_frame: Some(UiFrame::new(0.0, 40.0, 1440.0, 840.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 880.0, 1440.0, 20.0)),
        ..BuiltinHostRootShellFrames::default()
    };
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_root_shell_frames(
        UiSize::new(1440.0, 900.0),
        &geometry,
        true,
        &[],
        Some(&root_projection),
        None,
    );
    let pointer_x = 1120.0
        + 6.0
        + estimate_dock_tab_width("Inspector")
        + 4.0
        + estimate_dock_tab_width("Project") * 0.25;
    let pointer_route = bridge.drag_route_at(UiPoint::new(pointer_x, 44.0));

    assert_eq!(
        resolve_host_tab_drop_route_with_root_frames(
            &layout,
            &model,
            &geometry,
            &WorkbenchChromeMetrics::default(),
            "editor.hierarchy#1",
            pointer_route,
            "right",
            pointer_x,
            44.0,
            Some(&root_projection),
        ),
        Some(ResolvedHostTabDropRoute {
            target_group: HostDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedHostTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
                anchor: Some(TabInsertionAnchor {
                    target_id: ViewInstanceId::new("editor.project#1"),
                    side: TabInsertionSide::Before,
                }),
            }),
        })
    );
}

#[test]
fn resolve_host_tab_drop_route_uses_shared_root_projection_right_tab_strip_when_visible_drawer_geometry_is_stale(
) {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: BTreeMap::from([
            (
                ActivityDrawerSlot::RightTop,
                drawer(
                    ActivityDrawerSlot::RightTop,
                    &["editor.inspector#1"],
                    Some("editor.inspector#1"),
                    ActivityDrawerMode::Pinned,
                    true,
                ),
            ),
            (
                ActivityDrawerSlot::RightBottom,
                drawer(
                    ActivityDrawerSlot::RightBottom,
                    &["editor.project#1", "editor.console#1"],
                    Some("editor.console#1"),
                    ActivityDrawerMode::Pinned,
                    true,
                ),
            ),
        ]),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        BTreeMap::from([
            (
                ActivityDrawerSlot::RightTop,
                tool_window_stack(
                    ActivityDrawerSlot::RightTop,
                    &[pane_tab("editor.inspector#1", "Inspector", true)],
                    Some("editor.inspector#1"),
                    true,
                ),
            ),
            (
                ActivityDrawerSlot::RightBottom,
                tool_window_stack(
                    ActivityDrawerSlot::RightBottom,
                    &[
                        pane_tab("editor.project#1", "Project", false),
                        pane_tab("editor.console#1", "Console", true),
                    ],
                    Some("editor.console#1"),
                    true,
                ),
            ),
        ]),
        Vec::new(),
        Vec::new(),
    );
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 50.0, 1440.0, 830.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (ShellRegionId::Left, ShellFrame::default()),
            (
                ShellRegionId::Document,
                ShellFrame::new(34.0, 140.0, 960.0, 440.0),
            ),
            (
                ShellRegionId::Right,
                ShellFrame::new(1240.0, 140.0, 320.0, 520.0),
            ),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(96.0, 788.0, 640.0, 92.0),
            ),
        ]),
        splitter_frames: BTreeMap::new(),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::default(),
    };
    let root_projection = BuiltinHostRootShellFrames {
        shell_frame: Some(UiFrame::new(0.0, 0.0, 1440.0, 900.0)),
        host_body_frame: Some(UiFrame::new(0.0, 40.0, 1440.0, 840.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 880.0, 1440.0, 20.0)),
        ..BuiltinHostRootShellFrames::default()
    };
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_root_shell_frames(
        UiSize::new(1440.0, 900.0),
        &geometry,
        true,
        &[],
        Some(&root_projection),
        None,
    );
    let pointer_x = 1120.0
        + 6.0
        + estimate_dock_tab_width("Inspector")
        + 4.0
        + estimate_dock_tab_width("Project") * 0.25;
    let pointer_y = 44.0;
    let pointer_route = bridge.drag_route_at(UiPoint::new(pointer_x, pointer_y));

    assert_eq!(
        resolve_host_tab_drop_route_with_root_frames(
            &layout,
            &model,
            &geometry,
            &WorkbenchChromeMetrics::default(),
            "editor.hierarchy#1",
            pointer_route,
            "right",
            pointer_x,
            pointer_y,
            Some(&root_projection),
        ),
        Some(ResolvedHostTabDropRoute {
            target_group: HostDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedHostTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
                anchor: Some(TabInsertionAnchor {
                    target_id: ViewInstanceId::new("editor.project#1"),
                    side: TabInsertionSide::Before,
                }),
            }),
        })
    );
}

#[test]
fn resolve_host_tab_drop_route_uses_shared_root_projection_bottom_tab_strip_when_visible_drawer_geometry_is_stale(
) {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: BTreeMap::from([
            (
                ActivityDrawerSlot::BottomLeft,
                drawer(
                    ActivityDrawerSlot::BottomLeft,
                    &["editor.console#1"],
                    Some("editor.console#1"),
                    ActivityDrawerMode::Pinned,
                    true,
                ),
            ),
            (
                ActivityDrawerSlot::BottomRight,
                drawer(
                    ActivityDrawerSlot::BottomRight,
                    &["editor.project#1"],
                    Some("editor.project#1"),
                    ActivityDrawerMode::Pinned,
                    true,
                ),
            ),
        ]),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        BTreeMap::from([
            (
                ActivityDrawerSlot::BottomLeft,
                tool_window_stack(
                    ActivityDrawerSlot::BottomLeft,
                    &[pane_tab("editor.console#1", "Console", true)],
                    Some("editor.console#1"),
                    true,
                ),
            ),
            (
                ActivityDrawerSlot::BottomRight,
                tool_window_stack(
                    ActivityDrawerSlot::BottomRight,
                    &[pane_tab("editor.project#1", "Project", false)],
                    Some("editor.project#1"),
                    true,
                ),
            ),
        ]),
        Vec::new(),
        Vec::new(),
    );
    let geometry = WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 50.0, 1440.0, 830.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (ShellRegionId::Left, ShellFrame::default()),
            (
                ShellRegionId::Document,
                ShellFrame::new(34.0, 140.0, 1086.0, 440.0),
            ),
            (ShellRegionId::Right, ShellFrame::default()),
            (
                ShellRegionId::Bottom,
                ShellFrame::new(96.0, 788.0, 640.0, 92.0),
            ),
        ]),
        splitter_frames: BTreeMap::new(),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::default(),
    };
    let root_projection = BuiltinHostRootShellFrames {
        shell_frame: Some(UiFrame::new(0.0, 0.0, 1440.0, 900.0)),
        host_body_frame: Some(UiFrame::new(0.0, 40.0, 1440.0, 840.0)),
        status_bar_frame: Some(UiFrame::new(0.0, 880.0, 1440.0, 20.0)),
        ..BuiltinHostRootShellFrames::default()
    };
    let mut bridge = HostShellPointerBridge::new();
    bridge.update_layout_with_root_shell_frames(
        UiSize::new(1440.0, 900.0),
        &geometry,
        true,
        &[],
        Some(&root_projection),
        None,
    );
    let pointer_x =
        6.0 + estimate_dock_tab_width("Console") + 4.0 + estimate_dock_tab_width("Project") * 0.25;
    let pointer_y = 792.0;
    let pointer_route = bridge.drag_route_at(UiPoint::new(pointer_x, pointer_y));

    assert_eq!(
        resolve_host_tab_drop_route_with_root_frames(
            &layout,
            &model,
            &geometry,
            &WorkbenchChromeMetrics::default(),
            "editor.hierarchy#1",
            pointer_route,
            "bottom",
            pointer_x,
            pointer_y,
            Some(&root_projection),
        ),
        Some(ResolvedHostTabDropRoute {
            target_group: HostDragTargetGroup::Bottom,
            target_label: "bottom tool stack",
            target: ResolvedHostTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Drawer(ActivityDrawerSlot::BottomRight),
                anchor: Some(TabInsertionAnchor {
                    target_id: ViewInstanceId::new("editor.project#1"),
                    side: TabInsertionSide::Before,
                }),
            }),
        })
    );
}
