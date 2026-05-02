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
use crate::ui::slint_host::{self as host_contract, HostWindowPresentationData, UiHostWindow};
use crate::ui::template_runtime::EditorUiHostRuntime;
use crate::ui::workbench::autolayout::{ShellRegionId, WorkbenchShellGeometry};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::ui::workbench::snapshot::EditorChromeSnapshot;
use slint::{Model, ModelRc};
use zircon_runtime_interface::ui::layout::UiSize;

use super::template_node_conversion::to_host_contract_template_nodes;

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
    module_plugins: &host_window::ModulePluginsPaneViewData,
    shared_root_frames: Option<&BuiltinHostRootShellFrames>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
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
        module_plugins,
        floating_window_projection_bundle,
    );
    let document_pane_shows_viewport_toolbar =
        presentation.host_surface_data.document_pane.show_toolbar;
    let pane_surface_host = ui.global::<host_contract::PaneSurfaceHostContext>();

    pane_surface_host.set_recent_projects(to_host_contract_recent_projects(
        &presentation.welcome.recent_projects,
    ));
    pane_surface_host.set_project_overview(to_host_contract_project_overview(
        &presentation.project_overview,
    ));
    pane_surface_host.set_activity_asset_tree_folders(to_host_contract_asset_folders(
        &presentation.activity.tree_folders,
    ));
    pane_surface_host.set_activity_asset_content_folders(to_host_contract_asset_folders(
        &presentation.activity.content_folders,
    ));
    pane_surface_host.set_activity_asset_content_items(to_host_contract_asset_items(
        &presentation.activity.content_items,
    ));
    pane_surface_host.set_activity_asset_selection(to_host_contract_asset_selection(
        &presentation.activity.selection,
    ));
    pane_surface_host.set_activity_asset_references(to_host_contract_asset_references(
        &presentation.activity.references,
    ));
    pane_surface_host.set_activity_asset_used_by(to_host_contract_asset_references(
        &presentation.activity.used_by,
    ));
    pane_surface_host.set_activity_asset_search_query(presentation.activity.search_query);
    pane_surface_host.set_activity_asset_kind_filter(presentation.activity.kind_filter);
    pane_surface_host.set_activity_asset_view_mode(presentation.activity.view_mode);
    pane_surface_host.set_activity_asset_utility_tab(presentation.activity.utility_tab);
    pane_surface_host.set_browser_asset_tree_folders(to_host_contract_asset_folders(
        &presentation.browser.tree_folders,
    ));
    pane_surface_host.set_browser_asset_content_folders(to_host_contract_asset_folders(
        &presentation.browser.content_folders,
    ));
    pane_surface_host.set_browser_asset_content_items(to_host_contract_asset_items(
        &presentation.browser.content_items,
    ));
    pane_surface_host.set_browser_asset_selection(to_host_contract_asset_selection(
        &presentation.browser.selection,
    ));
    pane_surface_host.set_browser_asset_references(to_host_contract_asset_references(
        &presentation.browser.references,
    ));
    pane_surface_host.set_browser_asset_used_by(to_host_contract_asset_references(
        &presentation.browser.used_by,
    ));
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
        &model.menu_bar,
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
        host_scene_data: to_host_contract_host_scene_data_with_runtime(
            &host_scene_data,
            component_showcase_runtime,
        ),
        native_floating_surface_data: to_host_contract_native_floating_surface_data_with_runtime(
            &native_floating_surface_data,
            component_showcase_runtime,
        ),
        host_shell: to_host_contract_host_shell(&presentation.host_shell),
        host_layout: to_host_contract_host_window_layout(&host_layout),
    };
    ui.set_host_presentation(host_presentation);
    pane_surface_host.set_welcome_pane(to_host_contract_welcome_pane(&welcome_pane));
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

fn to_host_contract_frame_rect(frame: &host_window::FrameRect) -> host_contract::FrameRect {
    host_contract::FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}

fn to_host_contract_tab_data(tab: host_window::TabData) -> host_contract::TabData {
    host_contract::TabData {
        id: tab.id,
        slot: tab.slot,
        title: tab.title,
        icon_key: tab.icon_key,
        active: tab.active,
        closeable: tab.closeable,
    }
}

fn to_host_contract_tabs(tabs: &ModelRc<host_window::TabData>) -> ModelRc<host_contract::TabData> {
    map_model_rc(tabs, to_host_contract_tab_data)
}

fn to_host_contract_floating_window_data(
    window: host_window::FloatingWindowData,
    header_height_px: f32,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
) -> host_contract::FloatingWindowData {
    let resolved_header_height_px = if window.header_frame.height > 0.0 {
        window.header_frame.height
    } else {
        header_height_px
    };
    let pane_size = host_window::PaneContentSize::new(
        window.frame.width,
        (window.frame.height - resolved_header_height_px - 1.0).max(0.0),
    );
    host_contract::FloatingWindowData {
        window_id: window.window_id,
        title: window.title,
        frame: to_host_contract_frame_rect(&window.frame),
        header_nodes: to_host_contract_template_nodes(&window.header_nodes),
        header_frame: to_host_contract_frame_rect(&window.header_frame),
        tab_frames: map_model_rc(&window.tab_frames, to_host_contract_chrome_tab),
        target_group: window.target_group,
        left_edge_target_group: window.left_edge_target_group,
        right_edge_target_group: window.right_edge_target_group,
        top_edge_target_group: window.top_edge_target_group,
        bottom_edge_target_group: window.bottom_edge_target_group,
        focus_target_id: window.focus_target_id,
        tabs: to_host_contract_tabs(&window.tabs),
        active_pane: to_host_contract_pane(
            window.active_pane,
            pane_size,
            component_showcase_runtime,
        ),
    }
}

fn to_host_contract_floating_windows(
    windows: &ModelRc<host_window::FloatingWindowData>,
    header_height_px: f32,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
) -> ModelRc<host_contract::FloatingWindowData> {
    map_model_rc(windows, |window| {
        to_host_contract_floating_window_data(window, header_height_px, component_showcase_runtime)
    })
}

fn to_host_contract_new_project_form(
    form: &view_data::NewProjectFormData,
) -> host_contract::NewProjectFormData {
    host_contract::NewProjectFormData {
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

fn to_host_contract_welcome_pane(
    pane: &view_data::WelcomePaneData,
) -> host_contract::WelcomePaneData {
    host_contract::WelcomePaneData {
        nodes: to_host_contract_template_nodes(&pane.nodes),
        title: pane.title.clone(),
        subtitle: pane.subtitle.clone(),
        status_message: pane.status_message.clone(),
        form: to_host_contract_new_project_form(&pane.form),
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

fn to_host_contract_recent_project(
    data: view_data::RecentProjectData,
) -> host_contract::RecentProjectData {
    host_contract::RecentProjectData {
        display_name: data.display_name,
        path: data.path,
        last_opened_label: data.last_opened_label,
        status_label: data.status_label,
        invalid: data.invalid,
    }
}

fn to_host_contract_recent_projects(
    data: &ModelRc<view_data::RecentProjectData>,
) -> ModelRc<host_contract::RecentProjectData> {
    map_model_rc(data, to_host_contract_recent_project)
}

fn to_host_contract_asset_folder(
    data: view_data::AssetFolderData,
) -> host_contract::AssetFolderData {
    host_contract::AssetFolderData {
        id: data.id,
        name: data.name,
        count: data.count,
        depth: data.depth,
        selected: data.selected,
    }
}

fn to_host_contract_asset_folders(
    data: &ModelRc<view_data::AssetFolderData>,
) -> ModelRc<host_contract::AssetFolderData> {
    map_model_rc(data, to_host_contract_asset_folder)
}

fn to_host_contract_asset_item(data: view_data::AssetItemData) -> host_contract::AssetItemData {
    host_contract::AssetItemData {
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

fn to_host_contract_asset_items(
    data: &ModelRc<view_data::AssetItemData>,
) -> ModelRc<host_contract::AssetItemData> {
    map_model_rc(data, to_host_contract_asset_item)
}

fn to_host_contract_asset_reference(
    data: view_data::AssetReferenceData,
) -> host_contract::AssetReferenceData {
    host_contract::AssetReferenceData {
        uuid: data.uuid,
        locator: data.locator,
        name: data.name,
        kind: data.kind,
        known_project_asset: data.known_project_asset,
    }
}

fn to_host_contract_asset_references(
    data: &ModelRc<view_data::AssetReferenceData>,
) -> ModelRc<host_contract::AssetReferenceData> {
    map_model_rc(data, to_host_contract_asset_reference)
}

fn to_host_contract_asset_selection(
    data: &view_data::AssetSelectionData,
) -> host_contract::AssetSelectionData {
    host_contract::AssetSelectionData {
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

fn to_host_contract_scene_viewport_chrome(
    data: &view_data::SceneViewportChromeData,
) -> host_contract::SceneViewportChromeData {
    host_contract::SceneViewportChromeData {
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

fn to_host_contract_animation_editor_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::AnimationEditorPaneData {
    pane_data_conversion::to_host_contract_animation_editor_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_assets_activity_pane(
    data: host_window::AssetsActivityPaneViewData,
) -> host_contract::AssetsActivityPaneData {
    pane_data_conversion::to_host_contract_assets_activity_pane(data)
}

fn to_host_contract_hierarchy_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::HierarchyPaneData {
    pane_data_conversion::to_host_contract_hierarchy_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_inspector_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::InspectorPaneData {
    pane_data_conversion::to_host_contract_inspector_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_console_pane(
    data: &host_window::PaneData,
    pane_size: host_window::PaneContentSize,
) -> host_contract::ConsolePaneData {
    pane_data_conversion::to_host_contract_console_pane_from_host_pane(data, pane_size)
}

fn to_host_contract_project_overview_pane(
    data: host_window::ProjectOverviewPaneViewData,
) -> host_contract::ProjectOverviewPaneData {
    pane_data_conversion::to_host_contract_project_overview_pane(data)
}

fn to_host_contract_module_plugin_status(
    data: host_window::ModulePluginStatusViewData,
) -> host_contract::ModulePluginStatusData {
    host_contract::ModulePluginStatusData {
        plugin_id: data.plugin_id,
        display_name: data.display_name,
        package_source: data.package_source,
        load_state: data.load_state,
        enabled: data.enabled,
        required: data.required,
        target_modes: data.target_modes,
        packaging: data.packaging,
        runtime_crate: data.runtime_crate,
        editor_crate: data.editor_crate,
        runtime_capabilities: data.runtime_capabilities,
        editor_capabilities: data.editor_capabilities,
        diagnostics: data.diagnostics,
    }
}

fn to_host_contract_module_plugins_pane(
    data: host_window::ModulePluginsPaneViewData,
) -> host_contract::ModulePluginsPaneData {
    host_contract::ModulePluginsPaneData {
        plugins: map_model_rc(&data.plugins, to_host_contract_module_plugin_status),
        diagnostics: data.diagnostics,
    }
}

fn to_host_contract_ui_asset_pane(
    data: crate::ui::asset_editor::UiAssetEditorPanePresentation,
) -> host_contract::UiAssetEditorPaneData {
    pane_data_conversion::to_host_contract_ui_asset_pane(data)
}

fn to_host_contract_pane(
    data: host_window::PaneData,
    pane_size: host_window::PaneContentSize,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
) -> host_contract::PaneData {
    let hierarchy = to_host_contract_hierarchy_pane(&data, pane_size);
    let inspector = to_host_contract_inspector_pane(&data, pane_size);
    let console = to_host_contract_console_pane(&data, pane_size);
    let animation = to_host_contract_animation_editor_pane(&data, pane_size);
    let pane_kind = data.kind.to_string();
    let project_overview = if pane_kind == "UiComponentShowcase" {
        component_showcase_runtime.map_or_else(
            || {
                pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane(
                    &data, pane_size,
                )
            },
            |runtime| {
                pane_data_conversion::to_host_contract_component_showcase_pane_from_host_pane_with_runtime(
                    &data, pane_size, runtime,
                )
            },
        )
    } else {
        to_host_contract_project_overview_pane(data.native_body.project_overview.clone())
    };

    host_contract::PaneData {
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
        viewport: to_host_contract_scene_viewport_chrome(&data.viewport),
        hierarchy,
        inspector,
        console,
        assets_activity: to_host_contract_assets_activity_pane(data.native_body.assets_activity),
        asset_browser: pane_data_conversion::to_host_contract_asset_browser_pane(
            data.native_body.asset_browser,
        ),
        project_overview,
        module_plugins: to_host_contract_module_plugins_pane(data.native_body.module_plugins),
        ui_asset: to_host_contract_ui_asset_pane(data.native_body.ui_asset),
        animation,
    }
}

fn to_host_contract_project_overview(
    overview: &host_window::ProjectOverviewData,
) -> host_contract::ProjectOverviewData {
    host_contract::ProjectOverviewData {
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

fn to_host_contract_host_window_layout(
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

fn to_host_contract_host_shell(
    shell: &host_window::HostWindowShellData,
) -> host_contract::HostWindowShellData {
    host_contract::HostWindowShellData {
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
        native_window_bounds: to_host_contract_frame_rect(&shell.native_window_bounds),
    }
}

fn to_host_contract_metrics(
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

fn to_host_contract_orchestration(
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

fn to_host_contract_chrome_control_frame(
    data: host_window::HostChromeControlFrameData,
) -> host_contract::HostChromeControlFrameData {
    host_contract::HostChromeControlFrameData {
        control_id: data.control_id,
        frame: to_host_contract_frame_rect(&data.frame),
    }
}

fn to_host_contract_chrome_tab(
    data: host_window::HostChromeTabData,
) -> host_contract::HostChromeTabData {
    host_contract::HostChromeTabData {
        control_id: data.control_id,
        tab: to_host_contract_tab_data(data.tab),
        frame: to_host_contract_frame_rect(&data.frame),
        close_frame: to_host_contract_frame_rect(&data.close_frame),
    }
}

fn to_host_contract_menu_chrome(
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

fn to_host_contract_menu_chrome_menu(
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

fn to_host_contract_menu_chrome_item(
    item: host_window::HostMenuChromeItemData,
) -> host_contract::HostMenuChromeItemData {
    host_contract::HostMenuChromeItemData {
        label: item.label,
        shortcut: item.shortcut,
        action_id: item.action_id,
        enabled: item.enabled,
    }
}

fn to_host_contract_page_chrome(
    page: &host_window::HostPageChromeData,
) -> host_contract::HostPageChromeData {
    host_contract::HostPageChromeData {
        top_bar_height_px: page.top_bar_height_px,
        host_bar_height_px: page.host_bar_height_px,
        template_nodes: to_host_contract_template_nodes(&page.template_nodes),
        tab_row_frame: to_host_contract_frame_rect(&page.tab_row_frame),
        project_path_frame: to_host_contract_frame_rect(&page.project_path_frame),
        tab_frames: map_model_rc(&page.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&page.tabs),
        project_path: page.project_path.clone(),
    }
}

fn to_host_contract_status_bar(
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

fn to_host_contract_resize_layer(
    resize_layer: &host_window::HostResizeLayerData,
) -> host_contract::HostResizeLayerData {
    host_contract::HostResizeLayerData {
        left_splitter_frame: to_host_contract_frame_rect(&resize_layer.left_splitter_frame),
        right_splitter_frame: to_host_contract_frame_rect(&resize_layer.right_splitter_frame),
        bottom_splitter_frame: to_host_contract_frame_rect(&resize_layer.bottom_splitter_frame),
    }
}

fn to_host_contract_drag_overlay(
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

fn to_host_contract_side_dock(
    dock: &host_window::HostSideDockSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
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
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane: to_host_contract_pane(dock.pane.clone(), pane_size, component_showcase_runtime),
        rail_width_px: dock.rail_width_px,
        panel_width_px: dock.panel_width_px,
        panel_header_height_px: dock.panel_header_height_px,
    }
}

fn to_host_contract_document_dock(
    dock: &host_window::HostDocumentDockSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
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
        subtitle_frame: to_host_contract_frame_rect(&dock.subtitle_frame),
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane: to_host_contract_pane(dock.pane.clone(), pane_size, component_showcase_runtime),
        header_height_px: dock.header_height_px,
    }
}

fn to_host_contract_bottom_dock(
    dock: &host_window::HostBottomDockSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
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
        content_frame: to_host_contract_frame_rect(&dock.content_frame),
        tab_frames: map_model_rc(&dock.tab_frames, to_host_contract_chrome_tab),
        tabs: to_host_contract_tabs(&dock.tabs),
        pane: to_host_contract_pane(dock.pane.clone(), pane_size, component_showcase_runtime),
        expanded: dock.expanded,
        header_height_px: dock.header_height_px,
    }
}

fn to_host_contract_floating_layer(
    layer: &host_window::HostFloatingWindowLayerData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
) -> host_contract::HostFloatingWindowLayerData {
    host_contract::HostFloatingWindowLayerData {
        floating_windows: to_host_contract_floating_windows(
            &layer.floating_windows,
            layer.header_height_px,
            component_showcase_runtime,
        ),
        header_height_px: layer.header_height_px,
    }
}

#[cfg(test)]
pub(super) fn to_host_contract_host_scene_data(
    scene: &host_window::HostWindowSceneData,
) -> host_contract::HostWindowSceneData {
    to_host_contract_host_scene_data_with_runtime(scene, None)
}

fn to_host_contract_host_scene_data_with_runtime(
    scene: &host_window::HostWindowSceneData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
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
        left_dock: to_host_contract_side_dock(&scene.left_dock, component_showcase_runtime),
        document_dock: to_host_contract_document_dock(
            &scene.document_dock,
            component_showcase_runtime,
        ),
        right_dock: to_host_contract_side_dock(&scene.right_dock, component_showcase_runtime),
        bottom_dock: to_host_contract_bottom_dock(&scene.bottom_dock, component_showcase_runtime),
        floating_layer: to_host_contract_floating_layer(
            &scene.floating_layer,
            component_showcase_runtime,
        ),
    }
}

fn to_host_contract_native_floating_surface_data_with_runtime(
    surface: &host_window::HostNativeFloatingWindowSurfaceData,
    component_showcase_runtime: Option<&EditorUiHostRuntime>,
) -> host_contract::HostNativeFloatingWindowSurfaceData {
    host_contract::HostNativeFloatingWindowSurfaceData {
        floating_windows: to_host_contract_floating_windows(
            &surface.floating_windows,
            surface.header_height_px,
            component_showcase_runtime,
        ),
        native_floating_window_id: surface.native_floating_window_id.clone(),
        native_window_bounds: to_host_contract_frame_rect(&surface.native_window_bounds),
        header_height_px: surface.header_height_px,
    }
}
