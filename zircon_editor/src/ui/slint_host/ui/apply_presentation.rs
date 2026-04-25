use super::pane_data_conversion;
use crate::ui::layouts::common::model_rc;
use crate::ui::layouts::views as view_data;
use crate::ui::layouts::windows::workbench_host_window::{
    self as host_window, build_host_scene_data, build_native_floating_surface_data, frame_rect,
    ShellPresentation,
};
use crate::ui::slint_host::callback_dispatch::BuiltinHostRootShellFrames;
use crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_center_band_frame,
    resolve_root_document_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame, resolve_root_status_bar_frame,
    resolve_root_viewport_content_frame,
};
use crate::ui::slint_host::{self as slint_ui, HostWindowPresentationData, UiHostWindow};
use crate::ui::workbench::autolayout::{ShellRegionId, WorkbenchShellGeometry};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use slint::{ComponentHandle, Model, ModelRc};
use zircon_runtime::ui::layout::UiSize;

pub(crate) fn apply_presentation(
    ui: &UiHostWindow,
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
    ui_asset_panes: &std::collections::BTreeMap<
        String,
        crate::ui::asset_editor::UiAssetEditorPanePresentation,
    >,
    animation_panes: &std::collections::BTreeMap<
        String,
        crate::ui::animation_editor::AnimationEditorPanePresentation,
    >,
    runtime_diagnostics: Option<&zircon_runtime::core::diagnostics::RuntimeDiagnosticsSnapshot>,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) {
    let presentation = ShellPresentation::from_state(
        model,
        chrome,
        geometry,
        preset_names,
        active_preset_name,
        ui_asset_panes,
        animation_panes,
        runtime_diagnostics,
        floating_window_projection_bundle,
    );
    let document_pane_shows_viewport_toolbar =
        presentation.host_surface_data.document_pane.show_toolbar;
    let pane_surface_host = ui.global::<slint_ui::PaneSurfaceHostContext>();

    pane_surface_host.set_recent_projects(to_slint_recent_projects(
        &presentation.welcome.recent_projects,
    ));
    pane_surface_host
        .set_project_overview(to_slint_project_overview(&presentation.project_overview));
    pane_surface_host.set_activity_asset_tree_folders(to_slint_asset_folders(
        &presentation.activity.tree_folders,
    ));
    pane_surface_host.set_activity_asset_content_folders(to_slint_asset_folders(
        &presentation.activity.content_folders,
    ));
    pane_surface_host.set_activity_asset_content_items(to_slint_asset_items(
        &presentation.activity.content_items,
    ));
    pane_surface_host
        .set_activity_asset_selection(to_slint_asset_selection(&presentation.activity.selection));
    pane_surface_host.set_activity_asset_references(to_slint_asset_references(
        &presentation.activity.references,
    ));
    pane_surface_host
        .set_activity_asset_used_by(to_slint_asset_references(&presentation.activity.used_by));
    pane_surface_host.set_activity_asset_search_query(presentation.activity.search_query);
    pane_surface_host.set_activity_asset_kind_filter(presentation.activity.kind_filter);
    pane_surface_host.set_activity_asset_view_mode(presentation.activity.view_mode);
    pane_surface_host.set_activity_asset_utility_tab(presentation.activity.utility_tab);
    pane_surface_host
        .set_browser_asset_tree_folders(to_slint_asset_folders(&presentation.browser.tree_folders));
    pane_surface_host.set_browser_asset_content_folders(to_slint_asset_folders(
        &presentation.browser.content_folders,
    ));
    pane_surface_host
        .set_browser_asset_content_items(to_slint_asset_items(&presentation.browser.content_items));
    pane_surface_host
        .set_browser_asset_selection(to_slint_asset_selection(&presentation.browser.selection));
    pane_surface_host
        .set_browser_asset_references(to_slint_asset_references(&presentation.browser.references));
    pane_surface_host
        .set_browser_asset_used_by(to_slint_asset_references(&presentation.browser.used_by));
    pane_surface_host.set_browser_asset_search_query(presentation.browser.search_query);
    pane_surface_host.set_browser_asset_kind_filter(presentation.browser.kind_filter);
    pane_surface_host.set_browser_asset_view_mode(presentation.browser.view_mode);
    pane_surface_host.set_browser_asset_utility_tab(presentation.browser.utility_tab);

    let host_layout = host_window_layout(
        geometry,
        shared_root_frames,
        document_pane_shows_viewport_toolbar,
    );
    let host_scene_data = build_host_scene_data(
        &presentation.host_surface_data,
        &presentation.host_shell,
        &host_layout,
        &presentation.status_primary,
        chrome.inspector.is_some(),
        &chrome.project_overview,
    );
    let welcome_pane = project_welcome_pane(&presentation.welcome.pane, &host_scene_data);
    let native_floating_surface_data = build_native_floating_surface_data(
        &presentation.host_surface_data,
        &presentation.host_shell,
        &chrome.project_overview,
    );
    let host_presentation = HostWindowPresentationData {
        host_scene_data: to_slint_host_scene_data(&host_scene_data),
        native_floating_surface_data: to_slint_native_floating_surface_data(
            &native_floating_surface_data,
        ),
        host_shell: to_slint_host_shell(&presentation.host_shell),
        host_layout: to_slint_host_window_layout(&host_layout),
    };
    ui.set_host_presentation(host_presentation);
    pane_surface_host.set_welcome_pane(to_slint_welcome_pane(&welcome_pane));
    pane_surface_host.set_mesh_import_path(presentation.mesh_import_path);
}

fn host_window_layout(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    document_pane_shows_viewport_toolbar: bool,
) -> host_window::HostWindowLayoutData {
    host_window::HostWindowLayoutData {
        center_band_frame: frame_rect(resolve_root_center_band_frame(geometry, shared_root_frames)),
        status_bar_frame: frame_rect(resolve_root_status_bar_frame(geometry, shared_root_frames)),
        left_region_frame: frame_rect(resolve_root_left_region_frame(geometry, shared_root_frames)),
        document_region_frame: frame_rect(resolve_root_document_region_frame(
            geometry,
            shared_root_frames,
        )),
        right_region_frame: frame_rect(resolve_root_right_region_frame(
            geometry,
            shared_root_frames,
        )),
        bottom_region_frame: frame_rect(resolve_root_bottom_region_frame(
            geometry,
            shared_root_frames,
        )),
        left_splitter_frame: frame_rect(geometry.splitter_frame(ShellRegionId::Left)),
        right_splitter_frame: frame_rect(geometry.splitter_frame(ShellRegionId::Right)),
        bottom_splitter_frame: frame_rect(geometry.splitter_frame(ShellRegionId::Bottom)),
        viewport_content_frame: frame_rect(resolve_root_viewport_content_frame(
            geometry,
            shared_root_frames,
            document_pane_shows_viewport_toolbar,
        )),
    }
}

fn map_model_rc<T, U, F>(model: &ModelRc<T>, mut map: F) -> ModelRc<U>
where
    T: Clone + 'static,
    U: Clone + 'static,
    F: FnMut(T) -> U,
{
    model_rc(
        (0..model.row_count())
            .filter_map(|row| model.row_data(row))
            .map(&mut map)
            .collect(),
    )
}

fn to_slint_frame_rect(frame: &host_window::FrameRect) -> slint_ui::FrameRect {
    slint_ui::FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn to_slint_tab_data(tab: host_window::TabData) -> slint_ui::TabData {
    slint_ui::TabData {
        id: tab.id,
        slot: tab.slot,
        title: tab.title,
        icon_key: tab.icon_key,
        active: tab.active,
        closeable: tab.closeable,
    }
}

fn to_slint_tabs(tabs: &ModelRc<host_window::TabData>) -> ModelRc<slint_ui::TabData> {
    map_model_rc(tabs, to_slint_tab_data)
}

fn to_slint_floating_window_data(
    window: host_window::FloatingWindowData,
    header_height_px: f32,
) -> slint_ui::FloatingWindowData {
    let pane_size = host_window::PaneContentSize::new(
        window.frame.width,
        (window.frame.height - header_height_px).max(0.0),
    );
    slint_ui::FloatingWindowData {
        window_id: window.window_id,
        title: window.title,
        frame: to_slint_frame_rect(&window.frame),
        target_group: window.target_group,
        left_edge_target_group: window.left_edge_target_group,
        right_edge_target_group: window.right_edge_target_group,
        top_edge_target_group: window.top_edge_target_group,
        bottom_edge_target_group: window.bottom_edge_target_group,
        focus_target_id: window.focus_target_id,
        tabs: to_slint_tabs(&window.tabs),
        active_pane: to_slint_pane(window.active_pane, pane_size),
    }
}

fn to_slint_floating_windows(
    windows: &ModelRc<host_window::FloatingWindowData>,
    header_height_px: f32,
) -> ModelRc<slint_ui::FloatingWindowData> {
    map_model_rc(windows, |window| {
        to_slint_floating_window_data(window, header_height_px)
    })
}

fn to_slint_new_project_form(form: &view_data::NewProjectFormData) -> slint_ui::NewProjectFormData {
    slint_ui::NewProjectFormData {
        project_name: form.project_name.clone(),
        location: form.location.clone(),
        project_path_preview: form.project_path_preview.clone(),
        template_label: form.template_label.clone(),
        validation_message: form.validation_message.clone(),
        can_create: form.can_create,
        can_open_existing: form.can_open_existing,
        browse_supported: form.browse_supported,
    }
}

fn to_slint_template_frame(
    frame: &view_data::ViewTemplateFrameData,
) -> slint_ui::TemplateNodeFrameData {
    slint_ui::TemplateNodeFrameData {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn to_slint_template_node(
    data: &view_data::ViewTemplateNodeData,
) -> slint_ui::TemplatePaneNodeData {
    slint_ui::TemplatePaneNodeData {
        node_id: data.node_id.clone(),
        control_id: data.control_id.clone(),
        role: data.role.clone(),
        text: data.text.clone(),
        dispatch_kind: data.dispatch_kind.clone(),
        action_id: data.action_id.clone(),
        surface_variant: data.surface_variant.clone(),
        text_tone: data.text_tone.clone(),
        button_variant: data.button_variant.clone(),
        font_size: data.font_size,
        font_weight: data.font_weight,
        text_align: data.text_align.clone(),
        overflow: data.overflow.clone(),
        corner_radius: data.corner_radius,
        border_width: data.border_width,
        frame: to_slint_template_frame(&data.frame),
    }
}

fn to_slint_welcome_pane(pane: &view_data::WelcomePaneData) -> slint_ui::WelcomePaneData {
    slint_ui::WelcomePaneData {
        nodes: map_model_rc(&pane.nodes, |node| to_slint_template_node(&node)),
        title: pane.title.clone(),
        subtitle: pane.subtitle.clone(),
        status_message: pane.status_message.clone(),
        form: to_slint_new_project_form(&pane.form),
    }
}

fn project_welcome_pane(
    pane: &view_data::WelcomePaneData,
    scene: &host_window::HostWindowSceneData,
) -> view_data::WelcomePaneData {
    let mut pane = pane.clone();
    if let Some(size) = resolve_visible_welcome_pane_size(scene) {
        pane.nodes = view_data::welcome_pane_nodes(size);
    }
    pane
}

fn resolve_visible_welcome_pane_size(scene: &host_window::HostWindowSceneData) -> Option<UiSize> {
    if scene.document_dock.pane.kind.as_str() == "Welcome" {
        return Some(UiSize::new(
            scene.document_dock.region_frame.width.max(0.0),
            dock_content_height(
                scene.document_dock.region_frame.height,
                scene.document_dock.header_height_px,
            ),
        ));
    }

    if scene.left_dock.pane.kind.as_str() == "Welcome" {
        return Some(UiSize::new(
            scene.left_dock.panel_width_px.max(0.0),
            dock_content_height(
                scene.left_dock.region_frame.height,
                scene.left_dock.panel_header_height_px,
            ),
        ));
    }

    if scene.right_dock.pane.kind.as_str() == "Welcome" {
        return Some(UiSize::new(
            scene.right_dock.panel_width_px.max(0.0),
            dock_content_height(
                scene.right_dock.region_frame.height,
                scene.right_dock.panel_header_height_px,
            ),
        ));
    }

    if scene.bottom_dock.pane.kind.as_str() == "Welcome" {
        return Some(UiSize::new(
            scene.bottom_dock.region_frame.width.max(0.0),
            dock_content_height(
                scene.bottom_dock.region_frame.height,
                scene.bottom_dock.header_height_px,
            ),
        ));
    }

    (0..scene.floating_layer.floating_windows.row_count())
        .filter_map(|row| scene.floating_layer.floating_windows.row_data(row))
        .find_map(|window| {
            (window.active_pane.kind.as_str() == "Welcome").then(|| {
                UiSize::new(
                    window.frame.width.max(0.0),
                    (window.frame.height - scene.floating_layer.header_height_px).max(0.0),
                )
            })
        })
}

fn dock_content_height(region_height: f32, header_height: f32) -> f32 {
    (region_height - header_height - 1.0).max(0.0)
}

fn to_slint_recent_project(data: view_data::RecentProjectData) -> slint_ui::RecentProjectData {
    slint_ui::RecentProjectData {
        display_name: data.display_name,
        path: data.path,
        last_opened_label: data.last_opened_label,
        status_label: data.status_label,
        invalid: data.invalid,
    }
}

fn to_slint_recent_projects(
    data: &ModelRc<view_data::RecentProjectData>,
) -> ModelRc<slint_ui::RecentProjectData> {
    map_model_rc(data, to_slint_recent_project)
}

fn to_slint_asset_folder(data: view_data::AssetFolderData) -> slint_ui::AssetFolderData {
    slint_ui::AssetFolderData {
        id: data.id,
        name: data.name,
        count: data.count,
        depth: data.depth,
        selected: data.selected,
    }
}

fn to_slint_asset_folders(
    data: &ModelRc<view_data::AssetFolderData>,
) -> ModelRc<slint_ui::AssetFolderData> {
    map_model_rc(data, to_slint_asset_folder)
}

fn to_slint_asset_item(data: view_data::AssetItemData) -> slint_ui::AssetItemData {
    slint_ui::AssetItemData {
        uuid: data.uuid,
        locator: data.locator,
        name: data.name,
        file_name: data.file_name,
        kind: data.kind,
        extension: data.extension,
        dirty: data.dirty,
        has_error: data.has_error,
        has_preview: data.has_preview,
        state: data.state,
        revision: data.revision,
        selected: data.selected,
        preview: data.preview,
    }
}

fn to_slint_asset_items(
    data: &ModelRc<view_data::AssetItemData>,
) -> ModelRc<slint_ui::AssetItemData> {
    map_model_rc(data, to_slint_asset_item)
}

fn to_slint_asset_reference(data: view_data::AssetReferenceData) -> slint_ui::AssetReferenceData {
    slint_ui::AssetReferenceData {
        uuid: data.uuid,
        locator: data.locator,
        name: data.name,
        kind: data.kind,
        known_project_asset: data.known_project_asset,
    }
}

fn to_slint_asset_references(
    data: &ModelRc<view_data::AssetReferenceData>,
) -> ModelRc<slint_ui::AssetReferenceData> {
    map_model_rc(data, to_slint_asset_reference)
}

fn to_slint_asset_selection(data: &view_data::AssetSelectionData) -> slint_ui::AssetSelectionData {
    slint_ui::AssetSelectionData {
        uuid: data.uuid.clone(),
        name: data.name.clone(),
        locator: data.locator.clone(),
        kind: data.kind.clone(),
        meta_path: data.meta_path.clone(),
        adapter_key: data.adapter_key.clone(),
        state: data.state.clone(),
        revision: data.revision.clone(),
        diagnostics: data.diagnostics.clone(),
        has_preview: data.has_preview,
        preview: data.preview.clone(),
    }
}

fn to_slint_scene_viewport_chrome(
    data: &view_data::SceneViewportChromeData,
) -> slint_ui::SceneViewportChromeData {
    slint_ui::SceneViewportChromeData {
        tool: data.tool.clone(),
        transform_space: data.transform_space.clone(),
        projection_mode: data.projection_mode.clone(),
        view_orientation: data.view_orientation.clone(),
        display_mode: data.display_mode.clone(),
        grid_mode: data.grid_mode.clone(),
        gizmos_enabled: data.gizmos_enabled,
        preview_lighting: data.preview_lighting,
        preview_skybox: data.preview_skybox,
        translate_snap: data.translate_snap,
        rotate_snap_deg: data.rotate_snap_deg,
        scale_snap: data.scale_snap,
        translate_snap_label: data.translate_snap_label.clone(),
        rotate_snap_label: data.rotate_snap_label.clone(),
        scale_snap_label: data.scale_snap_label.clone(),
    }
}

fn to_slint_animation_editor_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> slint_ui::AnimationEditorPaneData {
    pane_data_conversion::to_slint_animation_editor_pane_from_host_pane(data, pane_size)
}

fn to_slint_assets_activity_pane(
    data: host_window::AssetsActivityPaneViewData,
) -> slint_ui::AssetsActivityPaneData {
    pane_data_conversion::to_slint_assets_activity_pane(data)
}

fn to_slint_hierarchy_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> slint_ui::HierarchyPaneData {
    pane_data_conversion::to_slint_hierarchy_pane_from_host_pane(data, pane_size)
}

fn to_slint_inspector_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> slint_ui::InspectorPaneData {
    pane_data_conversion::to_slint_inspector_pane_from_host_pane(data, pane_size)
}

fn to_slint_console_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> slint_ui::ConsolePaneData {
    pane_data_conversion::to_slint_console_pane_from_host_pane(data, pane_size)
}

fn to_slint_project_overview_pane(
    data: host_window::ProjectOverviewPaneViewData,
) -> slint_ui::ProjectOverviewPaneData {
    pane_data_conversion::to_slint_project_overview_pane(data)
}

fn to_slint_ui_asset_pane(
    data: crate::ui::asset_editor::UiAssetEditorPanePresentation,
) -> slint_ui::UiAssetEditorPaneData {
    pane_data_conversion::to_slint_ui_asset_pane(data)
}

fn to_slint_pane(
    data: host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> slint_ui::PaneData {
    let hierarchy = to_slint_hierarchy_pane(&data, pane_size);
    let inspector = to_slint_inspector_pane(&data, pane_size);
    let console = to_slint_console_pane(&data, pane_size);
    let animation = to_slint_animation_editor_pane(&data, pane_size);

    slint_ui::PaneData {
        id: data.id,
        slot: data.slot,
        kind: data.kind,
        title: data.title,
        icon_key: data.icon_key,
        subtitle: data.subtitle,
        info: data.info,
        show_empty: data.show_empty,
        empty_title: data.empty_title,
        empty_body: data.empty_body,
        primary_action_label: data.primary_action_label,
        primary_action_id: data.primary_action_id,
        secondary_action_label: data.secondary_action_label,
        secondary_action_id: data.secondary_action_id,
        secondary_hint: data.secondary_hint,
        show_toolbar: data.show_toolbar,
        viewport: to_slint_scene_viewport_chrome(&data.viewport),
        hierarchy,
        inspector,
        console,
        assets_activity: to_slint_assets_activity_pane(data.body_compat.assets_activity),
        asset_browser: pane_data_conversion::to_slint_asset_browser_pane(
            data.body_compat.asset_browser,
        ),
        project_overview: to_slint_project_overview_pane(data.body_compat.project_overview),
        ui_asset: to_slint_ui_asset_pane(data.body_compat.ui_asset),
        animation,
    }
}

fn to_slint_project_overview(
    overview: &host_window::ProjectOverviewData,
) -> slint_ui::ProjectOverviewData {
    slint_ui::ProjectOverviewData {
        project_name: overview.project_name.clone(),
        project_root: overview.project_root.clone(),
        assets_root: overview.assets_root.clone(),
        library_root: overview.library_root.clone(),
        default_scene_uri: overview.default_scene_uri.clone(),
        catalog_revision: overview.catalog_revision.clone(),
        folder_count: overview.folder_count.clone(),
        asset_count: overview.asset_count.clone(),
    }
}

fn to_slint_host_window_layout(
    layout: &host_window::HostWindowLayoutData,
) -> slint_ui::HostWindowLayoutData {
    slint_ui::HostWindowLayoutData {
        center_band_frame: to_slint_frame_rect(&layout.center_band_frame),
        status_bar_frame: to_slint_frame_rect(&layout.status_bar_frame),
        left_region_frame: to_slint_frame_rect(&layout.left_region_frame),
        document_region_frame: to_slint_frame_rect(&layout.document_region_frame),
        right_region_frame: to_slint_frame_rect(&layout.right_region_frame),
        bottom_region_frame: to_slint_frame_rect(&layout.bottom_region_frame),
        left_splitter_frame: to_slint_frame_rect(&layout.left_splitter_frame),
        right_splitter_frame: to_slint_frame_rect(&layout.right_splitter_frame),
        bottom_splitter_frame: to_slint_frame_rect(&layout.bottom_splitter_frame),
        viewport_content_frame: to_slint_frame_rect(&layout.viewport_content_frame),
    }
}

fn to_slint_host_shell(shell: &host_window::HostWindowShellData) -> slint_ui::HostWindowShellData {
    slint_ui::HostWindowShellData {
        project_path: shell.project_path.clone(),
        status_secondary: shell.status_secondary.clone(),
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
        shell_min_width_px: shell.shell_min_width_px,
        shell_min_height_px: shell.shell_min_height_px,
        native_floating_window_mode: shell.native_floating_window_mode,
        native_floating_window_id: shell.native_floating_window_id.clone(),
        native_window_title: shell.native_window_title.clone(),
        native_window_bounds: to_slint_frame_rect(&shell.native_window_bounds),
    }
}

fn to_slint_metrics(
    metrics: &host_window::HostWindowSurfaceMetricsData,
) -> slint_ui::HostWindowSurfaceMetricsData {
    slint_ui::HostWindowSurfaceMetricsData {
        outer_margin_px: metrics.outer_margin_px,
        rail_width_px: metrics.rail_width_px,
        top_bar_height_px: metrics.top_bar_height_px,
        host_bar_height_px: metrics.host_bar_height_px,
        panel_header_height_px: metrics.panel_header_height_px,
        document_header_height_px: metrics.document_header_height_px,
    }
}

fn to_slint_orchestration(
    orchestration: &host_window::HostWindowSurfaceOrchestrationData,
) -> slint_ui::HostWindowSurfaceOrchestrationData {
    slint_ui::HostWindowSurfaceOrchestrationData {
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
        left_tab_origin_x_px: orchestration.left_tab_origin_x_px,
        left_tab_origin_y_px: orchestration.left_tab_origin_y_px,
        document_tab_origin_x_px: orchestration.document_tab_origin_x_px,
        document_tab_origin_y_px: orchestration.document_tab_origin_y_px,
        right_tab_origin_x_px: orchestration.right_tab_origin_x_px,
        right_tab_origin_y_px: orchestration.right_tab_origin_y_px,
        bottom_tab_origin_x_px: orchestration.bottom_tab_origin_x_px,
        bottom_tab_origin_y_px: orchestration.bottom_tab_origin_y_px,
    }
}

fn to_slint_menu_chrome(menu: &host_window::HostMenuChromeData) -> slint_ui::HostMenuChromeData {
    slint_ui::HostMenuChromeData {
        outer_margin_px: menu.outer_margin_px,
        top_bar_height_px: menu.top_bar_height_px,
        save_project_enabled: menu.save_project_enabled,
        undo_enabled: menu.undo_enabled,
        redo_enabled: menu.redo_enabled,
        delete_enabled: menu.delete_enabled,
        preset_names: menu.preset_names.clone(),
        active_preset_name: menu.active_preset_name.clone(),
        resolved_preset_name: menu.resolved_preset_name.clone(),
    }
}

fn to_slint_page_chrome(page: &host_window::HostPageChromeData) -> slint_ui::HostPageChromeData {
    slint_ui::HostPageChromeData {
        top_bar_height_px: page.top_bar_height_px,
        host_bar_height_px: page.host_bar_height_px,
        tabs: to_slint_tabs(&page.tabs),
        project_path: page.project_path.clone(),
    }
}

fn to_slint_status_bar(status_bar: &host_window::HostStatusBarData) -> slint_ui::HostStatusBarData {
    slint_ui::HostStatusBarData {
        status_bar_frame: to_slint_frame_rect(&status_bar.status_bar_frame),
        status_primary: status_bar.status_primary.clone(),
        status_secondary: status_bar.status_secondary.clone(),
        viewport_label: status_bar.viewport_label.clone(),
    }
}

fn to_slint_resize_layer(
    resize_layer: &host_window::HostResizeLayerData,
) -> slint_ui::HostResizeLayerData {
    slint_ui::HostResizeLayerData {
        left_splitter_frame: to_slint_frame_rect(&resize_layer.left_splitter_frame),
        right_splitter_frame: to_slint_frame_rect(&resize_layer.right_splitter_frame),
        bottom_splitter_frame: to_slint_frame_rect(&resize_layer.bottom_splitter_frame),
    }
}

fn to_slint_drag_overlay(
    overlay: &host_window::HostTabDragOverlayData,
) -> slint_ui::HostTabDragOverlayData {
    slint_ui::HostTabDragOverlayData {
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

fn to_slint_side_dock(
    dock: &host_window::HostSideDockSurfaceData,
) -> slint_ui::HostSideDockSurfaceData {
    let pane_size = host_window::PaneContentSize::new(
        dock.panel_width_px,
        dock_content_height(dock.region_frame.height, dock.panel_header_height_px),
    );
    slint_ui::HostSideDockSurfaceData {
        region_frame: to_slint_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        rail_before_panel: dock.rail_before_panel,
        tabs: to_slint_tabs(&dock.tabs),
        pane: to_slint_pane(dock.pane.clone(), pane_size),
        rail_width_px: dock.rail_width_px,
        panel_width_px: dock.panel_width_px,
        panel_header_height_px: dock.panel_header_height_px,
        tab_origin_x_px: dock.tab_origin_x_px,
        tab_origin_y_px: dock.tab_origin_y_px,
    }
}

fn to_slint_document_dock(
    dock: &host_window::HostDocumentDockSurfaceData,
) -> slint_ui::HostDocumentDockSurfaceData {
    let pane_size = host_window::PaneContentSize::new(
        dock.region_frame.width,
        dock_content_height(dock.region_frame.height, dock.header_height_px),
    );
    slint_ui::HostDocumentDockSurfaceData {
        region_frame: to_slint_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        tabs: to_slint_tabs(&dock.tabs),
        pane: to_slint_pane(dock.pane.clone(), pane_size),
        header_height_px: dock.header_height_px,
        tab_origin_x_px: dock.tab_origin_x_px,
        tab_origin_y_px: dock.tab_origin_y_px,
    }
}

fn to_slint_bottom_dock(
    dock: &host_window::HostBottomDockSurfaceData,
) -> slint_ui::HostBottomDockSurfaceData {
    let pane_size = host_window::PaneContentSize::new(
        dock.region_frame.width,
        dock_content_height(dock.region_frame.height, dock.header_height_px),
    );
    slint_ui::HostBottomDockSurfaceData {
        region_frame: to_slint_frame_rect(&dock.region_frame),
        surface_key: dock.surface_key.clone(),
        tabs: to_slint_tabs(&dock.tabs),
        pane: to_slint_pane(dock.pane.clone(), pane_size),
        expanded: dock.expanded,
        header_height_px: dock.header_height_px,
        tab_origin_x_px: dock.tab_origin_x_px,
        tab_origin_y_px: dock.tab_origin_y_px,
    }
}

fn to_slint_floating_layer(
    layer: &host_window::HostFloatingWindowLayerData,
) -> slint_ui::HostFloatingWindowLayerData {
    slint_ui::HostFloatingWindowLayerData {
        floating_windows: to_slint_floating_windows(
            &layer.floating_windows,
            layer.header_height_px,
        ),
        header_height_px: layer.header_height_px,
    }
}

pub(super) fn to_slint_host_scene_data(
    scene: &host_window::HostWindowSceneData,
) -> slint_ui::HostWindowSceneData {
    slint_ui::HostWindowSceneData {
        layout: to_slint_host_window_layout(&scene.layout),
        metrics: to_slint_metrics(&scene.metrics),
        orchestration: to_slint_orchestration(&scene.orchestration),
        menu_chrome: to_slint_menu_chrome(&scene.menu_chrome),
        page_chrome: to_slint_page_chrome(&scene.page_chrome),
        status_bar: to_slint_status_bar(&scene.status_bar),
        resize_layer: to_slint_resize_layer(&scene.resize_layer),
        drag_overlay: to_slint_drag_overlay(&scene.drag_overlay),
        left_dock: to_slint_side_dock(&scene.left_dock),
        document_dock: to_slint_document_dock(&scene.document_dock),
        right_dock: to_slint_side_dock(&scene.right_dock),
        bottom_dock: to_slint_bottom_dock(&scene.bottom_dock),
        floating_layer: to_slint_floating_layer(&scene.floating_layer),
    }
}

fn to_slint_native_floating_surface_data(
    surface: &host_window::HostNativeFloatingWindowSurfaceData,
) -> slint_ui::HostNativeFloatingWindowSurfaceData {
    slint_ui::HostNativeFloatingWindowSurfaceData {
        floating_windows: to_slint_floating_windows(
            &surface.floating_windows,
            surface.header_height_px,
        ),
        native_floating_window_id: surface.native_floating_window_id.clone(),
        native_window_bounds: to_slint_frame_rect(&surface.native_window_bounds),
        header_height_px: surface.header_height_px,
    }
}
