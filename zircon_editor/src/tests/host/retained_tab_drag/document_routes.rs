use super::support::*;

#[test]
fn resolve_host_tab_drop_route_prefers_shared_pointer_route_over_stale_host_group() {
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
    let pointer_x = 1120.0
        + 6.0
        + estimate_dock_tab_width("Inspector")
        + 4.0
        + estimate_dock_tab_width("Project") * 0.25;
    let pointer_y = 54.0;

    assert_eq!(
        resolve_host_tab_drop_route_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.hierarchy#1",
            Some(HostShellPointerRoute::DragTarget(
                HostDragTargetGroup::Right,
            )),
            "document",
            pointer_x,
            pointer_y,
            Some(&root_frames_from_geometry_with_drawers(
                &geometry,
                &[ShellRegionId::Right],
            )),
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
fn resolve_host_tab_drop_route_falls_back_to_host_group_when_pointer_route_is_missing() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        default_drawers_model(),
        vec![
            document_tab("editor.scene#1", "Scene", vec![0], false, true),
            document_tab("editor.game#1", "Game", vec![1], false, false),
            document_tab("editor.prefab#1", "Enemy.prefab", vec![1], true, false),
        ],
        Vec::new(),
    );
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    let pointer_x = 34.0
        + 8.0
        + estimate_document_tab_width("Scene", false)
        + 2.0
        + estimate_document_tab_width("Game", false) * 0.75;
    let pointer_y = 54.0;

    assert_eq!(
        resolve_host_tab_drop_route_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.asset-browser#1",
            None,
            "document",
            pointer_x,
            pointer_y,
            Some(&root_frames_from_geometry(&geometry)),
        ),
        Some(ResolvedHostTabDropRoute {
            target_group: HostDragTargetGroup::Document,
            target_label: "document workspace",
            target: ResolvedHostTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::Document(MainPageId::workbench(), vec![1]),
                anchor: Some(TabInsertionAnchor {
                    target_id: ViewInstanceId::new("editor.game#1"),
                    side: TabInsertionSide::After,
                }),
            }),
        })
    );
}

#[test]
fn resolve_host_tab_drop_route_maps_document_edge_to_create_split_on_active_workspace_path() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        default_drawers_model(),
        vec![
            document_tab("editor.scene#1", "Scene", vec![0], false, false),
            document_tab("editor.game#1", "Game", vec![1], false, true),
            document_tab("editor.prefab#1", "Enemy.prefab", vec![1], true, false),
        ],
        Vec::new(),
    );
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );

    assert_eq!(
        resolve_host_tab_drop_route_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.assets#1",
            Some(HostShellPointerRoute::DocumentEdge(DockEdge::Left)),
            "document",
            48.0,
            240.0,
            Some(&root_frames_from_geometry(&geometry)),
        ),
        Some(ResolvedHostTabDropRoute {
            target_group: HostDragTargetGroup::Document,
            target_label: "Split Document Left",
            target: ResolvedHostTabDropTarget::Split {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![1],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::Before,
            },
        })
    );
}

#[test]
fn resolved_host_tab_drop_route_snapshot_matches_shared_pointer_and_group_key_for_document_edge() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        default_drawers_model(),
        vec![
            document_tab("editor.scene#1", "Scene", vec![0], false, false),
            document_tab("editor.game#1", "Game", vec![1], false, true),
            document_tab("editor.prefab#1", "Enemy.prefab", vec![1], true, false),
        ],
        Vec::new(),
    );
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
    let root_frames = root_frames_from_geometry(&geometry);
    bridge.update_layout_with_root_shell_frames(
        UiSize::new(1440.0, 900.0),
        false,
        &[],
        Some(&root_frames),
        None,
    );

    let pointer_route = bridge.drag_route_at(UiPoint::new(12.0, 240.0));
    let from_pointer = resolve_host_tab_drop_route_with_root_frames(
        &layout,
        &model,
        &WorkbenchChromeMetrics::default(),
        "editor.assets#1",
        pointer_route,
        "document",
        12.0,
        240.0,
        Some(&root_frames),
    )
    .expect("document edge route should resolve from shared pointer route");
    let from_group_key = resolve_host_tab_drop_route_with_root_frames(
        &layout,
        &model,
        &WorkbenchChromeMetrics::default(),
        "editor.assets#1",
        None,
        document_edge_group_key(DockEdge::Left),
        12.0,
        240.0,
        Some(&root_frames),
    )
    .expect("document edge route should resolve from fallback group key");

    let pointer_snapshot =
        EditorUiCompatibilityHarness::capture_resolved_tab_drop_route_snapshot(&from_pointer);
    let group_key_snapshot =
        EditorUiCompatibilityHarness::capture_resolved_tab_drop_route_snapshot(&from_group_key);

    assert_eq!(
        pointer_snapshot.route_result_entries,
        group_key_snapshot.route_result_entries
    );
}

#[test]
fn resolve_host_tab_drop_route_accepts_document_edge_group_fallback_keys() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        default_drawers_model(),
        vec![
            document_tab("editor.scene#1", "Scene", vec![0], false, true),
            document_tab("editor.game#1", "Game", vec![1], false, false),
        ],
        Vec::new(),
    );
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );

    assert_eq!(
        resolve_host_tab_drop_route_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.assets#1",
            None,
            "document-right",
            1088.0,
            240.0,
            Some(&root_frames_from_geometry(&geometry)),
        ),
        Some(ResolvedHostTabDropRoute {
            target_group: HostDragTargetGroup::Document,
            target_label: "Split Document Right",
            target: ResolvedHostTabDropTarget::Split {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![0],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::After,
            },
        })
    );
}

#[test]
fn resolve_host_tab_drop_route_prefers_document_edge_fallback_over_coarse_document_route() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        activity_windows: Default::default(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        default_drawers_model(),
        vec![
            document_tab("editor.scene#1", "Scene", vec![0], false, false),
            document_tab("editor.game#1", "Game", vec![1], false, true),
        ],
        Vec::new(),
    );
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );

    assert_eq!(
        resolve_host_tab_drop_route_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.assets#1",
            Some(HostShellPointerRoute::DragTarget(
                HostDragTargetGroup::Document,
            )),
            "document-top",
            640.0,
            60.0,
            Some(&root_frames_from_geometry(&geometry)),
        ),
        Some(ResolvedHostTabDropRoute {
            target_group: HostDragTargetGroup::Document,
            target_label: "Split Document Top",
            target: ResolvedHostTabDropTarget::Split {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![1],
                axis: SplitAxis::Vertical,
                placement: SplitPlacement::Before,
            },
        })
    );
}
