use std::collections::BTreeMap;

use crate::host::slint_host::shell_pointer::WorkbenchShellPointerRoute;
use crate::host::slint_host::tab_drag::{
    document_edge_group_key, drop_host_for_group, drop_host_for_tab, estimate_dock_tab_width,
    estimate_document_tab_width, floating_window_edge_group_key, floating_window_group_key,
    resolve_tab_drop, resolve_workbench_drag_target_group, resolve_workbench_tab_drop_route,
    workbench_shell_pointer_route_group_key, ResolvedTabDrop, ResolvedWorkbenchTabDropRoute,
    ResolvedWorkbenchTabDropTarget, WorkbenchDragTargetGroup,
};
use crate::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DockEdge, DocumentNode,
    DocumentTabModel, DocumentWorkspaceModel, DrawerRingModel, EditorUiCompatibilityHarness,
    FloatingWindowModel, MainHostPageLayout, MainHostStripModel, MainHostStripViewModel,
    MainPageId, MenuBarModel, PaneTabModel, ShellFrame, ShellRegionId, SplitAxis, SplitPlacement,
    StatusBarModel, TabInsertionAnchor, TabInsertionSide, TabStackLayout, ToolWindowStackModel,
    ViewContentKind, ViewDescriptorId, ViewHost, ViewInstanceId, WorkbenchLayout,
    WorkbenchShellGeometry, WorkbenchViewModel, WorkspaceTarget,
};
use zircon_ui::{UiPoint, UiSize};

use crate::host::slint_host::shell_pointer::WorkbenchShellPointerBridge;

#[test]
fn drop_host_for_left_group_prefers_active_visible_left_slot() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: BTreeMap::from([
            (
                ActivityDrawerSlot::LeftTop,
                drawer(
                    ActivityDrawerSlot::LeftTop,
                    &[],
                    None,
                    ActivityDrawerMode::Collapsed,
                    true,
                ),
            ),
            (
                ActivityDrawerSlot::LeftBottom,
                drawer(
                    ActivityDrawerSlot::LeftBottom,
                    &["editor.project#1"],
                    Some("editor.project#1"),
                    ActivityDrawerMode::Pinned,
                    true,
                ),
            ),
        ]),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    assert_eq!(
        drop_host_for_group(&layout, "left"),
        Some(ViewHost::Drawer(ActivityDrawerSlot::LeftBottom))
    );
}

#[test]
fn drop_host_for_document_group_uses_active_workbench_page() {
    let active_page = MainPageId::new("workbench-b");
    let layout = WorkbenchLayout {
        active_main_page: active_page.clone(),
        main_pages: vec![
            workbench_page(MainPageId::new("workbench-a")),
            workbench_page(active_page.clone()),
        ],
        drawers: default_drawers(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    assert_eq!(
        drop_host_for_group(&layout, "document"),
        Some(ViewHost::Document(active_page, Vec::new()))
    );
}

#[test]
fn drop_host_for_document_group_falls_back_to_first_workbench_page() {
    let fallback_page = MainPageId::new("workbench-a");
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::new("page:editor.asset_browser#1"),
        main_pages: vec![
            MainHostPageLayout::ExclusiveActivityWindowPage {
                id: MainPageId::new("page:editor.asset_browser#1"),
                title: "Asset Browser".to_string(),
                window_instance: ViewInstanceId::new("editor.asset_browser#1"),
            },
            workbench_page(fallback_page.clone()),
        ],
        drawers: default_drawers(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    assert_eq!(
        drop_host_for_group(&layout, "document"),
        Some(ViewHost::Document(fallback_page, Vec::new()))
    );
}

#[test]
fn drop_host_for_unknown_group_returns_none() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    assert_eq!(drop_host_for_group(&layout, "mystery"), None);
}

#[test]
fn drop_host_for_tab_keeps_current_right_bottom_slot_when_dropping_within_same_group() {
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
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    assert_eq!(
        drop_host_for_tab(&layout, "editor.project#1", "right"),
        Some(ViewHost::Drawer(ActivityDrawerSlot::RightBottom))
    );
}

#[test]
fn drop_host_for_tab_keeps_current_document_stack_when_dropping_within_document_group() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![MainHostPageLayout::WorkbenchPage {
            id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            document_workspace: DocumentNode::SplitNode {
                axis: crate::SplitAxis::Horizontal,
                ratio: 0.5,
                first: Box::new(DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![ViewInstanceId::new("editor.scene#1")],
                    active_tab: Some(ViewInstanceId::new("editor.scene#1")),
                })),
                second: Box::new(DocumentNode::Tabs(TabStackLayout {
                    tabs: vec![
                        ViewInstanceId::new("editor.game#1"),
                        ViewInstanceId::new("editor.prefab#1"),
                    ],
                    active_tab: Some(ViewInstanceId::new("editor.prefab#1")),
                })),
            },
        }],
        drawers: default_drawers(),
        floating_windows: Vec::new(),
        region_overrides: BTreeMap::new(),
        view_overrides: BTreeMap::new(),
    };

    assert_eq!(
        drop_host_for_tab(&layout, "editor.game#1", "document"),
        Some(ViewHost::Document(MainPageId::workbench(), vec![1]))
    );
}

#[test]
fn resolve_tab_drop_targets_specific_right_tab_slot_and_inserts_before_it() {
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
        resolve_tab_drop(
            &layout,
            &model,
            &geometry,
            &crate::WorkbenchChromeMetrics::default(),
            "editor.hierarchy#1",
            "right",
            pointer_x,
            pointer_y,
        ),
        Some(ResolvedTabDrop {
            host: ViewHost::Drawer(ActivityDrawerSlot::RightBottom),
            anchor: Some(TabInsertionAnchor {
                target_id: ViewInstanceId::new("editor.project#1"),
                side: TabInsertionSide::Before,
            }),
        })
    );
}

#[test]
fn resolve_tab_drop_targets_specific_document_stack_and_inserts_after_it() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
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
        resolve_tab_drop(
            &layout,
            &model,
            &geometry,
            &crate::WorkbenchChromeMetrics::default(),
            "editor.asset-browser#1",
            "document",
            pointer_x,
            pointer_y,
        ),
        Some(ResolvedTabDrop {
            host: ViewHost::Document(MainPageId::workbench(), vec![1]),
            anchor: Some(TabInsertionAnchor {
                target_id: ViewInstanceId::new("editor.game#1"),
                side: TabInsertionSide::After,
            }),
        })
    );
}

#[test]
fn resolve_workbench_tab_drop_route_prefers_shared_pointer_route_over_stale_host_group() {
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
        resolve_workbench_tab_drop_route(
            &layout,
            &model,
            &geometry,
            &crate::WorkbenchChromeMetrics::default(),
            "editor.hierarchy#1",
            Some(WorkbenchShellPointerRoute::DragTarget(
                WorkbenchDragTargetGroup::Right,
            )),
            "document",
            pointer_x,
            pointer_y,
        ),
        Some(ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Right,
            target_label: "right tool stack",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
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
fn resolve_workbench_tab_drop_route_falls_back_to_host_group_when_pointer_route_is_missing() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
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
        resolve_workbench_tab_drop_route(
            &layout,
            &model,
            &geometry,
            &crate::WorkbenchChromeMetrics::default(),
            "editor.asset-browser#1",
            None,
            "document",
            pointer_x,
            pointer_y,
        ),
        Some(ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Document,
            target_label: "document workspace",
            target: ResolvedWorkbenchTabDropTarget::Attach(ResolvedTabDrop {
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
fn resolve_workbench_tab_drop_route_maps_document_edge_to_create_split_on_active_workspace_path() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
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
        resolve_workbench_tab_drop_route(
            &layout,
            &model,
            &geometry,
            &crate::WorkbenchChromeMetrics::default(),
            "editor.assets#1",
            Some(WorkbenchShellPointerRoute::DocumentEdge(DockEdge::Left)),
            "document",
            48.0,
            240.0,
        ),
        Some(ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Document,
            target_label: "Split Document Left",
            target: ResolvedWorkbenchTabDropTarget::Split {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![1],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::Before,
            },
        })
    );
}

#[test]
fn resolved_workbench_tab_drop_route_snapshot_matches_shared_pointer_and_group_key_for_document_edge(
) {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
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
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(UiSize::new(1440.0, 900.0), &geometry, false, &[]);

    let pointer_route = bridge.drag_route_at(UiPoint::new(12.0, 240.0));
    let from_pointer = resolve_workbench_tab_drop_route(
        &layout,
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        "editor.assets#1",
        pointer_route,
        "document",
        12.0,
        240.0,
    )
    .expect("document edge route should resolve from shared pointer route");
    let from_group_key = resolve_workbench_tab_drop_route(
        &layout,
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
        "editor.assets#1",
        None,
        document_edge_group_key(DockEdge::Left),
        12.0,
        240.0,
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
fn resolve_workbench_tab_drop_route_accepts_document_edge_group_fallback_keys() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
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
        resolve_workbench_tab_drop_route(
            &layout,
            &model,
            &geometry,
            &crate::WorkbenchChromeMetrics::default(),
            "editor.assets#1",
            None,
            "document-right",
            1088.0,
            240.0,
        ),
        Some(ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Document,
            target_label: "Split Document Right",
            target: ResolvedWorkbenchTabDropTarget::Split {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![0],
                axis: SplitAxis::Horizontal,
                placement: SplitPlacement::After,
            },
        })
    );
}

#[test]
fn resolve_workbench_tab_drop_route_prefers_document_edge_fallback_over_coarse_document_route() {
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
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
        resolve_workbench_tab_drop_route(
            &layout,
            &model,
            &geometry,
            &crate::WorkbenchChromeMetrics::default(),
            "editor.assets#1",
            Some(WorkbenchShellPointerRoute::DragTarget(
                WorkbenchDragTargetGroup::Document,
            )),
            "document-top",
            640.0,
            60.0,
        ),
        Some(ResolvedWorkbenchTabDropRoute {
            target_group: WorkbenchDragTargetGroup::Document,
            target_label: "Split Document Top",
            target: ResolvedWorkbenchTabDropTarget::Split {
                workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
                path: vec![1],
                axis: SplitAxis::Vertical,
                placement: SplitPlacement::Before,
            },
        })
    );
}

#[test]
fn resolve_workbench_tab_drop_route_accepts_floating_window_group_fallback_key() {
    let floating_window_id = MainPageId::new("window:prefab");
    let layout = WorkbenchLayout {
        active_main_page: MainPageId::workbench(),
        main_pages: vec![workbench_page(MainPageId::workbench())],
        drawers: default_drawers(),
        floating_windows: vec![crate::FloatingWindowLayout {
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
            &crate::WorkbenchChromeMetrics::default(),
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
        floating_windows: vec![crate::FloatingWindowLayout {
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
    );

    let pointer_route = bridge.drag_route_at(UiPoint::new(600.0, 300.0));
    let from_pointer = resolve_workbench_tab_drop_route(
        &layout,
        &model,
        &geometry,
        &crate::WorkbenchChromeMetrics::default(),
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
        &crate::WorkbenchChromeMetrics::default(),
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
        floating_windows: vec![crate::FloatingWindowLayout {
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
            &crate::WorkbenchChromeMetrics::default(),
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
    let mut bridge = WorkbenchShellPointerBridge::new();
    bridge.update_layout_with_floating_windows(UiSize::new(1440.0, 900.0), &geometry, false, &[]);

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(12.0, 240.0)),
        Some(WorkbenchShellPointerRoute::DocumentEdge(DockEdge::Left))
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(12.0, 240.0)),
        Some(WorkbenchDragTargetGroup::Document)
    );
}

#[test]
fn workbench_shell_pointer_route_group_key_normalizes_document_and_floating_routes() {
    let window_id = MainPageId::new("window:preview");

    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::DragTarget(
            WorkbenchDragTargetGroup::Right,
        )),
        Some("right".to_string())
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::DocumentEdge(
            DockEdge::Bottom,
        )),
        Some("document-bottom".to_string())
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindow(
            window_id.clone(),
        )),
        Some(floating_window_group_key(&window_id))
    );
    assert_eq!(
        workbench_shell_pointer_route_group_key(&WorkbenchShellPointerRoute::FloatingWindowEdge {
            window_id: window_id.clone(),
            edge: DockEdge::Left,
        }),
        Some(floating_window_edge_group_key(&window_id, DockEdge::Left))
    );
}

#[test]
fn shared_shell_pointer_route_reports_floating_window_attach_from_shared_surface() {
    let window_id = MainPageId::new("window:preview");
    let mut geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    );
    let floating_windows = vec![floating_window(
        window_id.clone(),
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
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(600.0, 300.0)),
        Some(WorkbenchShellPointerRoute::FloatingWindow(window_id))
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(600.0, 300.0)),
        Some(WorkbenchDragTargetGroup::Document)
    );
}

#[test]
fn shared_shell_pointer_route_reports_floating_window_edge_from_shared_surface() {
    let window_id = MainPageId::new("window:preview");
    let mut geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );
    geometry.floating_window_frames.insert(
        window_id.clone(),
        ShellFrame::new(420.0, 180.0, 360.0, 240.0),
    );
    let floating_windows = vec![floating_window(
        window_id.clone(),
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
    );

    assert_eq!(
        bridge.drag_route_at(UiPoint::new(426.0, 300.0)),
        Some(WorkbenchShellPointerRoute::FloatingWindowEdge {
            window_id,
            edge: DockEdge::Left,
        })
    );
    assert_eq!(
        bridge.drag_target_at(UiPoint::new(426.0, 300.0)),
        Some(WorkbenchDragTargetGroup::Document)
    );
}

#[test]
fn shared_drag_target_route_prefers_right_over_bottom_in_overlap_when_pointer_is_closer_to_right_edge(
) {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            UiSize::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(1428.0, 860.0),
        ),
        Some(WorkbenchDragTargetGroup::Right)
    );
}

#[test]
fn shared_drag_target_route_prefers_bottom_over_right_in_overlap_when_pointer_is_closer_to_bottom_edge(
) {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            UiSize::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(1380.0, 860.0),
        ),
        Some(WorkbenchDragTargetGroup::Bottom)
    );
}

#[test]
fn shared_drag_target_route_returns_document_inside_document_region() {
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            UiSize::new(1440.0, 900.0),
            &geometry,
            true,
            UiPoint::new(720.0, 240.0),
        ),
        Some(WorkbenchDragTargetGroup::Document)
    );
}

#[test]
fn shared_drag_target_route_disables_empty_tool_regions_when_drawers_are_hidden() {
    let geometry = shell_geometry(
        ShellFrame::new(1348.0, 50.0, 0.0, 666.0),
        ShellFrame::new(34.0, 50.0, 1314.0, 666.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 0.0),
    );

    assert_eq!(
        resolve_workbench_drag_target_group(
            UiSize::new(1440.0, 900.0),
            &geometry,
            false,
            UiPoint::new(12.0, 240.0),
        ),
        None
    );
}

#[test]
fn shared_drag_capture_surface_replaces_legacy_direct_drop_callback_abi() {
    let workbench = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/ui/workbench.slint"));
    let wiring = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/callback_wiring.rs"
    ));
    let docking = include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/host/slint_host/app/workspace_docking.rs"
    ));

    for needle in [
        "callback drop_tab(tab_id: string, target_group: string, pointer_x: float, pointer_y: float);",
        "callback update_drag_target(x: float, y: float);",
        "root.update_drag_target(root.drag_pointer_x, root.drag_pointer_y);",
        "root.drop_tab(",
        "ui.on_drop_tab(",
        "ui.on_update_drag_target(",
        "fn drop_tab(",
        "fn update_drag_target(",
    ] {
        let found =
            workbench.contains(needle) || wiring.contains(needle) || docking.contains(needle);
        assert!(
            !found,
            "drag capture path still exposes legacy direct callback `{needle}`"
        );
    }

    for needle in [
        "callback workbench_drag_pointer_event(kind: int, x: float, y: float);",
        "root.workbench_drag_pointer_event(",
    ] {
        assert!(
            workbench.contains(needle),
            "workbench shell is missing shared drag pointer hook `{needle}`"
        );
    }

    assert!(
        wiring.contains("ui.on_workbench_drag_pointer_event("),
        "slint host callback wiring must register shared drag pointer callback"
    );
    assert!(
        docking.contains("fn workbench_drag_pointer_event("),
        "workspace docking host must handle shared drag pointer events"
    );
}

fn workbench_page(id: MainPageId) -> MainHostPageLayout {
    MainHostPageLayout::WorkbenchPage {
        id,
        title: "Workbench".to_string(),
        document_workspace: DocumentNode::default(),
    }
}

fn drawer(
    slot: ActivityDrawerSlot,
    tabs: &[&str],
    active_tab: Option<&str>,
    mode: ActivityDrawerMode,
    visible: bool,
) -> ActivityDrawerLayout {
    ActivityDrawerLayout {
        slot,
        tab_stack: TabStackLayout {
            tabs: tabs.iter().map(|tab| ViewInstanceId::new(*tab)).collect(),
            active_tab: active_tab.map(ViewInstanceId::new),
        },
        active_view: active_tab.map(ViewInstanceId::new),
        mode,
        extent: 260.0,
        visible,
    }
}

fn default_drawers() -> BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout> {
    ActivityDrawerSlot::ALL
        .into_iter()
        .map(|slot| {
            (
                slot,
                drawer(slot, &[], None, ActivityDrawerMode::Collapsed, true),
            )
        })
        .collect()
}

fn default_drawers_model() -> BTreeMap<ActivityDrawerSlot, ToolWindowStackModel> {
    ActivityDrawerSlot::ALL
        .into_iter()
        .map(|slot| {
            (
                slot,
                ToolWindowStackModel {
                    slot,
                    mode: ActivityDrawerMode::Collapsed,
                    visible: true,
                    tabs: Vec::new(),
                    active_tab: None,
                },
            )
        })
        .collect()
}

fn pane_tab(id: &str, title: &str, active: bool) -> PaneTabModel {
    PaneTabModel {
        instance_id: ViewInstanceId::new(id),
        descriptor_id: ViewDescriptorId::new(id),
        title: title.to_string(),
        icon_key: "tool".to_string(),
        content_kind: ViewContentKind::Project,
        active,
        closeable: false,
        empty_state: None,
    }
}

fn tool_window_stack(
    slot: ActivityDrawerSlot,
    tabs: &[PaneTabModel],
    active_tab: Option<&str>,
    visible: bool,
) -> ToolWindowStackModel {
    ToolWindowStackModel {
        slot,
        mode: ActivityDrawerMode::Pinned,
        visible,
        tabs: tabs.to_vec(),
        active_tab: active_tab.map(ViewInstanceId::new),
    }
}

fn document_tab(
    id: &str,
    title: &str,
    workspace_path: Vec<usize>,
    closeable: bool,
    active: bool,
) -> DocumentTabModel {
    DocumentTabModel {
        workspace: WorkspaceTarget::MainPage(MainPageId::workbench()),
        workspace_path,
        instance_id: ViewInstanceId::new(id),
        descriptor_id: ViewDescriptorId::new(id),
        title: title.to_string(),
        icon_key: "tool".to_string(),
        content_kind: ViewContentKind::Scene,
        active,
        closeable,
        empty_state: None,
    }
}

fn workbench_model(
    tool_windows: BTreeMap<ActivityDrawerSlot, ToolWindowStackModel>,
    document_tabs: Vec<DocumentTabModel>,
    floating_windows: Vec<FloatingWindowModel>,
) -> WorkbenchViewModel {
    WorkbenchViewModel {
        menu_bar: MenuBarModel { menus: Vec::new() },
        host_strip: MainHostStripViewModel {
            mode: MainHostStripModel::Workbench,
            pages: Vec::new(),
            active_page: MainPageId::workbench(),
            breadcrumbs: Vec::new(),
        },
        drawer_ring: DrawerRingModel {
            visible: true,
            drawers: BTreeMap::new(),
        },
        tool_windows,
        document_tabs,
        floating_windows,
        document: DocumentWorkspaceModel::Workbench {
            page_id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            workspace: crate::DocumentWorkspaceSnapshot::Tabs {
                tabs: Vec::new(),
                active_tab: None,
            },
        },
        status_bar: StatusBarModel {
            primary_text: String::new(),
            secondary_text: None,
            viewport_label: String::new(),
        },
    }
}

fn floating_window(
    window_id: MainPageId,
    title: &str,
    tabs: Vec<DocumentTabModel>,
    focused_view: Option<&str>,
) -> FloatingWindowModel {
    FloatingWindowModel {
        window_id,
        title: title.to_string(),
        focused_view: focused_view.map(ViewInstanceId::new),
        tabs,
    }
}

fn floating_tab(
    window_id: MainPageId,
    id: &str,
    title: &str,
    workspace_path: Vec<usize>,
    closeable: bool,
    active: bool,
) -> DocumentTabModel {
    DocumentTabModel {
        workspace: WorkspaceTarget::FloatingWindow(window_id),
        workspace_path,
        instance_id: ViewInstanceId::new(id),
        descriptor_id: ViewDescriptorId::new(id),
        title: title.to_string(),
        icon_key: "tool".to_string(),
        content_kind: ViewContentKind::Scene,
        active,
        closeable,
        empty_state: None,
    }
}

fn shell_geometry(
    right_region: ShellFrame,
    document_region: ShellFrame,
    bottom_region: ShellFrame,
) -> WorkbenchShellGeometry {
    WorkbenchShellGeometry {
        window_min_width: 0.0,
        window_min_height: 0.0,
        center_band_frame: ShellFrame::new(0.0, 50.0, 1440.0, 830.0),
        status_bar_frame: ShellFrame::new(0.0, 880.0, 1440.0, 20.0),
        region_frames: BTreeMap::from([
            (
                ShellRegionId::Left,
                ShellFrame::new(0.0, 50.0, 320.0, 738.0),
            ),
            (ShellRegionId::Document, document_region),
            (ShellRegionId::Right, right_region),
            (ShellRegionId::Bottom, bottom_region),
        ]),
        splitter_frames: BTreeMap::new(),
        floating_window_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(0.0, 0.0, 0.0, 0.0),
    }
}
