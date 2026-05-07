use slint::{Model, ModelRc, SharedString};

use super::chrome_template_projection::{
    activity_rail_active_control_id, activity_rail_button_frames, activity_rail_nodes,
    bottom_dock_header_nodes, dock_header_frame, dock_subtitle_frame, dock_tab_frames,
    document_dock_header_nodes, floating_window_header_nodes, menu_chrome_nodes,
    menu_control_frames, menu_popup_nodes, page_chrome_nodes, page_project_path_frame,
    page_tab_frames, page_tab_row_frame, side_dock_header_nodes, status_bar_nodes,
    surface_metrics_from_chrome_assets, MENU_SLOT_COUNT,
};
use super::*;
use crate::ui::asset_editor::ui_asset_editor_node_projection;
use crate::ui::binding::EditorUiBindingPayload;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views::animation_editor_pane_nodes;
use crate::ui::layouts::views::asset_browser_pane_nodes;
use crate::ui::layouts::views::assets_activity_pane_data;
use crate::ui::layouts::views::console_pane_nodes;
use crate::ui::layouts::views::hierarchy_pane_nodes;
use crate::ui::layouts::views::inspector_pane_nodes;
use crate::ui::layouts::views::project_overview_pane_data;
use crate::ui::workbench::model::{MenuBarModel, MenuItemModel, MenuModel};
use zircon_runtime_interface::ui::layout::UiSize;

const DEFAULT_PRESET_NAME: &str = "rider";
const MIN_DROP_TARGET_PX: f32 = 92.0;
const MENU_POPUP_WIDTHS_PX: [f32; 7] = [208.0, 186.0, 218.0, 172.0, 198.0, 224.0, 194.0];
const DEFAULT_MENU_POPUP_WIDTH_PX: f32 = 224.0;
const MENU_POPUP_PADDING_PX: f32 = 6.0;
const MENU_POPUP_ROW_HEIGHT_PX: f32 = 28.0;
const MENU_POPUP_ROW_GAP_PX: f32 = 2.0;

pub(crate) fn build_host_scene_data(
    menu_bar: &MenuBarModel,
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    status_primary: &SharedString,
    delete_enabled: bool,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
) -> HostWindowSceneData {
    let resolved_preset_name = if host_shell.active_preset_name.is_empty() {
        SharedString::from(DEFAULT_PRESET_NAME)
    } else {
        host_shell.active_preset_name.clone()
    };
    let shell_width = host_layout
        .status_bar_frame
        .width
        .max(host_layout.center_band_frame.width)
        .max(host_layout.bottom_region_frame.width)
        .max(0.0);
    let metrics = surface_metrics_from_chrome_assets(shell_width);
    let orchestration =
        surface_orchestration_data(host_surface_data, host_shell, host_layout, &metrics);

    let menu_chrome = host_menu_chrome_data(
        menu_bar,
        host_shell,
        delete_enabled,
        &metrics,
        resolved_preset_name,
        shell_width,
    );
    let page_template_nodes = page_chrome_nodes(
        &host_surface_data.host_tabs,
        &host_shell.project_path,
        shell_width,
        metrics.top_bar_height_px + 1.0 + metrics.host_bar_height_px,
    );
    let page_chrome = HostPageChromeData {
        top_bar_height_px: metrics.top_bar_height_px,
        host_bar_height_px: metrics.host_bar_height_px,
        tab_row_frame: page_tab_row_frame(&page_template_nodes),
        project_path_frame: page_project_path_frame(&page_template_nodes),
        tab_frames: page_tab_frames(&page_template_nodes, &host_surface_data.host_tabs),
        template_nodes: page_template_nodes,
        tabs: host_surface_data.host_tabs.clone(),
        project_path: host_shell.project_path.clone(),
    };
    let status_bar = HostStatusBarData {
        status_bar_frame: host_layout.status_bar_frame.clone(),
        template_nodes: status_bar_nodes(
            status_primary,
            &host_shell.status_secondary,
            &host_shell.viewport_label,
            host_layout.status_bar_frame.width,
            host_layout.status_bar_frame.height,
        ),
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
    let left_header_nodes = side_dock_header_nodes(
        &host_surface_data.left_tabs,
        orchestration.left_panel_width_px,
        metrics.panel_header_height_px,
    );
    let left_rail_nodes = activity_rail_nodes(
        &host_surface_data.left_tabs,
        orchestration.left_rail_width_px,
        host_layout.left_region_frame.height,
    );
    let left_content_frame = FrameRect {
        x: 0.0,
        y: metrics.panel_header_height_px + 1.0,
        width: orchestration.left_panel_width_px,
        height: left_content_height,
    };
    let left_dock = HostSideDockSurfaceData {
        region_frame: host_layout.left_region_frame.clone(),
        surface_key: "left".into(),
        rail_before_panel: true,
        rail_button_frames: activity_rail_button_frames(
            &left_rail_nodes,
            &host_surface_data.left_tabs,
        ),
        rail_active_control_id: activity_rail_active_control_id(&host_surface_data.left_tabs),
        rail_nodes: left_rail_nodes,
        header_frame: dock_header_frame(&left_header_nodes),
        content_frame: left_content_frame,
        tab_frames: dock_tab_frames(&left_header_nodes, &host_surface_data.left_tabs),
        header_nodes: left_header_nodes,
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
    };
    let document_header_nodes = document_dock_header_nodes(
        &host_surface_data.document_tabs,
        &host_surface_data.document_pane.subtitle,
        host_layout.document_region_frame.width,
        metrics.document_header_height_px,
    );
    let document_content_frame = FrameRect {
        x: 0.0,
        y: metrics.document_header_height_px + 1.0,
        width: host_layout.document_region_frame.width,
        height: document_content_height,
    };
    let document_dock = HostDocumentDockSurfaceData {
        region_frame: host_layout.document_region_frame.clone(),
        surface_key: "document".into(),
        header_frame: dock_header_frame(&document_header_nodes),
        subtitle_frame: dock_subtitle_frame(&document_header_nodes),
        content_frame: document_content_frame,
        tab_frames: dock_tab_frames(&document_header_nodes, &host_surface_data.document_tabs),
        header_nodes: document_header_nodes,
        tabs: host_surface_data.document_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.document_pane.clone(),
            host_layout.document_region_frame.width,
            document_content_height,
            project_overview,
        ),
        header_height_px: metrics.document_header_height_px,
    };
    let right_header_nodes = side_dock_header_nodes(
        &host_surface_data.right_tabs,
        orchestration.right_panel_width_px,
        metrics.panel_header_height_px,
    );
    let right_rail_nodes = activity_rail_nodes(
        &host_surface_data.right_tabs,
        orchestration.right_rail_width_px,
        host_layout.right_region_frame.height,
    );
    let right_content_frame = FrameRect {
        x: 0.0,
        y: metrics.panel_header_height_px + 1.0,
        width: orchestration.right_panel_width_px,
        height: right_content_height,
    };
    let right_dock = HostSideDockSurfaceData {
        region_frame: host_layout.right_region_frame.clone(),
        surface_key: "right".into(),
        rail_before_panel: false,
        rail_button_frames: activity_rail_button_frames(
            &right_rail_nodes,
            &host_surface_data.right_tabs,
        ),
        rail_active_control_id: activity_rail_active_control_id(&host_surface_data.right_tabs),
        rail_nodes: right_rail_nodes,
        header_frame: dock_header_frame(&right_header_nodes),
        content_frame: right_content_frame,
        tab_frames: dock_tab_frames(&right_header_nodes, &host_surface_data.right_tabs),
        header_nodes: right_header_nodes,
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
    };
    let bottom_header_nodes = bottom_dock_header_nodes(
        &host_surface_data.bottom_tabs,
        host_layout.bottom_region_frame.width,
        metrics.panel_header_height_px,
    );
    let bottom_content_frame = FrameRect {
        x: 0.0,
        y: metrics.panel_header_height_px + 1.0,
        width: host_layout.bottom_region_frame.width,
        height: bottom_content_height,
    };
    let bottom_dock = HostBottomDockSurfaceData {
        region_frame: host_layout.bottom_region_frame.clone(),
        surface_key: "bottom".into(),
        header_frame: dock_header_frame(&bottom_header_nodes),
        content_frame: bottom_content_frame,
        tab_frames: dock_tab_frames(&bottom_header_nodes, &host_surface_data.bottom_tabs),
        header_nodes: bottom_header_nodes,
        tabs: host_surface_data.bottom_tabs.clone(),
        pane: pane_with_host_owned_shell_layouts(
            host_surface_data.bottom_pane.clone(),
            host_layout.bottom_region_frame.width,
            bottom_content_height,
            project_overview,
        ),
        expanded: host_shell.bottom_expanded,
        header_height_px: metrics.panel_header_height_px,
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

fn host_menu_chrome_data(
    menu_bar: &MenuBarModel,
    host_shell: &HostWindowShellData,
    delete_enabled: bool,
    metrics: &HostWindowSurfaceMetricsData,
    resolved_preset_name: SharedString,
    shell_width: f32,
) -> HostMenuChromeData {
    let menus = model_rc(
        menu_bar
            .menus
            .iter()
            .enumerate()
            .map(|(index, menu)| {
                host_menu_chrome_menu_data(menu, index, host_shell, &resolved_preset_name)
            })
            .collect(),
    );
    let template_nodes = menu_chrome_nodes(&menus, shell_width, metrics.top_bar_height_px + 1.0);
    let menu_frames = menu_control_frames(&template_nodes, menus.row_count().max(MENU_SLOT_COUNT));

    HostMenuChromeData {
        outer_margin_px: metrics.outer_margin_px,
        top_bar_height_px: metrics.top_bar_height_px,
        template_nodes,
        menu_frames,
        save_project_enabled: host_shell.save_project_enabled,
        undo_enabled: host_shell.undo_enabled,
        redo_enabled: host_shell.redo_enabled,
        delete_enabled,
        preset_names: host_shell.preset_names.clone(),
        active_preset_name: host_shell.active_preset_name.clone(),
        resolved_preset_name: resolved_preset_name.clone(),
        menus,
    }
}

fn host_menu_chrome_menu_data(
    menu: &MenuModel,
    menu_index: usize,
    host_shell: &HostWindowShellData,
    resolved_preset_name: &SharedString,
) -> HostMenuChromeMenuData {
    let mut items = if menu.label.eq_ignore_ascii_case("Window") {
        window_menu_items(menu, host_shell, resolved_preset_name)
    } else {
        host_menu_items(&menu.items)
    };
    let popup_height_px = menu_popup_height(items.len());
    let popup_width_px = MENU_POPUP_WIDTHS_PX
        .get(menu_index)
        .copied()
        .unwrap_or(DEFAULT_MENU_POPUP_WIDTH_PX);
    let item_model = model_rc(std::mem::take(&mut items));
    let popup_nodes = menu_popup_nodes(&item_model, popup_width_px, popup_height_px);

    HostMenuChromeMenuData {
        label: menu.label.clone().into(),
        popup_width_px,
        popup_height_px,
        popup_nodes,
        items: item_model,
    }
}

fn window_menu_items(
    menu: &MenuModel,
    host_shell: &HostWindowShellData,
    resolved_preset_name: &SharedString,
) -> Vec<HostMenuChromeItemData> {
    let mut items = vec![HostMenuChromeItemData {
        label: "Save Preset Asset".into(),
        shortcut: resolved_preset_name.clone(),
        action_id: format!("SavePreset.{resolved_preset_name}").into(),
        enabled: true,
        children: ModelRc::default(),
    }];
    items.extend(host_menu_items(&menu.items));
    items.extend((0..host_shell.preset_names.row_count()).filter_map(|row| {
        let preset = host_shell.preset_names.row_data(row)?;
        Some(HostMenuChromeItemData {
            label: preset.clone(),
            shortcut: if preset == host_shell.active_preset_name {
                "active".into()
            } else {
                "".into()
            },
            action_id: format!("LoadPreset.{preset}").into(),
            enabled: true,
            children: ModelRc::default(),
        })
    }));
    items
}

fn host_menu_items(items: &[MenuItemModel]) -> Vec<HostMenuChromeItemData> {
    items.iter().map(host_menu_chrome_item).collect()
}

fn host_menu_chrome_item(item: &MenuItemModel) -> HostMenuChromeItemData {
    if item.has_children() {
        HostMenuChromeItemData {
            label: item.label.clone().into(),
            shortcut: ">".into(),
            action_id: "".into(),
            enabled: item.enabled,
            children: model_rc(host_menu_items(&item.children)),
        }
    } else {
        HostMenuChromeItemData {
            label: item.label.clone().into(),
            shortcut: item.shortcut.clone().unwrap_or_default().into(),
            action_id: menu_item_action_id(item),
            enabled: item.enabled,
            children: ModelRc::default(),
        }
    }
}

fn menu_item_action_id(item: &MenuItemModel) -> SharedString {
    match item.binding.payload() {
        EditorUiBindingPayload::MenuAction { action_id } => action_id.as_str().into(),
        EditorUiBindingPayload::EditorOperation { operation_id, .. } => {
            operation_id.as_str().into()
        }
        _ => "".into(),
    }
}

fn menu_popup_height(item_count: usize) -> f32 {
    if item_count == 0 {
        0.0
    } else {
        MENU_POPUP_PADDING_PX * 2.0
            + item_count as f32 * MENU_POPUP_ROW_HEIGHT_PX
            + (item_count as f32 - 1.0) * MENU_POPUP_ROW_GAP_PX
    }
}

pub(crate) fn build_native_floating_surface_data(
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    project_overview: &crate::ui::workbench::snapshot::ProjectOverviewSnapshot,
) -> HostNativeFloatingWindowSurfaceData {
    let metrics =
        surface_metrics_from_chrome_assets(host_shell.native_window_bounds.width.max(0.0));
    HostNativeFloatingWindowSurfaceData {
        floating_windows: floating_windows_with_pane_shell_layouts(
            &host_surface_data.floating_windows,
            metrics.document_header_height_px,
            project_overview,
        ),
        native_floating_window_id: host_shell.native_floating_window_id.clone(),
        native_window_bounds: host_shell.native_window_bounds.clone(),
        header_height_px: metrics.document_header_height_px,
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
    }
}

fn pane_with_ui_asset_nodes(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "UiAssetEditor" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    let projection = ui_asset_editor_node_projection(size);
    pane.native_body.ui_asset.nodes = projection.nodes;
    pane.native_body.ui_asset.center_column_node = projection.center_column_node;
    pane.native_body.ui_asset.designer_panel_node = projection.designer_panel_node;
    pane.native_body.ui_asset.designer_canvas_panel_node = projection.designer_canvas_panel_node;
    pane.native_body.ui_asset.inspector_panel_node = projection.inspector_panel_node;
    pane.native_body.ui_asset.stylesheet_panel_node = projection.stylesheet_panel_node;
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
    pane.native_body.hierarchy.nodes = hierarchy_pane_nodes(size);
    pane
}

fn pane_with_inspector_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "Inspector" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.native_body.inspector.nodes = inspector_pane_nodes(size);
    pane
}

fn pane_with_console_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "Console" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.native_body.console.nodes = console_pane_nodes(size);
    pane
}

fn pane_with_assets_activity_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "Assets" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.native_body.assets_activity = assets_activity_pane_data(size);
    pane
}

fn pane_with_asset_browser_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "AssetBrowser" {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.native_body.asset_browser.nodes = asset_browser_pane_nodes(size);
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
    pane.native_body.project_overview = project_overview_pane_data(project_overview, size);
    pane
}

fn pane_with_animation_projection(mut pane: PaneData, width: f32, height: f32) -> PaneData {
    if pane.kind.as_str() != "AnimationSequenceEditor"
        && pane.kind.as_str() != "AnimationGraphEditor"
    {
        return pane;
    }

    let size = UiSize::new(width.max(0.0), height.max(0.0));
    pane.native_body.animation.nodes = animation_editor_pane_nodes(size);
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
                let header_nodes = floating_window_header_nodes(
                    &window.tabs,
                    &window.title,
                    window.frame.width,
                    header_height_px,
                );
                window.header_frame = dock_header_frame(&header_nodes);
                window.tab_frames = dock_tab_frames(&header_nodes, &window.tabs);
                window.header_nodes = header_nodes;
                let content_height =
                    (window.frame.height - window.header_frame.height - 1.0).max(0.0);
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
