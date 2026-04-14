use std::collections::BTreeMap;

use crate::host::slint_host::tab_drag::{
    drop_host_for_group, drop_host_for_tab, estimate_dock_tab_width,
    estimate_document_tab_width, resolve_tab_drop, ResolvedTabDrop,
};
use crate::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, DocumentNode,
    DocumentTabModel, DocumentWorkspaceModel, DrawerRingModel, MainHostPageLayout,
    MainHostStripModel, MainHostStripViewModel, MainPageId, MenuBarModel, PaneTabModel,
    ShellFrame, ShellRegionId, StatusBarModel, TabInsertionAnchor, TabInsertionSide,
    TabStackLayout, ToolWindowStackModel, ViewContentKind, ViewDescriptorId, ViewHost,
    ViewInstanceId, WorkbenchLayout, WorkbenchShellGeometry, WorkbenchViewModel,
};

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
        page_id: MainPageId::workbench(),
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
            (ShellRegionId::Left, ShellFrame::new(0.0, 50.0, 320.0, 738.0)),
            (ShellRegionId::Document, document_region),
            (ShellRegionId::Right, right_region),
            (ShellRegionId::Bottom, bottom_region),
        ]),
        splitter_frames: BTreeMap::new(),
        viewport_content_frame: ShellFrame::new(0.0, 0.0, 0.0, 0.0),
    }
}
