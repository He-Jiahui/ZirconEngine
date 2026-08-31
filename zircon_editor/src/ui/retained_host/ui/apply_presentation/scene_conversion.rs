use super::*;

pub(super) fn to_host_contract_project_overview(
    overview: &host_window::ProjectOverviewData,
) -> host_contract::ProjectOverviewData {
    host_contract::ProjectOverviewData {
        project_name: overview.project_name.clone(),
        project_root: overview.project_root.clone(),
        assets_root: overview.assets_root.clone(),
        cache_root: overview.cache_root.clone(),
        default_scene_uri: overview.default_scene_uri.clone(),
        catalog_revision: overview.catalog_revision.clone(),
        folder_count: overview.folder_count.clone(),
        asset_count: overview.asset_count.clone(),
    }
}

pub(in crate::ui::retained_host::ui) fn to_host_contract_host_window_layout(
    layout: &host_window::HostWindowLayoutData,
) -> host_contract::HostWindowLayoutData {
    host_contract::HostWindowLayoutData {
        center_band_frame: to_host_contract_frame_rect(&layout.center_band_frame),
        status_bar_frame: to_host_contract_frame_rect(&layout.status_bar_frame),
        left_region_frame: to_host_contract_frame_rect(&layout.left_region_frame),
        document_region_frame: to_host_contract_frame_rect(&layout.document_region_frame),
        right_region_frame: to_host_contract_frame_rect(&layout.right_region_frame),
        bottom_region_frame: to_host_contract_frame_rect(&layout.bottom_region_frame),
        left_splitter_frame: to_host_contract_frame_rect(&layout.left_splitter_frame),
        right_splitter_frame: to_host_contract_frame_rect(&layout.right_splitter_frame),
        bottom_splitter_frame: to_host_contract_frame_rect(&layout.bottom_splitter_frame),
        viewport_content_frame: to_host_contract_frame_rect(&layout.viewport_content_frame),
    }
}

pub(in crate::ui::retained_host::ui) fn to_host_contract_host_shell(
    shell: &host_window::HostWindowShellData,
) -> host_contract::HostWindowShellData {
    host_contract::HostWindowShellData {
        project_path: shell.project_path.clone(),
        status_secondary: shell.status_secondary.clone(),
        debug_refresh_rate: shell.debug_refresh_rate.clone(),
        viewport_label: shell.viewport_label.clone(),
        drawers_visible: shell.drawers_visible,
        left_expanded: shell.left_expanded,
        right_expanded: shell.right_expanded,
        bottom_expanded: shell.bottom_expanded,
        save_project_enabled: shell.save_project_enabled,
        undo_enabled: shell.undo_enabled,
        redo_enabled: shell.redo_enabled,
        preset_names: shell.preset_names.clone(),
        active_preset_name: shell.active_preset_name.clone(),
        skin_id: shell.skin_id.clone(),
        panel_preset_id: shell.panel_preset_id.clone(),
        shell_preset_id: shell.shell_preset_id.clone(),
        window_model_preset_id: shell.window_model_preset_id.clone(),
        shell_min_width_px: shell.shell_min_width_px,
        shell_min_height_px: shell.shell_min_height_px,
        native_floating_window_mode: shell.native_floating_window_mode,
        native_floating_window_id: shell.native_floating_window_id.clone(),
        native_surface_tree_id: shell.native_surface_tree_id.clone(),
        native_window_title: shell.native_window_title.clone(),
        native_window_bounds: to_host_contract_frame_rect(&shell.native_window_bounds),
    }
}

pub(super) fn to_host_contract_metrics(
    metrics: &host_window::HostWindowSurfaceMetricsData,
) -> host_contract::HostWindowSurfaceMetricsData {
    host_contract::HostWindowSurfaceMetricsData {
        outer_margin_px: metrics.outer_margin_px,
        rail_width_px: metrics.rail_width_px,
        top_bar_height_px: metrics.top_bar_height_px,
        host_bar_height_px: metrics.host_bar_height_px,
        panel_header_height_px: metrics.panel_header_height_px,
        document_header_height_px: metrics.document_header_height_px,
    }
}

pub(super) fn to_host_contract_orchestration(
    orchestration: &host_window::HostWindowSurfaceOrchestrationData,
) -> host_contract::HostWindowSurfaceOrchestrationData {
    host_contract::HostWindowSurfaceOrchestrationData {
        left_rail_width_px: orchestration.left_rail_width_px,
        right_rail_width_px: orchestration.right_rail_width_px,
        left_stack_width_px: orchestration.left_stack_width_px,
        right_stack_width_px: orchestration.right_stack_width_px,
        left_panel_width_px: orchestration.left_panel_width_px,
        right_panel_width_px: orchestration.right_panel_width_px,
        bottom_panel_height_px: orchestration.bottom_panel_height_px,
        main_content_y_px: orchestration.main_content_y_px,
        document_zone_x_px: orchestration.document_zone_x_px,
        right_stack_x_px: orchestration.right_stack_x_px,
        bottom_panel_y_px: orchestration.bottom_panel_y_px,
    }
}

pub(super) fn to_host_contract_chrome_control_frame(
    data: host_window::HostChromeControlFrameData,
) -> host_contract::HostChromeControlFrameData {
    host_contract::HostChromeControlFrameData {
        control_id: data.control_id,
        frame: to_host_contract_frame_rect(&data.frame),
    }
}

pub(super) fn to_host_contract_chrome_tab(
    data: host_window::HostChromeTabData,
) -> host_contract::HostChromeTabData {
    host_contract::HostChromeTabData {
        control_id: data.control_id,
        tab: to_host_contract_tab_data(data.tab),
        frame: to_host_contract_frame_rect(&data.frame),
        close_frame: to_host_contract_frame_rect(&data.close_frame),
    }
}

pub(super) fn to_host_contract_menu_chrome(
    menu: &host_window::HostMenuChromeData,
) -> host_contract::HostMenuChromeData {
    host_contract::HostMenuChromeData {
        outer_margin_px: menu.outer_margin_px,
        top_bar_height_px: menu.top_bar_height_px,
        template_nodes: to_host_contract_template_nodes(&menu.template_nodes),
        menu_frames: map_model_rc(&menu.menu_frames, to_host_contract_chrome_control_frame),
        save_project_enabled: menu.save_project_enabled,
        undo_enabled: menu.undo_enabled,
        redo_enabled: menu.redo_enabled,
        delete_enabled: menu.delete_enabled,
        preset_names: menu.preset_names.clone(),
        active_preset_name: menu.active_preset_name.clone(),
        resolved_preset_name: menu.resolved_preset_name.clone(),
        menus: map_model_rc(&menu.menus, to_host_contract_menu_chrome_menu),
    }
}

pub(super) fn to_host_contract_menu_chrome_menu(
    menu: host_window::HostMenuChromeMenuData,
) -> host_contract::HostMenuChromeMenuData {
    host_contract::HostMenuChromeMenuData {
        label: menu.label,
        popup_width_px: menu.popup_width_px,
        popup_height_px: menu.popup_height_px,
        popup_nodes: to_host_contract_template_nodes(&menu.popup_nodes),
        items: map_model_rc(&menu.items, to_host_contract_menu_chrome_item),
    }
}

pub(super) fn to_host_contract_menu_chrome_item(
    item: host_window::HostMenuChromeItemData,
) -> host_contract::HostMenuChromeItemData {
    host_contract::HostMenuChromeItemData {
        label: item.label,
        shortcut: item.shortcut,
        action_id: item.action_id,
        enabled: item.enabled,
        children: map_model_rc(&item.children, to_host_contract_menu_chrome_item),
    }
}

pub(super) fn to_host_contract_page_chrome(
    page: &host_window::HostPageChromeData,
) -> host_contract::HostPageChromeData {
    host_contract::HostPageChromeData {
        top_bar_height_px: page.top_bar_height_px,
        host_bar_height_px: page.host_bar_height_px,
        template_nodes: to_host_contract_template_nodes(&page.template_nodes),
        tab_row_frame: to_host_contract_frame_rect(&page.tab_row_frame),
        project_path_frame: to_host_contract_frame_rect(&page.project_path_frame),
        overflow_frame: to_host_contract_frame_rect(&page.overflow_frame),
        overflow_hidden_tab_indices: page.overflow_hidden_tab_indices.clone(),
        overflow_widest_title_width_px: page.overflow_widest_title_width_px,
        tab_frames: map_model_rc(&page.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&page.tabs),
        project_path: page.project_path.clone(),
    }
}

pub(super) fn to_host_contract_status_bar(
    status_bar: &host_window::HostStatusBarData,
) -> host_contract::HostStatusBarData {
    host_contract::HostStatusBarData {
        status_bar_frame: to_host_contract_frame_rect(&status_bar.status_bar_frame),
        template_nodes: to_host_contract_template_nodes(&status_bar.template_nodes),
        status_primary: status_bar.status_primary.clone(),
        status_secondary: status_bar.status_secondary.clone(),
        viewport_label: status_bar.viewport_label.clone(),
    }
}

pub(super) fn to_host_contract_resize_layer(
    resize_layer: &host_window::HostResizeLayerData,
) -> host_contract::HostResizeLayerData {
    host_contract::HostResizeLayerData {
        left_splitter_frame: to_host_contract_frame_rect(&resize_layer.left_splitter_frame),
        right_splitter_frame: to_host_contract_frame_rect(&resize_layer.right_splitter_frame),
        bottom_splitter_frame: to_host_contract_frame_rect(&resize_layer.bottom_splitter_frame),
    }
}

pub(super) fn to_host_contract_drag_overlay(
    overlay: &host_window::HostTabDragOverlayData,
) -> host_contract::HostTabDragOverlayData {
    host_contract::HostTabDragOverlayData {
        left_drop_enabled: overlay.left_drop_enabled,
        right_drop_enabled: overlay.right_drop_enabled,
        bottom_drop_enabled: overlay.bottom_drop_enabled,
        left_drop_width_px: overlay.left_drop_width_px,
        right_drop_width_px: overlay.right_drop_width_px,
        bottom_drop_height_px: overlay.bottom_drop_height_px,
        main_content_y_px: overlay.main_content_y_px,
        main_content_height_px: overlay.main_content_height_px,
        document_zone_x_px: overlay.document_zone_x_px,
        document_zone_width_px: overlay.document_zone_width_px,
        bottom_drop_top_px: overlay.bottom_drop_top_px,
        drag_overlay_bottom_px: overlay.drag_overlay_bottom_px,
    }
}

pub(in crate::ui::retained_host::ui) fn to_host_contract_side_dock(
    dock: &host_window::HostSideDockSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
    console_projection_cache: &mut pane_data_conversion::ConsolePaneProjectionCache,
    module_plugins_projection_cache: &mut pane_data_conversion::ModulePluginsPaneProjectionCache,
) -> host_contract::HostSideDockSurfaceData {
    let pane_size = host_window::PaneContentSize::new(
        dock.panel_width_px,
        dock_content_height(dock.region_frame.height, dock.panel_header_height_px),
    );
    host_contract::HostSideDockSurfaceData {
        region_frame: to_host_contract_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        rail_before_panel: dock.rail_before_panel,
        rail_nodes: to_host_contract_template_nodes(&dock.rail_nodes),
        rail_button_frames: map_model_rc(
            &dock.rail_button_frames,
            to_host_contract_chrome_control_frame,
        ),
        rail_active_control_id: dock.rail_active_control_id.clone(),
        header_nodes: to_host_contract_template_nodes(&dock.header_nodes),
        header_frame: to_host_contract_frame_rect(&dock.header_frame),
        overflow_frame: to_host_contract_frame_rect(&dock.overflow_frame),
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane: to_host_contract_pane(
            dock.pane.clone(),
            pane_size,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        ),
        rail_width_px: dock.rail_width_px,
        panel_width_px: dock.panel_width_px,
        panel_header_height_px: dock.panel_header_height_px,
    }
}

pub(super) fn to_host_contract_document_dock(
    dock: &host_window::HostDocumentDockSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
    console_projection_cache: &mut pane_data_conversion::ConsolePaneProjectionCache,
    module_plugins_projection_cache: &mut pane_data_conversion::ModulePluginsPaneProjectionCache,
) -> host_contract::HostDocumentDockSurfaceData {
    let pane_size = host_window::PaneContentSize::new(
        dock.region_frame.width,
        dock_content_height(dock.region_frame.height, dock.header_height_px),
    );
    host_contract::HostDocumentDockSurfaceData {
        region_frame: to_host_contract_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        header_nodes: to_host_contract_template_nodes(&dock.header_nodes),
        header_frame: to_host_contract_frame_rect(&dock.header_frame),
        overflow_frame: to_host_contract_frame_rect(&dock.overflow_frame),
        subtitle_frame: to_host_contract_frame_rect(&dock.subtitle_frame),
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane: to_host_contract_pane(
            dock.pane.clone(),
            pane_size,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        ),
        header_height_px: dock.header_height_px,
    }
}

pub(in crate::ui::retained_host::ui) fn to_host_contract_bottom_dock(
    dock: &host_window::HostBottomDockSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
    console_projection_cache: &mut pane_data_conversion::ConsolePaneProjectionCache,
    module_plugins_projection_cache: &mut pane_data_conversion::ModulePluginsPaneProjectionCache,
) -> host_contract::HostBottomDockSurfaceData {
    let pane_size = host_window::PaneContentSize::new(
        dock.region_frame.width,
        dock_content_height(dock.region_frame.height, dock.header_height_px),
    );
    host_contract::HostBottomDockSurfaceData {
        region_frame: to_host_contract_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        header_nodes: to_host_contract_template_nodes(&dock.header_nodes),
        header_frame: to_host_contract_frame_rect(&dock.header_frame),
        overflow_frame: to_host_contract_frame_rect(&dock.overflow_frame),
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane: to_host_contract_pane(
            dock.pane.clone(),
            pane_size,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        ),
        expanded: dock.expanded,
        header_height_px: dock.header_height_px,
    }
}

pub(super) fn to_host_contract_floating_layer(
    layer: &host_window::HostFloatingWindowLayerData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
    console_projection_cache: &mut pane_data_conversion::ConsolePaneProjectionCache,
    module_plugins_projection_cache: &mut pane_data_conversion::ModulePluginsPaneProjectionCache,
) -> host_contract::HostFloatingWindowLayerData {
    host_contract::HostFloatingWindowLayerData {
        floating_windows: to_host_contract_floating_windows(
            &layer.floating_windows,
            layer.header_height_px,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        ),
        header_height_px: layer.header_height_px,
    }
}

#[cfg(test)]
pub(in crate::ui::retained_host::ui) fn to_host_contract_host_scene_data(
    scene: &host_window::HostWindowSceneData,
) -> host_contract::HostWindowSceneData {
    let mut console_projection_cache = pane_data_conversion::ConsolePaneProjectionCache::default();
    let mut module_plugins_projection_cache =
        pane_data_conversion::ModulePluginsPaneProjectionCache::default();
    to_host_contract_host_scene_data_with_runtime(
        scene,
        None,
        None,
        "",
        &mut console_projection_cache,
        &mut module_plugins_projection_cache,
    )
}

pub(super) fn to_host_contract_host_scene_data_with_runtime(
    scene: &host_window::HostWindowSceneData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
    console_projection_cache: &mut pane_data_conversion::ConsolePaneProjectionCache,
    module_plugins_projection_cache: &mut pane_data_conversion::ModulePluginsPaneProjectionCache,
) -> host_contract::HostWindowSceneData {
    let layout = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_layout");
        to_host_contract_host_window_layout(&scene.layout)
    };
    let metrics = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_metrics");
        to_host_contract_metrics(&scene.metrics)
    };
    let orchestration = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_orchestration");
        to_host_contract_orchestration(&scene.orchestration)
    };
    let menu_chrome = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_menu_chrome");
        to_host_contract_menu_chrome(&scene.menu_chrome)
    };
    let page_chrome = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_page_chrome");
        to_host_contract_page_chrome(&scene.page_chrome)
    };
    let status_bar = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_status_bar");
        to_host_contract_status_bar(&scene.status_bar)
    };
    let resize_layer = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_resize_layer");
        to_host_contract_resize_layer(&scene.resize_layer)
    };
    let drag_overlay = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_drag_overlay");
        to_host_contract_drag_overlay(&scene.drag_overlay)
    };
    let left_dock = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_left_dock");
        to_host_contract_side_dock(
            &scene.left_dock,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        )
    };
    let document_dock = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_document_dock");
        to_host_contract_document_dock(
            &scene.document_dock,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        )
    };
    let right_dock = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_right_dock");
        to_host_contract_side_dock(
            &scene.right_dock,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        )
    };
    let bottom_dock = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_bottom_dock");
        to_host_contract_bottom_dock(
            &scene.bottom_dock,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        )
    };
    let floating_layer = {
        zircon_runtime::profile_scope!("editor", "retained_host", "convert_scene_floating_layer");
        to_host_contract_floating_layer(
            &scene.floating_layer,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        )
    };

    host_contract::HostWindowSceneData {
        layout,
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

pub(in crate::ui::retained_host::ui) fn to_host_contract_host_scene_geometry_with_retained_panes(
    scene: &host_window::HostWindowSceneData,
    current: &host_contract::HostWindowSceneData,
) -> host_contract::HostWindowSceneData {
    host_contract::HostWindowSceneData {
        layout: to_host_contract_host_window_layout(&scene.layout),
        metrics: to_host_contract_metrics(&scene.metrics),
        orchestration: to_host_contract_orchestration(&scene.orchestration),
        menu_chrome: to_host_contract_menu_chrome(&scene.menu_chrome),
        page_chrome: to_host_contract_page_chrome(&scene.page_chrome),
        status_bar: to_host_contract_status_bar(&scene.status_bar),
        resize_layer: to_host_contract_resize_layer(&scene.resize_layer),
        drag_overlay: to_host_contract_drag_overlay(&scene.drag_overlay),
        left_dock: to_host_contract_side_dock_geometry(
            &scene.left_dock,
            current.left_dock.pane.clone(),
        ),
        document_dock: to_host_contract_document_dock_geometry(
            &scene.document_dock,
            current.document_dock.pane.clone(),
        ),
        right_dock: to_host_contract_side_dock_geometry(
            &scene.right_dock,
            current.right_dock.pane.clone(),
        ),
        bottom_dock: to_host_contract_bottom_dock_geometry(
            &scene.bottom_dock,
            current.bottom_dock.pane.clone(),
        ),
        floating_layer: to_host_contract_floating_layer_geometry(
            &scene.floating_layer,
            &current.floating_layer,
        ),
    }
}

fn to_host_contract_floating_layer_geometry(
    layer: &host_window::HostFloatingWindowLayerData,
    current: &host_contract::HostFloatingWindowLayerData,
) -> host_contract::HostFloatingWindowLayerData {
    let floating_windows = map_model_rc(&layer.floating_windows, |window| {
        let retained = current
            .floating_windows
            .iter()
            .find(|candidate| candidate.window_id == window.window_id)
            .map(|candidate| candidate.active_pane.clone())
            .unwrap_or_default();
        host_contract::FloatingWindowData {
            window_id: window.window_id.clone(),
            title: window.title.clone(),
            frame: to_host_contract_frame_rect(&window.frame),
            header_nodes: to_host_contract_template_nodes(&window.header_nodes),
            header_frame: to_host_contract_frame_rect(&window.header_frame),
            overflow_frame: to_host_contract_frame_rect(&window.overflow_frame),
            tab_frames: map_model_rc(&window.tab_frames, to_host_contract_chrome_tab),
            target_group: window.target_group.clone(),
            left_edge_target_group: window.left_edge_target_group.clone(),
            right_edge_target_group: window.right_edge_target_group.clone(),
            top_edge_target_group: window.top_edge_target_group.clone(),
            bottom_edge_target_group: window.bottom_edge_target_group.clone(),
            focus_target_id: window.focus_target_id.clone(),
            tabs: to_host_contract_tabs(&window.tabs),
            active_pane: retained,
        }
    });
    host_contract::HostFloatingWindowLayerData {
        floating_windows,
        header_height_px: layer.header_height_px,
    }
}

pub(in crate::ui::retained_host::ui) fn to_host_contract_native_floating_surface_geometry_with_retained_panes(
    surface: &host_window::HostNativeFloatingWindowSurfaceData,
    current: &host_contract::HostNativeFloatingWindowSurfaceData,
) -> host_contract::HostNativeFloatingWindowSurfaceData {
    let floating_windows = map_model_rc(&surface.floating_windows, |window| {
        let retained = current
            .floating_windows
            .iter()
            .find(|candidate| candidate.window_id == window.window_id)
            .map(|candidate| candidate.active_pane.clone())
            .unwrap_or_default();
        host_contract::FloatingWindowData {
            window_id: window.window_id.clone(),
            title: window.title.clone(),
            frame: to_host_contract_frame_rect(&window.frame),
            header_nodes: to_host_contract_template_nodes(&window.header_nodes),
            header_frame: to_host_contract_frame_rect(&window.header_frame),
            overflow_frame: to_host_contract_frame_rect(&window.overflow_frame),
            tab_frames: map_model_rc(&window.tab_frames, to_host_contract_chrome_tab),
            target_group: window.target_group.clone(),
            left_edge_target_group: window.left_edge_target_group.clone(),
            right_edge_target_group: window.right_edge_target_group.clone(),
            top_edge_target_group: window.top_edge_target_group.clone(),
            bottom_edge_target_group: window.bottom_edge_target_group.clone(),
            focus_target_id: window.focus_target_id.clone(),
            tabs: to_host_contract_tabs(&window.tabs),
            active_pane: retained,
        }
    });
    host_contract::HostNativeFloatingWindowSurfaceData {
        floating_windows,
        native_floating_window_id: surface.native_floating_window_id.clone(),
        native_surface_tree_id: surface.native_surface_tree_id.clone(),
        native_window_bounds: to_host_contract_frame_rect(&surface.native_window_bounds),
        header_height_px: surface.header_height_px,
    }
}

fn to_host_contract_side_dock_geometry(
    dock: &host_window::HostSideDockSurfaceData,
    pane: host_contract::PaneData,
) -> host_contract::HostSideDockSurfaceData {
    host_contract::HostSideDockSurfaceData {
        region_frame: to_host_contract_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        rail_before_panel: dock.rail_before_panel,
        rail_nodes: to_host_contract_template_nodes(&dock.rail_nodes),
        rail_button_frames: map_model_rc(
            &dock.rail_button_frames,
            to_host_contract_chrome_control_frame,
        ),
        rail_active_control_id: dock.rail_active_control_id.clone(),
        header_nodes: to_host_contract_template_nodes(&dock.header_nodes),
        header_frame: to_host_contract_frame_rect(&dock.header_frame),
        overflow_frame: to_host_contract_frame_rect(&dock.overflow_frame),
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane,
        rail_width_px: dock.rail_width_px,
        panel_width_px: dock.panel_width_px,
        panel_header_height_px: dock.panel_header_height_px,
    }
}

fn to_host_contract_document_dock_geometry(
    dock: &host_window::HostDocumentDockSurfaceData,
    pane: host_contract::PaneData,
) -> host_contract::HostDocumentDockSurfaceData {
    host_contract::HostDocumentDockSurfaceData {
        region_frame: to_host_contract_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        header_nodes: to_host_contract_template_nodes(&dock.header_nodes),
        header_frame: to_host_contract_frame_rect(&dock.header_frame),
        overflow_frame: to_host_contract_frame_rect(&dock.overflow_frame),
        subtitle_frame: to_host_contract_frame_rect(&dock.subtitle_frame),
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane,
        header_height_px: dock.header_height_px,
    }
}

fn to_host_contract_bottom_dock_geometry(
    dock: &host_window::HostBottomDockSurfaceData,
    pane: host_contract::PaneData,
) -> host_contract::HostBottomDockSurfaceData {
    host_contract::HostBottomDockSurfaceData {
        region_frame: to_host_contract_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        header_nodes: to_host_contract_template_nodes(&dock.header_nodes),
        header_frame: to_host_contract_frame_rect(&dock.header_frame),
        overflow_frame: to_host_contract_frame_rect(&dock.overflow_frame),
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane,
        expanded: dock.expanded,
        header_height_px: dock.header_height_px,
    }
}

pub(super) fn to_host_contract_native_floating_surface_data_with_runtime(
    surface: &host_window::HostNativeFloatingWindowSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
    welcome: Option<&view_data::WelcomePresentation>,
    hierarchy_filter_query: &str,
    console_projection_cache: &mut pane_data_conversion::ConsolePaneProjectionCache,
    module_plugins_projection_cache: &mut pane_data_conversion::ModulePluginsPaneProjectionCache,
) -> host_contract::HostNativeFloatingWindowSurfaceData {
    host_contract::HostNativeFloatingWindowSurfaceData {
        floating_windows: to_host_contract_floating_windows(
            &surface.floating_windows,
            surface.header_height_px,
            component_showcase_runtime,
            welcome,
            hierarchy_filter_query,
            console_projection_cache,
            module_plugins_projection_cache,
        ),
        native_floating_window_id: surface.native_floating_window_id.clone(),
        native_surface_tree_id: surface.native_surface_tree_id.clone(),
        native_window_bounds: to_host_contract_frame_rect(&surface.native_window_bounds),
        header_height_px: surface.header_height_px,
    }
}
