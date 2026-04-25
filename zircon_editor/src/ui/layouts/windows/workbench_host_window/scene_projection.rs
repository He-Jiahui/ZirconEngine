use slint::{Model, SharedString};

use super::*;
use crate::ui::asset_editor::ui_asset_editor_node_projection;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::animation_editor_pane_nodes;
use crate::ui::layouts::views::asset_browser_pane_nodes;
use crate::ui::layouts::views::assets_activity_pane_data;
use crate::ui::layouts::views::console_pane_nodes;
use crate::ui::layouts::views::hierarchy_pane_nodes;
use crate::ui::layouts::views::inspector_pane_nodes;
use crate::ui::layouts::views::project_overview_pane_data;
use zircon_runtime::ui::layout::UiSize;

const DEFAULT_PRESET_NAME: &str = "rider";
const OUTER_MARGIN_PX: f32 = 0.0;
const RAIL_WIDTH_PX: f32 = 34.0;
const TOP_BAR_HEIGHT_PX: f32 = 25.0;
const HOST_BAR_HEIGHT_PX: f32 = 24.0;
const PANEL_HEADER_HEIGHT_PX: f32 = 25.0;
const DOCUMENT_HEADER_HEIGHT_PX: f32 = 31.0;
const MIN_DROP_TARGET_PX: f32 = 92.0;

pub(crate) fn build_host_scene_data(
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    status_primary: &SharedString,
    delete_enabled: bool,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
) -> HostWindowSceneData {
    let metrics = surface_metrics();
    let orchestration =
        surface_orchestration_data(host_surface_data, host_shell, host_layout, &metrics);
    let resolved_preset_name = if host_shell.active_preset_name.is_empty() {
        SharedString::from(DEFAULT_PRESET_NAME)
    } else {
        host_shell.active_preset_name.clone()
    };

    let menu_chrome = HostMenuChromeData {
        outer_margin_px: metrics.outer_margin_px,
        top_bar_height_px: metrics.top_bar_height_px,
        save_project_enabled: host_shell.save_project_enabled,
        undo_enabled: host_shell.undo_enabled,
        redo_enabled: host_shell.redo_enabled,
        delete_enabled,
        preset_names: host_shell.preset_names.clone(),
        active_preset_name: host_shell.active_preset_name.clone(),
        resolved_preset_name,
    };
    let page_chrome = HostPageChromeData {
        top_bar_height_px: metrics.top_bar_height_px,
        host_bar_height_px: metrics.host_bar_height_px,
        tabs: host_surface_data.host_tabs.clone(),
        project_path: host_shell.project_path.clone(),
    };
    let status_bar = HostStatusBarData {
        status_bar_frame: host_layout.status_bar_frame.clone(),
        status_primary: status_primary.clone(),
        status_secondary: host_shell.status_secondary.clone(),
        viewport_label: host_shell.viewport_label.clone(),
    };
    let resize_layer = HostResizeLayerData {
        left_splitter_frame: host_layout.left_splitter_frame.clone(),
        right_splitter_frame: host_layout.right_splitter_frame.clone(),
        bottom_splitter_frame: host_layout.bottom_splitter_frame.clone(),
    };
    let drag_overlay = HostTabDragOverlayData {
        left_drop_enabled: host_shell.drawers_visible,
        right_drop_enabled: host_shell.drawers_visible,
        bottom_drop_enabled: host_shell.drawers_visible,
        left_drop_width_px: orchestration.left_stack_width_px.max(MIN_DROP_TARGET_PX),
        right_drop_width_px: orchestration.right_stack_width_px.max(MIN_DROP_TARGET_PX),
        bottom_drop_height_px: orchestration.bottom_panel_height_px.max(MIN_DROP_TARGET_PX),
        main_content_y_px: orchestration.main_content_y_px,
        main_content_height_px: host_layout.center_band_frame.height,
        document_zone_x_px: orchestration.document_zone_x_px,
        document_zone_width_px: host_layout.document_region_frame.width,
        bottom_drop_top_px: host_layout.status_bar_frame.y
            - orchestration.bottom_panel_height_px.max(MIN_DROP_TARGET_PX),
        drag_overlay_bottom_px: host_layout.status_bar_frame.y,
    };
    let left_content_height =
        (host_layout.left_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    let document_content_height =
        (host_layout.document_region_frame.height - metrics.document_header_height_px - 1.0)
            .max(0.0);
    let right_content_height =
        (host_layout.right_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    let bottom_content_height =
        (host_layout.bottom_region_frame.height - metrics.panel_header_height_px - 1.0).max(0.0);
    let floating_windows = floating_windows_with_pane_shell_layouts(
        &host_surface_data.floating_windows,
        metrics.document_header_height_px,
        project_overview,
    );
    let left_dock = HostSideDockSurfaceData {
        region_frame: host_layout.left_region_frame.clone(),
        surface_key: "left".into(),
        rail_before_panel: true,
        tabs: host_surface_data.left_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.left_pane.clone(),
            orchestration.left_panel_width_px,
            left_content_height,
            project_overview,
        ),
        rail_width_px: orchestration.left_rail_width_px,
        panel_width_px: orchestration.left_panel_width_px,
        panel_header_height_px: metrics.panel_header_height_px,
        tab_origin_x_px: orchestration.left_tab_origin_x_px,
        tab_origin_y_px: orchestration.left_tab_origin_y_px,
    };
    let document_dock = HostDocumentDockSurfaceData {
        region_frame: host_layout.document_region_frame.clone(),
        surface_key: "document".into(),
        tabs: host_surface_data.document_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.document_pane.clone(),
            host_layout.document_region_frame.width,
            document_content_height,
            project_overview,
        ),
        header_height_px: metrics.document_header_height_px,
        tab_origin_x_px: orchestration.document_tab_origin_x_px,
        tab_origin_y_px: orchestration.document_tab_origin_y_px,
    };
    let right_dock = HostSideDockSurfaceData {
        region_frame: host_layout.right_region_frame.clone(),
        surface_key: "right".into(),
        rail_before_panel: false,
        tabs: host_surface_data.right_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.right_pane.clone(),
            orchestration.right_panel_width_px,
            right_content_height,
            project_overview,
        ),
        rail_width_px: orchestration.right_rail_width_px,
        panel_width_px: orchestration.right_panel_width_px,
        panel_header_height_px: metrics.panel_header_height_px,
        tab_origin_x_px: orchestration.right_tab_origin_x_px,
        tab_origin_y_px: orchestration.right_tab_origin_y_px,
    };
    let bottom_dock = HostBottomDockSurfaceData {
        region_frame: host_layout.bottom_region_frame.clone(),
        surface_key: "bottom".into(),
        tabs: host_surface_data.bottom_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.bottom_pane.clone(),
            host_layout.bottom_region_frame.width,
            bottom_content_height,
            project_overview,
        ),
        expanded: host_shell.bottom_expanded,
        header_height_px: metrics.panel_header_height_px,
        tab_origin_x_px: orchestration.bottom_tab_origin_x_px,
        tab_origin_y_px: orchestration.bottom_tab_origin_y_px,
    };
    let floating_layer = HostFloatingWindowLayerData {
        floating_windows,
        header_height_px: metrics.document_header_height_px,
    };

    HostWindowSceneData {
        layout: host_layout.clone(),
        metrics,
        orchestration,
        menu_chrome,
        page_chrome,
        status_bar,
        resize_layer,
        drag_overlay,
        left_dock,
        document_dock,
        right_dock,
        bottom_dock,
        floating_layer,
    }
}

pub(crate) fn build_native_floating_surface_data(
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
) -> HostNativeFloatingWindowSurfaceData {
    HostNativeFloatingWindowSurfaceData {
        floating_windows: floating_windows_with_pane_shell_layouts(
            &host_surface_data.floating_windows,
            DOCUMENT_HEADER_HEIGHT_PX,
            project_overview,
        ),
        native_floating_window_id: host_shell.native_floating_window_id.clone(),
        native_window_bounds: host_shell.native_window_bounds.clone(),
        header_height_px: DOCUMENT_HEADER_HEIGHT_PX,
    }
}

fn surface_metrics() -> HostWindowSurfaceMetricsData {
    HostWindowSurfaceMetricsData {
        outer_margin_px: OUTER_MARGIN_PX,
        rail_width_px: RAIL_WIDTH_PX,
        top_bar_height_px: TOP_BAR_HEIGHT_PX,
        host_bar_height_px: HOST_BAR_HEIGHT_PX,
        panel_header_height_px: PANEL_HEADER_HEIGHT_PX,
        document_header_height_px: DOCUMENT_HEADER_HEIGHT_PX,
    }
}

fn surface_orchestration_data(
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    metrics: &HostWindowSurfaceMetricsData,
) -> HostWindowSurfaceOrchestrationData {
    let left_has_rail =
        host_layout.left_region_frame.width > 0.0 && host_surface_data.left_tabs.row_count() > 0;
    let right_has_rail =
        host_layout.right_region_frame.width > 0.0 && host_surface_data.right_tabs.row_count() > 0;
    let left_rail_width_px = if left_has_rail {
        metrics.rail_width_px
    } else {
        0.0
    };
    let right_rail_width_px = if right_has_rail {
        metrics.rail_width_px
    } else {
        0.0
    };
    let left_stack_width_px = host_layout.left_region_frame.width;
    let right_stack_width_px = host_layout.right_region_frame.width;
    let left_panel_width_px =
        if host_shell.left_expanded && host_layout.left_region_frame.width > left_rail_width_px {
            host_layout.left_region_frame.width - left_rail_width_px
        } else {
            0.0
        };
    let right_panel_width_px = if host_shell.right_expanded
        && host_layout.right_region_frame.width > right_rail_width_px
    {
        host_layout.right_region_frame.width - right_rail_width_px
    } else {
        0.0
    };

    HostWindowSurfaceOrchestrationData {
        left_rail_width_px,
        right_rail_width_px,
        left_stack_width_px,
        right_stack_width_px,
        left_panel_width_px,
        right_panel_width_px,
        bottom_panel_height_px: host_layout.bottom_region_frame.height,
        main_content_y_px: host_layout.center_band_frame.y,
        document_zone_x_px: host_layout.document_region_frame.x,
        right_stack_x_px: host_layout.right_region_frame.x,
        bottom_panel_y_px: host_layout.bottom_region_frame.y,
        left_tab_origin_x_px: left_rail_width_px
            + if left_panel_width_px > 0.0 { 1.0 } else { 0.0 }
            + 6.0,
        left_tab_origin_y_px: host_layout.center_band_frame.y + 2.0,
        document_tab_origin_x_px: host_layout.document_region_frame.x + 8.0,
        document_tab_origin_y_px: host_layout.center_band_frame.y + 1.0,
        right_tab_origin_x_px: host_layout.right_region_frame.x + 6.0,
        right_tab_origin_y_px: host_layout.center_band_frame.y + 2.0,
        bottom_tab_origin_x_px: 6.0,
        bottom_tab_origin_y_px: host_layout.bottom_region_frame.y + 2.0,
    }
}

fn pane_with_ui_asset_nodes(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "UiAssetEditor" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    let projection = ui_asset_editor_node_projection(size);
    pane.body_compat.ui_asset.nodes = projection.nodes;
    pane.body_compat.ui_asset.center_column_node = projection.center_column_node;
    pane.body_compat.ui_asset.designer_panel_node = projection.designer_panel_node;
    pane.body_compat.ui_asset.designer_canvas_panel_node = projection.designer_canvas_panel_node;
    pane.body_compat.ui_asset.inspector_panel_node = projection.inspector_panel_node;
    pane.body_compat.ui_asset.stylesheet_panel_node = projection.stylesheet_panel_node;
    pane
}

fn pane_with_host_owned_shell_layouts(
    mut pane: PaneData,
    width: f32,
    height: f32,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
) -> PaneData {
    pane = pane_with_ui_asset_nodes(pane, width, height);
    pane = pane_with_hierarchy_projection(pane, width, height);
    pane = pane_with_inspector_projection(pane, width, height);
    pane = pane_with_console_projection(pane, width, height);
    pane = pane_with_assets_activity_projection(pane, width, height);
    pane = pane_with_asset_browser_projection(pane, width, height);
    pane = pane_with_project_overview_projection(pane, width, height, project_overview);
    pane_with_animation_projection(pane, width, height)
}

fn pane_with_hierarchy_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "Hierarchy" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.body_compat.hierarchy.nodes = hierarchy_pane_nodes(size);
    pane
}

fn pane_with_inspector_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "Inspector" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.body_compat.inspector.nodes = inspector_pane_nodes(size);
    pane
}

fn pane_with_console_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "Console" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.body_compat.console.nodes = console_pane_nodes(size);
    pane
}

fn pane_with_assets_activity_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "Assets" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.body_compat.assets_activity = assets_activity_pane_data(size);
    pane
}

fn pane_with_asset_browser_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "AssetBrowser" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.body_compat.asset_browser.nodes = asset_browser_pane_nodes(size);
    pane
}

fn pane_with_project_overview_projection(
    mut pane: PaneData,
    width: f32,
    height: f32,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
) -> PaneData {
    if pane.kind.as_str() != "Project" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.body_compat.project_overview = project_overview_pane_data(project_overview, size);
    pane
}

fn pane_with_animation_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "AnimationSequenceEditor"
        && pane.kind.as_str() != "AnimationGraphEditor"
    {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.body_compat.animation.nodes = animation_editor_pane_nodes(size);
    pane
}

fn floating_windows_with_pane_shell_layouts(
    floating_windows: &slint::ModelRc<FloatingWindowData>,
    header_height_px: f32,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
) -> slint::ModelRc<FloatingWindowData> {
    model_rc(
        (0..floating_windows.row_count())
            .filter_map(|row| floating_windows.row_data(row))
            .map(|mut window| {
                let content_height = (window.frame.height - header_height_px).max(0.0);
                window.active_pane = pane_with_host_owned_shell_layouts(
                    window.active_pane.clone(),
                    window.frame.width,
                    content_height,
                    project_overview,
                );
                window
            })
            .collect(),
    )
}
