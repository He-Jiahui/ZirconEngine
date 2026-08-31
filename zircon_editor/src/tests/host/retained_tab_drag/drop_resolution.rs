use super::support::*;

#[test]
fn drop_host_for_left_group_prefers_active_visible_left_slot() {
    let layout = workbench_layout(
        MainPageId::workbench(),
        vec![workbench_page(MainPageId::workbench())],
        BTreeMap::from([
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
        Vec::new(),
    );

    assert_eq!(
        drop_host_for_group(&layout, "left"),
        Some(ViewHost::Drawer(ActivityDrawerSlot::LeftBottom))
    );
}

#[test]
fn drop_host_for_document_group_uses_active_workbench_page() {
    let active_page = MainPageId::new("workbench-b");
    let layout = workbench_layout(
        active_page.clone(),
        vec![
            workbench_page(MainPageId::new("workbench-a")),
            workbench_page(active_page.clone()),
        ],
        default_drawers(),
        Vec::new(),
    );

    assert_eq!(
        drop_host_for_group(&layout, "document"),
        Some(ViewHost::Document(active_page, Vec::new()))
    );
}

#[test]
fn drop_host_for_document_group_falls_back_to_first_workbench_page() {
    let fallback_page = MainPageId::new("workbench-a");
    let layout = workbench_layout(
        MainPageId::new("page:editor.asset_browser#1"),
        vec![
            MainHostPageLayout::ExclusiveActivityWindowPage {
                id: MainPageId::new("page:editor.asset_browser#1"),
                title: "Asset Browser".to_string(),
                window_instance: ViewInstanceId::new("editor.asset_browser#1"),
            },
            workbench_page(fallback_page.clone()),
        ],
        default_drawers(),
        Vec::new(),
    );

    assert_eq!(
        drop_host_for_group(&layout, "document"),
        Some(ViewHost::Document(fallback_page, Vec::new()))
    );
}

#[test]
fn drop_host_for_unknown_group_returns_none() {
    let layout = workbench_layout(
        MainPageId::workbench(),
        vec![workbench_page(MainPageId::workbench())],
        default_drawers(),
        Vec::new(),
    );

    assert_eq!(drop_host_for_group(&layout, "mystery"), None);
}

#[test]
fn drop_host_for_tab_keeps_current_right_bottom_slot_when_dropping_within_same_group() {
    let layout = workbench_layout(
        MainPageId::workbench(),
        vec![workbench_page(MainPageId::workbench())],
        BTreeMap::from([
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
        Vec::new(),
    );

    assert_eq!(
        drop_host_for_tab(&layout, "editor.project#1", "right"),
        Some(ViewHost::Drawer(ActivityDrawerSlot::RightBottom))
    );
}

#[test]
fn drop_host_for_tab_keeps_current_document_stack_when_dropping_within_document_group() {
    let mut layout = workbench_layout(
        MainPageId::workbench(),
        vec![MainHostPageLayout::WorkbenchPage {
            id: MainPageId::workbench(),
            title: "Workbench".to_string(),
            activity_window: ActivityWindowId::workbench(),
        }],
        default_drawers(),
        Vec::new(),
    );
    *layout
        .content_workspace_for_page_mut(&MainPageId::workbench())
        .expect("workbench page should resolve its activity-window content workspace") =
        DocumentNode::SplitNode {
            axis: SplitAxis::Horizontal,
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
        };

    assert_eq!(
        drop_host_for_tab(&layout, "editor.game#1", "document"),
        Some(ViewHost::Document(MainPageId::workbench(), vec![1]))
    );
}

#[test]
fn resolve_tab_drop_targets_specific_right_tab_slot_and_inserts_before_it() {
    let layout = workbench_layout(
        MainPageId::workbench(),
        vec![workbench_page(MainPageId::workbench())],
        BTreeMap::from([
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
        Vec::new(),
    );
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
        resolve_tab_drop_with_workbench_layout_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.hierarchy#1",
            "right",
            pointer_x,
            pointer_y,
            workbench_layout_frames_from_geometry_with_drawers(&geometry, &[ShellRegionId::Right],),
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
    let layout = workbench_layout(
        MainPageId::workbench(),
        vec![workbench_page(MainPageId::workbench())],
        default_drawers(),
        Vec::new(),
    );
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
        resolve_tab_drop_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.asset-browser#1",
            "document",
            pointer_x,
            pointer_y,
            Some(&root_frames_from_geometry(&geometry)),
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
fn resolve_tab_drop_uses_the_previous_document_tab_for_a_strip_gap() {
    let layout = workbench_layout(
        MainPageId::workbench(),
        vec![workbench_page(MainPageId::workbench())],
        default_drawers(),
        Vec::new(),
    );
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
    let pointer_x = 34.0 + 8.0 + estimate_document_tab_width("Scene", false) + 1.0;

    assert_eq!(
        resolve_tab_drop_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.asset-browser#1",
            "document",
            pointer_x,
            54.0,
            Some(&root_frames_from_geometry(&geometry)),
        ),
        Some(ResolvedTabDrop {
            host: ViewHost::Document(MainPageId::workbench(), vec![0]),
            anchor: Some(TabInsertionAnchor {
                target_id: ViewInstanceId::new("editor.scene#1"),
                side: TabInsertionSide::After,
            }),
        })
    );
}

#[test]
fn resolve_tab_drop_after_the_document_strip_uses_the_last_tab() {
    let layout = workbench_layout(
        MainPageId::workbench(),
        vec![workbench_page(MainPageId::workbench())],
        default_drawers(),
        Vec::new(),
    );
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
        resolve_tab_drop_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.asset-browser#1",
            "document",
            1000.0,
            54.0,
            Some(&root_frames_from_geometry(&geometry)),
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
fn resolve_tab_drop_returns_none_when_only_the_dragged_tab_remains() {
    let layout = workbench_layout(
        MainPageId::workbench(),
        vec![workbench_page(MainPageId::workbench())],
        default_drawers(),
        Vec::new(),
    );
    let model = workbench_model(
        default_drawers_model(),
        vec![document_tab(
            "editor.scene#1",
            "Scene",
            vec![0],
            false,
            true,
        )],
        Vec::new(),
    );
    let geometry = shell_geometry(
        ShellFrame::new(1120.0, 50.0, 320.0, 738.0),
        ShellFrame::new(34.0, 50.0, 1086.0, 738.0),
        ShellFrame::new(0.0, 788.0, 1440.0, 92.0),
    );

    assert_eq!(
        resolve_tab_drop_with_root_frames(
            &layout,
            &model,
            &WorkbenchChromeMetrics::default(),
            "editor.scene#1",
            "document",
            60.0,
            54.0,
            Some(&root_frames_from_geometry(&geometry)),
        ),
        None
    );
}
