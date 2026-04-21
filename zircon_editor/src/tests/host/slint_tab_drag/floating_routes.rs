use super::support::*;

#[test]
fn resolve_workbench_tab_drop_route_accepts_floating_window_group_fallback_key() {
    let floating_window_id = MainPageId::new("window:prefab");
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        floating_windows: vec![FloatingWindowLayout {
            window_id: floating_window_id.clone(),
            title: "Prefab Popout".to_string(),
            workspace: DocumentNode::SplitNode {
                axis: SplitAxis::Horizontal,
                ratio: 0.5,
                first: Box::new(DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("editor.scene#float")],
                    active_tab: Some(ViewInstanceId::new("editor.scene#float")),
                })),
                second: Box::new(DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("editor.prefab#float")],
                    active_tab: Some(ViewInstanceId::new("editor.prefab#float")),
                })),
            },
            focused_view: Some(ViewInstanceId::new("editor.prefab#float")),
            frame: ShellFrame::default(),
        }],
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        default_drawers_model(),
        vec![document_tab(
            "editor.scene#1",
            "Scene",
            vec![0],
            false,
            true,
        )],
        vec![floating_window(
            floating_window_id.clone(),
            "Prefab Popout",
            vec![
                floating_tab(
                    floating_window_id.clone(),
                    "editor.scene#float",
                    "Scene",
                    vec![0],
                    false,
                    false,
                ),
                floating_tab(
                    floating_window_id.clone(),
                    "editor.prefab#float",
                    "Prefab Editor",
                    vec![1],
                    true,
                    true,
                ),
            ],
            Some("editor.prefab#float"),
        )],
    );
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );

    assert_eq!(
        resolve_workbench_tab_drop_route(
            &layout,
            &model,
            &geometry,
            &WorkbenchChromeMetrics::default(),
            "editor.console#1",
            None,
            &floating_window_group_key(&floating_window_id),
            0.0,
            0.0,
        ),
        Some(ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Document,
            target_label: "floating window",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
                host: ViewHost::FloatingWindow(floating_window_id, vec![1]),
                anchor: None,
            }),
        })
    );
}

#[test]
fn resolved_workbench_tab_drop_route_snapshot_matches_shared_pointer_and_group_key_for_floating_window(
) {
    let floating_window_id = MainPageId::new("window:prefab");
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        floating_windows: vec![FloatingWindowLayout {
            window_id: floating_window_id.clone(),
            title: "Prefab Popout".to_string(),
            workspace: DocumentNode::SplitNode {
                axis: SplitAxis::Horizontal,
                ratio: 0.5,
                first: Box::new(DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("editor.scene#float")],
                    active_tab: Some(ViewInstanceId::new("editor.scene#float")),
                })),
                second: Box::new(DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("editor.prefab#float")],
                    active_tab: Some(ViewInstanceId::new("editor.prefab#float")),
                })),
            },
            focused_view: Some(ViewInstanceId::new("editor.prefab#float")),
            frame: ShellFrame::default(),
        }],
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        default_drawers_model(),
        vec![document_tab(
            "editor.scene#1",
            "Scene",
            vec![0],
            false,
            true,
        )],
        vec![floating_window(
            floating_window_id.clone(),
            "Prefab Popout",
            vec![
                floating_tab(
                    floating_window_id.clone(),
                    "editor.scene#float",
                    "Scene",
                    vec![0],
                    false,
                    false,
                ),
                floating_tab(
                    floating_window_id.clone(),
                    "editor.prefab#float",
                    "Prefab Editor",
                    vec![1],
                    true,
                    true,
                ),
            ],
            Some("editor.prefab#float"),
        )],
    );
    let mut geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    geometry.floating_window_frames.insert(
        floating_window_id.clone(),
        ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    );
    let floating_windows = vec![floating_window(
        floating_window_id.clone(),
        "Preview Popout",
        Vec::new(),
        None,
    )];
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(
        UiSize::new(1440.0, 900.0),
        &geometry,
        false,
        &floating_windows,
        &[],
    );

    let pointer_route = bridge.drag_route_at(UiPoint::new(600.0, 300.0));
    let from_pointer = resolve_workbench_tab_drop_route(
        &layout,
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        "editor.console#1",
        pointer_route,
        "document",
        600.0,
        300.0,
    )
    .expect("floating window route should resolve from shared pointer route");
    let from_group_key = resolve_workbench_tab_drop_route(
        &layout,
        &model,
        &geometry,
        &WorkbenchChromeMetrics::default(),
        "editor.console#1",
        None,
        &floating_window_group_key(&floating_window_id),
        600.0,
        300.0,
    )
    .expect("floating window route should resolve from fallback group key");

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
fn resolve_workbench_tab_drop_route_accepts_floating_window_edge_fallback_key() {
    let floating_window_id = MainPageId::new("window:prefab");
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        floating_windows: vec![FloatingWindowLayout {
            window_id: floating_window_id.clone(),
            title: "Prefab Popout".to_string(),
            workspace: DocumentNode::SplitNode {
                axis: SplitAxis::Horizontal,
                ratio: 0.5,
                first: Box::new(DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("editor.scene#float")],
                    active_tab: Some(ViewInstanceId::new("editor.scene#float")),
                })),
                second: Box::new(DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("editor.prefab#float")],
                    active_tab: Some(ViewInstanceId::new("editor.prefab#float")),
                })),
            },
            focused_view: Some(ViewInstanceId::new("editor.prefab#float")),
            frame: ShellFrame::default(),
        }],
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };
    let model = workbench_model(
        default_drawers_model(),
        vec![document_tab(
            "editor.scene#1",
            "Scene",
            vec![0],
            false,
            true,
        )],
        vec![floating_window(
            floating_window_id.clone(),
            "Prefab Popout",
            vec![
                floating_tab(
                    floating_window_id.clone(),
                    "editor.scene#float",
                    "Scene",
                    vec![0],
                    false,
                    false,
                ),
                floating_tab(
                    floating_window_id.clone(),
                    "editor.prefab#float",
                    "Prefab Editor",
                    vec![1],
                    true,
                    true,
                ),
            ],
            Some("editor.prefab#float"),
        )],
    );
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );

    assert_eq!(
        resolve_workbench_tab_drop_route(
            &layout,
            &model,
            &geometry,
            &WorkbenchChromeMetrics::default(),
            "editor.console#1",
            None,
            &floating_window_edge_group_key(&floating_window_id, DockEdge::Right),
            0.0,
            0.0,
        ),
        Some(ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Document,
            target_label: "Split Floating Window Right",
            target: ResolvedWorkbenchTabDropTarget::Split {
                workspace: WorkspaceTarget::FloatingWindow(floating_window_id),
                path: vec![1],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::After,
            },
        })
    );
}
