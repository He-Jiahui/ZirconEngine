pub(super) use std::collections::BTreeMap;

pub(super) use crate::ui::host::NativeWindowHostState;
pub(super) use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
pub(super) use crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
pub(super) use crate::ui::slint_host::shell_pointer::{
    HostShellPointerBridge, HostShellPointerRoute,
};
pub(super) use crate::ui::slint_host::tab_drag::{
    document_edge_group_key, drop_host_for_group, drop_host_for_tab, estimate_dock_tab_width,
    estimate_document_tab_width, floating_window_edge_group_key, floating_window_group_key,
    host_shell_pointer_route_group_key, resolve_host_drag_target_group_with_root_frames,
    resolve_host_tab_drop_route, resolve_host_tab_drop_route_with_root_frames,
    resolve_tab_drop_with_root_frames, HostDragTargetGroup, ResolvedHostTabDropRoute,
    ResolvedHostTabDropTarget, ResolvedTabDrop,
};
pub(super) use crate::ui::template_runtime::EditorUiCompatibilityHarness;
pub(super) use crate::ui::workbench::autolayout::{
    ShellFrame, ShellRegionId, WorkbenchChromeMetrics, WorkbenchShellGeometry,
};
pub(super) use crate::ui::workbench::layout::{
    ActivityDrawerLayout, ActivityDrawerMode, ActivityDrawerSlot, ActivityWindowId, DockEdge,
    DocumentNode, FloatingWindowLayout, MainHostPageLayout, MainPageId, SplitAxis, SplitPlacement,
    TabInsertionAnchor, TabInsertionSide, TabStackLayout, WorkbenchLayout, WorkspaceTarget,
};
pub(super) use crate::ui::workbench::model::{
    DocumentTabModel, DocumentWorkspaceModel, DrawerRingModel, FloatingWindowModel,
    MainHostStripModel, MainHostStripViewModel, MenuBarModel, PaneTabModel, StatusBarModel,
    ToolWindowStackModel, WorkbenchViewModel,
};
pub(super) use crate::ui::workbench::snapshot::{DocumentWorkspaceSnapshot, ViewContentKind};
pub(super) use crate::ui::workbench::view::{ViewDescriptorId, ViewHost, ViewInstanceId};
pub(super) use zircon_runtime_interface::ui::layout::{UiFrame, UiPoint, UiSize};

pub(super) fn workbench_page(id: MainPageId) -> MainHostPageLayout {
    MainHostPageLayout::WorkbenchPage {
        id,
        title: "Workbench".to_string(),
        activity_window: ActivityWindowId::workbench(),
        document_workspace: DocumentNode::default(),
    }
}

pub(super) fn drawer(
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

pub(super) fn default_drawers() -> BTreeMap<ActivityDrawerSlot, ActivityDrawerLayout> {
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

pub(super) fn default_drawers_model() -> BTreeMap<ActivityDrawerSlot, ToolWindowStackModel> {
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

pub(super) fn pane_tab(id: &str, title: &str, active: bool) -> PaneTabModel {
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

pub(super) fn tool_window_stack(
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

pub(super) fn document_tab(
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

pub(super) fn workbench_model(
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
            workspace: DocumentWorkspaceSnapshot::Tabs {
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

pub(super) fn floating_window(
    window_id: MainPageId,
    title: &str,
    tabs: Vec<DocumentTabModel>,
    focused_view: Option<&str>,
) -> FloatingWindowModel {
    floating_window_with_frame(
        window_id,
        title,
        ShellFrame::new(420.0, 180.0, 360.0, 240.0),
        tabs,
        focused_view,
    )
}

pub(super) fn floating_window_with_frame(
    window_id: MainPageId,
    title: &str,
    requested_frame: ShellFrame,
    tabs: Vec<DocumentTabModel>,
    focused_view: Option<&str>,
) -> FloatingWindowModel {
    FloatingWindowModel {
        window_id,
        title: title.to_string(),
        requested_frame,
        focused_view: focused_view.map(ViewInstanceId::new),
        tabs,
    }
}

pub(super) fn floating_tab(
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

pub(super) fn shell_geometry(
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

pub(super) fn root_frames_from_geometry(
    geometry: &WorkbenchShellGeometry,
) -> BuiltinHostRootShellFrames {
    root_frames_from_geometry_with_drawers(geometry, &[])
}

pub(super) fn root_frames_from_geometry_with_drawers(
    geometry: &WorkbenchShellGeometry,
    drawer_regions: &[ShellRegionId],
) -> BuiltinHostRootShellFrames {
    let shell_width = geometry
        .center_band_frame
        .width
        .max(geometry.status_bar_frame.width)
        .max(geometry.region_frame(ShellRegionId::Document).right());
    let shell_height = geometry
        .status_bar_frame
        .bottom()
        .max(geometry.center_band_frame.bottom());

    BuiltinHostRootShellFrames {
        shell_frame: Some(UiFrame::new(0.0, 0.0, shell_width, shell_height)),
        host_body_frame: Some(ui_frame(geometry.center_band_frame)),
        left_drawer_shell_frame: drawer_regions
            .contains(&ShellRegionId::Left)
            .then(|| ui_frame(geometry.region_frame(ShellRegionId::Left))),
        right_drawer_shell_frame: drawer_regions
            .contains(&ShellRegionId::Right)
            .then(|| ui_frame(geometry.region_frame(ShellRegionId::Right))),
        bottom_drawer_shell_frame: drawer_regions
            .contains(&ShellRegionId::Bottom)
            .then(|| ui_frame(geometry.region_frame(ShellRegionId::Bottom))),
        document_host_frame: Some(ui_frame(geometry.region_frame(ShellRegionId::Document))),
        status_bar_frame: Some(ui_frame(geometry.status_bar_frame)),
        ..BuiltinHostRootShellFrames::default()
    }
}

fn ui_frame(frame: ShellFrame) -> UiFrame {
    UiFrame::new(frame.x, frame.y, frame.width, frame.height)
}
