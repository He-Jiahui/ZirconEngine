use super::super::chrome_template_projection::{
    activity_rail_active_control_id, activity_rail_button_frames, activity_rail_nodes_for_surface,
    bottom_dock_header_nodes, dock_header_frame, dock_overflow_frame, dock_tab_frames,
    side_dock_header_nodes, surface_metrics_from_chrome_assets,
};
use super::super::{
    FrameRect, HostBottomDockSurfaceData, HostSideDockSurfaceData, HostWindowLayoutData,
    HostWindowShellData, HostWindowSurfaceData, HostWindowSurfaceMetricsData,
    HostWindowSurfaceOrchestrationData,
};
use super::{pane_with_host_owned_shell_layouts, surface_orchestration_data};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HostDockSurfaceId {
    Left,
    Right,
    Bottom,
}

pub(crate) enum HostDockSurfacePatch {
    Left(HostSideDockSurfaceData),
    Right(HostSideDockSurfaceData),
    Bottom(HostBottomDockSurfaceData),
}

pub(crate) fn build_host_dock_surface_patch(
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
    target: HostDockSurfaceId,
) -> HostDockSurfacePatch {
    let shell_width = host_layout
        .status_bar_frame
        .width
        .max(host_layout.center_band_frame.width)
        .max(host_layout.bottom_region_frame.width)
        .max(0.0);
    let metrics = surface_metrics_from_chrome_assets(shell_width);
    let orchestration =
        surface_orchestration_data(host_surface_data, host_shell, host_layout, &metrics);
    match target {
        HostDockSurfaceId::Left => HostDockSurfacePatch::Left(build_left_dock(
            host_surface_data,
            host_shell,
            host_layout,
            &metrics,
            &orchestration,
            project_overview,
            chrome,
        )),
        HostDockSurfaceId::Right => HostDockSurfacePatch::Right(build_right_dock(
            host_surface_data,
            host_shell,
            host_layout,
            &metrics,
            &orchestration,
            project_overview,
            chrome,
        )),
        HostDockSurfaceId::Bottom => HostDockSurfacePatch::Bottom(build_bottom_dock(
            host_surface_data,
            host_shell,
            host_layout,
            &metrics,
            project_overview,
            chrome,
        )),
    }
}

pub(super) fn build_left_dock(
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    metrics: &HostWindowSurfaceMetricsData,
    orchestration: &HostWindowSurfaceOrchestrationData,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
) -> HostSideDockSurfaceData {
    let content_height =
        (host_layout.left_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    let header_nodes = side_dock_header_nodes(
        &host_surface_data.left_tabs,
        &host_shell.panel_preset_id,
        orchestration.left_panel_width_px,
        metrics.panel_header_height_px,
    );
    let rail_nodes = activity_rail_nodes_for_surface(
        "host.left.activity.rail",
        &host_surface_data.left_tabs,
        &host_shell.shell_preset_id,
        orchestration.left_rail_width_px,
        host_layout.left_region_frame.height,
    );
    HostSideDockSurfaceData {
        region_frame: host_layout.left_region_frame.clone(),
        surface_key: "left".into(),
        rail_before_panel: true,
        rail_button_frames: activity_rail_button_frames(&rail_nodes, &host_surface_data.left_tabs),
        rail_active_control_id: activity_rail_active_control_id(&host_surface_data.left_tabs),
        rail_nodes,
        header_frame: dock_header_frame(&header_nodes),
        overflow_frame: dock_overflow_frame(&header_nodes),
        content_frame: FrameRect {
            x: 0.0,
            y: metrics.panel_header_height_px + 1.0,
            width: orchestration.left_panel_width_px,
            height: content_height,
        },
        tab_frames: dock_tab_frames(&header_nodes, &host_surface_data.left_tabs),
        header_nodes,
        tabs: host_surface_data.left_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.left_pane.clone(),
            orchestration.left_panel_width_px,
            content_height,
            project_overview,
            chrome,
        ),
        rail_width_px: orchestration.left_rail_width_px,
        panel_width_px: orchestration.left_panel_width_px,
        panel_header_height_px: metrics.panel_header_height_px,
    }
}

pub(super) fn build_right_dock(
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    metrics: &HostWindowSurfaceMetricsData,
    orchestration: &HostWindowSurfaceOrchestrationData,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
) -> HostSideDockSurfaceData {
    let content_height =
        (host_layout.right_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    let header_nodes = side_dock_header_nodes(
        &host_surface_data.right_tabs,
        &host_shell.panel_preset_id,
        orchestration.right_panel_width_px,
        metrics.panel_header_height_px,
    );
    let rail_nodes = activity_rail_nodes_for_surface(
        "host.right.activity.rail",
        &host_surface_data.right_tabs,
        &host_shell.shell_preset_id,
        orchestration.right_rail_width_px,
        host_layout.right_region_frame.height,
    );
    HostSideDockSurfaceData {
        region_frame: host_layout.right_region_frame.clone(),
        surface_key: "right".into(),
        rail_before_panel: false,
        rail_button_frames: activity_rail_button_frames(&rail_nodes, &host_surface_data.right_tabs),
        rail_active_control_id: activity_rail_active_control_id(&host_surface_data.right_tabs),
        rail_nodes,
        header_frame: dock_header_frame(&header_nodes),
        overflow_frame: dock_overflow_frame(&header_nodes),
        content_frame: FrameRect {
            x: 0.0,
            y: metrics.panel_header_height_px + 1.0,
            width: orchestration.right_panel_width_px,
            height: content_height,
        },
        tab_frames: dock_tab_frames(&header_nodes, &host_surface_data.right_tabs),
        header_nodes,
        tabs: host_surface_data.right_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.right_pane.clone(),
            orchestration.right_panel_width_px,
            content_height,
            project_overview,
            chrome,
        ),
        rail_width_px: orchestration.right_rail_width_px,
        panel_width_px: orchestration.right_panel_width_px,
        panel_header_height_px: metrics.panel_header_height_px,
    }
}

pub(super) fn build_bottom_dock(
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    metrics: &HostWindowSurfaceMetricsData,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
    chrome: &crate::ui::workbench::snapshot::EditorChromeSnapshot,
) -> HostBottomDockSurfaceData {
    let content_height =
        (host_layout.bottom_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    let header_nodes = bottom_dock_header_nodes(
        &host_surface_data.bottom_tabs,
        &host_shell.panel_preset_id,
        host_layout.bottom_region_frame.width,
        metrics.panel_header_height_px,
    );
    HostBottomDockSurfaceData {
        region_frame: host_layout.bottom_region_frame.clone(),
        surface_key: "bottom".into(),
        header_frame: dock_header_frame(&header_nodes),
        overflow_frame: dock_overflow_frame(&header_nodes),
        content_frame: FrameRect {
            x: 0.0,
            y: metrics.panel_header_height_px + 1.0,
            width: host_layout.bottom_region_frame.width,
            height: content_height,
        },
        tab_frames: dock_tab_frames(&header_nodes, &host_surface_data.bottom_tabs),
        header_nodes,
        tabs: host_surface_data.bottom_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.bottom_pane.clone(),
            host_layout.bottom_region_frame.width,
            content_height,
            project_overview,
            chrome,
        ),
        expanded: host_shell.bottom_expanded,
        header_height_px: metrics.panel_header_height_px,
    }
}

pub(super) fn rebuild_left_dock_geometry(
    current: &HostSideDockSurfaceData,
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    metrics: &HostWindowSurfaceMetricsData,
    orchestration: &HostWindowSurfaceOrchestrationData,
) -> HostSideDockSurfaceData {
    let mut dock = current.clone();
    let header_nodes = side_dock_header_nodes(
        &host_surface_data.left_tabs,
        &host_shell.panel_preset_id,
        orchestration.left_panel_width_px,
        metrics.panel_header_height_px,
    );
    let rail_nodes = activity_rail_nodes_for_surface(
        "host.left.activity.rail",
        &host_surface_data.left_tabs,
        &host_shell.shell_preset_id,
        orchestration.left_rail_width_px,
        host_layout.left_region_frame.height,
    );
    let content_height =
        (host_layout.left_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    dock.region_frame = host_layout.left_region_frame.clone();
    dock.rail_before_panel = true;
    dock.rail_button_frames =
        activity_rail_button_frames(&rail_nodes, &host_surface_data.left_tabs);
    dock.rail_active_control_id = activity_rail_active_control_id(&host_surface_data.left_tabs);
    dock.rail_nodes = rail_nodes;
    dock.header_frame = dock_header_frame(&header_nodes);
    dock.overflow_frame = dock_overflow_frame(&header_nodes);
    dock.content_frame = FrameRect {
        x: 0.0,
        y: metrics.panel_header_height_px + 1.0,
        width: orchestration.left_panel_width_px,
        height: content_height,
    };
    dock.tab_frames = dock_tab_frames(&header_nodes, &host_surface_data.left_tabs);
    dock.header_nodes = header_nodes;
    dock.tabs = host_surface_data.left_tabs.clone();
    dock.rail_width_px = orchestration.left_rail_width_px;
    dock.panel_width_px = orchestration.left_panel_width_px;
    dock.panel_header_height_px = metrics.panel_header_height_px;
    dock
}

pub(super) fn rebuild_right_dock_geometry(
    current: &HostSideDockSurfaceData,
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    metrics: &HostWindowSurfaceMetricsData,
    orchestration: &HostWindowSurfaceOrchestrationData,
) -> HostSideDockSurfaceData {
    let mut dock = current.clone();
    let header_nodes = side_dock_header_nodes(
        &host_surface_data.right_tabs,
        &host_shell.panel_preset_id,
        orchestration.right_panel_width_px,
        metrics.panel_header_height_px,
    );
    let rail_nodes = activity_rail_nodes_for_surface(
        "host.right.activity.rail",
        &host_surface_data.right_tabs,
        &host_shell.shell_preset_id,
        orchestration.right_rail_width_px,
        host_layout.right_region_frame.height,
    );
    let content_height =
        (host_layout.right_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    dock.region_frame = host_layout.right_region_frame.clone();
    dock.rail_before_panel = false;
    dock.rail_button_frames =
        activity_rail_button_frames(&rail_nodes, &host_surface_data.right_tabs);
    dock.rail_active_control_id = activity_rail_active_control_id(&host_surface_data.right_tabs);
    dock.rail_nodes = rail_nodes;
    dock.header_frame = dock_header_frame(&header_nodes);
    dock.overflow_frame = dock_overflow_frame(&header_nodes);
    dock.content_frame = FrameRect {
        x: 0.0,
        y: metrics.panel_header_height_px + 1.0,
        width: orchestration.right_panel_width_px,
        height: content_height,
    };
    dock.tab_frames = dock_tab_frames(&header_nodes, &host_surface_data.right_tabs);
    dock.header_nodes = header_nodes;
    dock.tabs = host_surface_data.right_tabs.clone();
    dock.rail_width_px = orchestration.right_rail_width_px;
    dock.panel_width_px = orchestration.right_panel_width_px;
    dock.panel_header_height_px = metrics.panel_header_height_px;
    dock
}

pub(super) fn rebuild_bottom_dock_geometry(
    current: &HostBottomDockSurfaceData,
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    metrics: &HostWindowSurfaceMetricsData,
) -> HostBottomDockSurfaceData {
    let mut dock = current.clone();
    let header_nodes = bottom_dock_header_nodes(
        &host_surface_data.bottom_tabs,
        &host_shell.panel_preset_id,
        host_layout.bottom_region_frame.width,
        metrics.panel_header_height_px,
    );
    let content_height =
        (host_layout.bottom_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    dock.region_frame = host_layout.bottom_region_frame.clone();
    dock.header_frame = dock_header_frame(&header_nodes);
    dock.overflow_frame = dock_overflow_frame(&header_nodes);
    dock.content_frame = FrameRect {
        x: 0.0,
        y: metrics.panel_header_height_px + 1.0,
        width: host_layout.bottom_region_frame.width,
        height: content_height,
    };
    dock.tab_frames = dock_tab_frames(&header_nodes, &host_surface_data.bottom_tabs);
    dock.header_nodes = header_nodes;
    dock.tabs = host_surface_data.bottom_tabs.clone();
    dock.expanded = host_shell.bottom_expanded;
    dock.header_height_px = metrics.panel_header_height_px;
    dock
}
