use super::*;

/// Rebuilds only geometry-bearing scene data from the last stable domain scene.
/// Pane payloads, floating active panes, and menu item models remain cloned from
/// `current`; no pane projection functions are called on the resize path.
pub(crate) fn build_host_scene_geometry(
    current: &HostWindowSceneData,
    host_surface_data: &HostWindowSurfaceData,
    host_shell: &HostWindowShellData,
    host_layout: &HostWindowLayoutData,
    status_primary: &SharedString,
    floating_window_projection_bundle: &crate::ui::retained_host::floating_window_projection::FloatingWindowProjectionBundle,
) -> HostWindowSceneData {
    let mut scene = current.clone();
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

    let mut menu_chrome = scene.menu_chrome.clone();
    let menus = model_rc(
        (0..menu_chrome.menus.row_count())
            .filter_map(|row| menu_chrome.menus.row_data(row).map(|menu| (row, menu)))
            .map(|(index, mut menu)| {
                let fallback_popup_width_px = MENU_POPUP_WIDTHS_PX
                    .get(index)
                    .copied()
                    .unwrap_or(DEFAULT_MENU_POPUP_WIDTH_PX);
                menu.popup_width_px = content_measured_menu_popup_width(
                    fallback_popup_width_px,
                    shell_width,
                    menu.items
                        .iter()
                        .map(|item| (item.label.as_str(), item.shortcut.as_str())),
                    menu_popup_text_width,
                );
                menu
            })
            .collect(),
    );
    menu_chrome.outer_margin_px = metrics.outer_margin_px;
    menu_chrome.top_bar_height_px = metrics.top_bar_height_px;
    menu_chrome.template_nodes =
        menu_chrome_nodes(&menus, shell_width, metrics.top_bar_height_px + 1.0);
    menu_chrome.menu_frames = menu_control_frames(
        &menu_chrome.template_nodes,
        menus.row_count().max(MENU_SLOT_COUNT),
    );
    menu_chrome.save_project_enabled = host_shell.save_project_enabled;
    menu_chrome.undo_enabled = host_shell.undo_enabled;
    menu_chrome.redo_enabled = host_shell.redo_enabled;
    menu_chrome.active_preset_name = host_shell.active_preset_name.clone();
    menu_chrome.resolved_preset_name = resolved_preset_name;

    let page_template_nodes = page_chrome_nodes(
        &host_surface_data.host_tabs,
        &host_shell.project_path,
        &host_shell.shell_preset_id,
        shell_width,
        metrics.top_bar_height_px + 1.0 + metrics.host_bar_height_px,
    );
    let overflow_hidden_tab_indices =
        page_overflow_hidden_tab_indices(&page_template_nodes, &host_surface_data.host_tabs);
    let overflow_widest_title_width_px = overflow_hidden_tab_indices
        .iter()
        .filter_map(|page_index| host_surface_data.host_tabs.row_data(*page_index))
        .map(|tab| menu_popup_text_width(tab.title.as_str()))
        .fold(0.0_f32, f32::max);

    let status_template_nodes = status_bar_nodes(
        status_primary,
        &host_shell.status_secondary,
        &host_shell.viewport_label,
        &host_shell.skin_id,
        host_layout.status_bar_frame.width,
        host_layout.status_bar_frame.height,
    );
    let left_dock = rebuild_left_dock_geometry(
        &scene.left_dock,
        host_surface_data,
        host_shell,
        host_layout,
        &metrics,
        &orchestration,
    );
    let right_dock = rebuild_right_dock_geometry(
        &scene.right_dock,
        host_surface_data,
        host_shell,
        host_layout,
        &metrics,
        &orchestration,
    );
    let bottom_dock = rebuild_bottom_dock_geometry(
        &scene.bottom_dock,
        host_surface_data,
        host_shell,
        host_layout,
        &metrics,
    );
    let document_header_nodes = document_dock_header_nodes(
        &host_surface_data.document_tabs,
        &host_surface_data.document_pane.subtitle,
        &host_shell.panel_preset_id,
        host_layout.document_region_frame.width,
        metrics.document_header_height_px,
    );
    let document_content_height =
        (host_layout.document_region_frame.height - metrics.document_header_height_px - 1.0)
            .max(0.0);
    let document_dock = HostDocumentDockSurfaceData {
        region_frame: host_layout.document_region_frame.clone(),
        surface_key: "document".into(),
        header_nodes: document_header_nodes.clone(),
        header_frame: dock_header_frame(&document_header_nodes),
        subtitle_frame: dock_subtitle_frame(&document_header_nodes),
        content_frame: FrameRect {
            x: 0.0,
            y: metrics.document_header_height_px + 1.0,
            width: host_layout.document_region_frame.width,
            height: document_content_height,
        },
        tab_frames: dock_tab_frames(&document_header_nodes, &host_surface_data.document_tabs),
        tabs: host_surface_data.document_tabs.clone(),
        pane: scene.document_dock.pane,
        header_height_px: metrics.document_header_height_px,
    };
    let floating_windows = model_rc(
        (0..scene.floating_layer.floating_windows.row_count())
            .filter_map(|row| scene.floating_layer.floating_windows.row_data(row))
            .map(|mut window| {
                let window_id =
                    crate::ui::workbench::layout::MainPageId::new(window.window_id.as_str());
                if let Some(frame) = floating_window_projection_bundle.outer_frame(&window_id) {
                    window.frame = frame_rect(frame);
                }
                let surface_id = format!("host.floating.{}.dock.header", window.window_id.as_str());
                let header_nodes = floating_window_header_nodes(
                    &surface_id,
                    &window.tabs,
                    &window.title,
                    window.frame.width,
                    metrics.document_header_height_px,
                );
                window.header_frame = dock_header_frame(&header_nodes);
                window.tab_frames = dock_tab_frames(&header_nodes, &window.tabs);
                window.header_nodes = header_nodes;
                window
            })
            .collect(),
    );

    scene.layout = host_layout.clone();
    scene.metrics = metrics.clone();
    scene.orchestration = orchestration.clone();
    menu_chrome.menus = menus;
    scene.menu_chrome = menu_chrome;
    scene.page_chrome = HostPageChromeData {
        top_bar_height_px: metrics.top_bar_height_px,
        host_bar_height_px: metrics.host_bar_height_px,
        tab_row_frame: page_tab_row_frame(&page_template_nodes),
        project_path_frame: page_project_path_frame(&page_template_nodes),
        tab_frames: page_tab_frames(&page_template_nodes, &host_surface_data.host_tabs),
        tabs: host_surface_data.host_tabs.clone(),
        project_path: host_shell.project_path.clone(),
        overflow_frame: page_overflow_frame(&page_template_nodes),
        overflow_hidden_tab_indices,
        overflow_widest_title_width_px,
        template_nodes: page_template_nodes,
    };
    scene.status_bar = HostStatusBarData {
        status_bar_frame: host_layout.status_bar_frame.clone(),
        template_nodes: status_template_nodes,
        status_primary: status_primary.clone(),
        status_secondary: host_shell.status_secondary.clone(),
        viewport_label: host_shell.viewport_label.clone(),
    };
    scene.resize_layer = HostResizeLayerData {
        left_splitter_frame: host_layout.left_splitter_frame.clone(),
        right_splitter_frame: host_layout.right_splitter_frame.clone(),
        bottom_splitter_frame: host_layout.bottom_splitter_frame.clone(),
    };
    scene.drag_overlay = HostTabDragOverlayData {
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
    scene.left_dock = left_dock;
    scene.document_dock = document_dock;
    scene.right_dock = right_dock;
    scene.bottom_dock = bottom_dock;
    scene.floating_layer = HostFloatingWindowLayerData {
        floating_windows,
        header_height_px: metrics.document_header_height_px,
    };
    scene
}

#[cfg(test)]
mod tests {
    #[test]
    fn geometry_builder_does_not_invoke_semantic_pane_projection() {
        let source = include_str!("geometry.rs");
        let production = source
            .split("#[cfg(test)]")
            .next()
            .expect("geometry production source");
        let function = production
            .split("pub(crate) fn build_host_scene_geometry")
            .nth(1)
            .expect("geometry builder source");

        assert!(function.contains("current.clone()"));
        assert!(function.contains("rebuild_left_dock_geometry"));
        for forbidden in [
            "pane_with_host_owned_shell_layouts(",
            "pane_with_ui_asset_nodes(",
            "pane_with_hierarchy_projection(",
            "pane_with_inspector_projection(",
            "pane_with_assets_activity_projection(",
            "pane_with_asset_browser_projection(",
            "pane_with_project_overview_projection(",
            "pane_with_animation_projection(",
            "floating_windows_with_pane_shell_layouts(",
        ] {
            assert!(
                !function.contains(forbidden),
                "geometry builder must not invoke {forbidden}"
            );
        }
    }
}
