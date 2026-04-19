use crate::snapshot::EditorChromeSnapshot;
use crate::ui::layouts::windows::workbench_host_window::{
    build_host_scene_data, build_native_floating_surface_data, frame_rect, ShellPresentation,
};
use crate::ui::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::ui::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::ui::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_center_band_frame,
    resolve_root_document_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame, resolve_root_status_bar_frame,
    resolve_root_viewport_content_frame,
};
use crate::ui::slint_host::{HostWindowLayoutData, UiHostWindow};
use crate::ui::workbench::model::WorkbenchViewModel;
use crate::{ShellRegionId, WorkbenchShellGeometry};
use slint::ComponentHandle;

pub(crate) fn apply_presentation(
    ui: &UiHostWindow,
    model: &WorkbenchViewModel,
    chrome: &EditorChromeSnapshot,
    geometry: &WorkbenchShellGeometry,
    preset_names: &[String],
    active_preset_name: Option<&str>,
    ui_asset_panes: &std::collections::BTreeMap<String, crate::UiAssetEditorPanePresentation>,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
    floating_window_projection_bundle: &FloatingWindowProjectionBundle,
) {
    let presentation = ShellPresentation::from_state(
        model,
        chrome,
        geometry,
        preset_names,
        active_preset_name,
        ui_asset_panes,
        floating_window_projection_bundle,
    );
    let document_pane_shows_viewport_toolbar =
        presentation.host_surface_data.document_pane.show_toolbar;
    let pane_surface_host = ui.global::<crate::ui::slint_host::PaneSurfaceHostContext>();

    pane_surface_host.set_welcome_pane(presentation.welcome.pane);
    pane_surface_host.set_recent_projects(presentation.welcome.recent_projects);
    pane_surface_host.set_hierarchy_nodes(presentation.hierarchy_nodes);
    pane_surface_host.set_project_overview(presentation.project_overview);
    pane_surface_host.set_activity_asset_tree_folders(presentation.activity.tree_folders);
    pane_surface_host.set_activity_asset_content_folders(presentation.activity.content_folders);
    pane_surface_host.set_activity_asset_content_items(presentation.activity.content_items);
    pane_surface_host.set_activity_asset_selection(presentation.activity.selection);
    pane_surface_host.set_activity_asset_references(presentation.activity.references);
    pane_surface_host.set_activity_asset_used_by(presentation.activity.used_by);
    pane_surface_host.set_activity_asset_search_query(presentation.activity.search_query);
    pane_surface_host.set_activity_asset_kind_filter(presentation.activity.kind_filter);
    pane_surface_host.set_activity_asset_view_mode(presentation.activity.view_mode);
    pane_surface_host.set_activity_asset_utility_tab(presentation.activity.utility_tab);
    pane_surface_host.set_browser_asset_tree_folders(presentation.browser.tree_folders);
    pane_surface_host.set_browser_asset_content_folders(presentation.browser.content_folders);
    pane_surface_host.set_browser_asset_content_items(presentation.browser.content_items);
    pane_surface_host.set_browser_asset_selection(presentation.browser.selection);
    pane_surface_host.set_browser_asset_references(presentation.browser.references);
    pane_surface_host.set_browser_asset_used_by(presentation.browser.used_by);
    pane_surface_host.set_browser_asset_search_query(presentation.browser.search_query);
    pane_surface_host.set_browser_asset_kind_filter(presentation.browser.kind_filter);
    pane_surface_host.set_browser_asset_view_mode(presentation.browser.view_mode);
    pane_surface_host.set_browser_asset_utility_tab(presentation.browser.utility_tab);
    let host_layout = host_window_layout(
        geometry,
        shared_root_frames,
        document_pane_shows_viewport_toolbar,
    );
    ui.set_workbench_scene_data(build_host_scene_data(
        &presentation.host_surface_data,
        &presentation.host_shell,
        &host_layout,
        &presentation.status_primary,
        presentation.delete_enabled,
    ));
    ui.set_native_floating_surface_data(build_native_floating_surface_data(
        &presentation.host_surface_data,
        &presentation.host_shell,
    ));
    ui.set_host_shell(presentation.host_shell);
    ui.set_host_layout(host_layout);
    pane_surface_host.set_status_text(presentation.status_primary);
    pane_surface_host.set_delete_enabled(presentation.delete_enabled);
    pane_surface_host.set_inspector_name(presentation.inspector_name);
    pane_surface_host.set_inspector_parent(presentation.inspector_parent);
    pane_surface_host.set_inspector_x(presentation.inspector_x);
    pane_surface_host.set_inspector_y(presentation.inspector_y);
    pane_surface_host.set_inspector_z(presentation.inspector_z);
    pane_surface_host.set_mesh_import_path(presentation.mesh_import_path);
}

fn host_window_layout(
    geometry: &WorkbenchShellGeometry,
    shared_root_frames: Option<&BuiltinWorkbenchRootShellFrames>,
    document_pane_shows_viewport_toolbar: bool,
) -> HostWindowLayoutData {
    HostWindowLayoutData {
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
