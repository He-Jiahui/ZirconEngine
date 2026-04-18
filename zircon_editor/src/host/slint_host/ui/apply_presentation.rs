use super::shell_presentation::ShellPresentation;
use super::*;
use crate::host::slint_host::callback_dispatch::BuiltinWorkbenchRootShellFrames;
use crate::host::slint_host::floating_window_projection::FloatingWindowProjectionBundle;
use crate::host::slint_host::root_shell_projection::{
    resolve_root_bottom_region_frame, resolve_root_center_band_frame,
    resolve_root_document_region_frame, resolve_root_left_region_frame,
    resolve_root_right_region_frame, resolve_root_status_bar_frame,
    resolve_root_viewport_content_frame,
};

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
    let document_pane_shows_viewport_toolbar = presentation.document_pane.show_toolbar;

    ui.set_host_tabs(presentation.host_tabs);
    ui.set_breadcrumbs(presentation.breadcrumbs);
    ui.set_left_tabs(presentation.left_tabs);
    ui.set_right_tabs(presentation.right_tabs);
    ui.set_bottom_tabs(presentation.bottom_tabs);
    ui.set_document_tabs(presentation.document_tabs);
    ui.set_floating_windows(presentation.floating_windows);
    ui.set_left_pane(presentation.left_pane);
    ui.set_right_pane(presentation.right_pane);
    ui.set_bottom_pane(presentation.bottom_pane);
    ui.set_document_pane(presentation.document_pane);
    ui.set_welcome_pane(presentation.welcome.pane);
    ui.set_recent_projects(presentation.welcome.recent_projects);
    ui.set_hierarchy_nodes(presentation.hierarchy_nodes);
    ui.set_project_overview(presentation.project_overview);
    ui.set_activity_asset_tree_folders(presentation.activity.tree_folders);
    ui.set_activity_asset_content_folders(presentation.activity.content_folders);
    ui.set_activity_asset_content_items(presentation.activity.content_items);
    ui.set_activity_asset_selection(presentation.activity.selection);
    ui.set_activity_asset_references(presentation.activity.references);
    ui.set_activity_asset_used_by(presentation.activity.used_by);
    ui.set_activity_asset_search_query(presentation.activity.search_query);
    ui.set_activity_asset_kind_filter(presentation.activity.kind_filter);
    ui.set_activity_asset_view_mode(presentation.activity.view_mode);
    ui.set_activity_asset_utility_tab(presentation.activity.utility_tab);
    ui.set_browser_asset_tree_folders(presentation.browser.tree_folders);
    ui.set_browser_asset_content_folders(presentation.browser.content_folders);
    ui.set_browser_asset_content_items(presentation.browser.content_items);
    ui.set_browser_asset_selection(presentation.browser.selection);
    ui.set_browser_asset_references(presentation.browser.references);
    ui.set_browser_asset_used_by(presentation.browser.used_by);
    ui.set_browser_asset_search_query(presentation.browser.search_query);
    ui.set_browser_asset_kind_filter(presentation.browser.kind_filter);
    ui.set_browser_asset_view_mode(presentation.browser.view_mode);
    ui.set_browser_asset_utility_tab(presentation.browser.utility_tab);
    ui.set_project_path(presentation.project_path);
    ui.set_status_primary(presentation.status_primary);
    ui.set_status_secondary(presentation.status_secondary);
    ui.set_viewport_label(presentation.viewport_label);
    ui.set_drawers_visible(presentation.drawers_visible);
    ui.set_left_expanded(presentation.left_expanded);
    ui.set_right_expanded(presentation.right_expanded);
    ui.set_bottom_expanded(presentation.bottom_expanded);
    ui.set_left_drawer_extent(presentation.left_drawer_extent);
    ui.set_right_drawer_extent(presentation.right_drawer_extent);
    ui.set_bottom_drawer_extent(presentation.bottom_drawer_extent);
    ui.set_save_project_enabled(presentation.save_project_enabled);
    ui.set_undo_enabled(presentation.undo_enabled);
    ui.set_redo_enabled(presentation.redo_enabled);
    ui.set_delete_enabled(presentation.delete_enabled);
    ui.set_inspector_name(presentation.inspector_name);
    ui.set_inspector_parent(presentation.inspector_parent);
    ui.set_inspector_x(presentation.inspector_x);
    ui.set_inspector_y(presentation.inspector_y);
    ui.set_inspector_z(presentation.inspector_z);
    ui.set_mesh_import_path(presentation.mesh_import_path);
    ui.set_preset_names(presentation.preset_names);
    ui.set_active_preset_name(presentation.active_preset_name);
    ui.set_shell_min_width_px(geometry.window_min_width);
    ui.set_shell_min_height_px(geometry.window_min_height);
    ui.set_center_band_frame(frame_rect(resolve_root_center_band_frame(
        geometry,
        shared_root_frames,
    )));
    ui.set_status_bar_frame(frame_rect(resolve_root_status_bar_frame(
        geometry,
        shared_root_frames,
    )));
    ui.set_left_region_frame(frame_rect(resolve_root_left_region_frame(
        geometry,
        shared_root_frames,
    )));
    ui.set_document_region_frame(frame_rect(resolve_root_document_region_frame(
        geometry,
        shared_root_frames,
    )));
    ui.set_right_region_frame(frame_rect(resolve_root_right_region_frame(
        geometry,
        shared_root_frames,
    )));
    ui.set_bottom_region_frame(frame_rect(resolve_root_bottom_region_frame(
        geometry,
        shared_root_frames,
    )));
    ui.set_left_splitter_frame(frame_rect(geometry.splitter_frame(ShellRegionId::Left)));
    ui.set_right_splitter_frame(frame_rect(geometry.splitter_frame(ShellRegionId::Right)));
    ui.set_bottom_splitter_frame(frame_rect(geometry.splitter_frame(ShellRegionId::Bottom)));
    ui.set_viewport_content_frame(frame_rect(resolve_root_viewport_content_frame(
        geometry,
        shared_root_frames,
        document_pane_shows_viewport_toolbar,
    )));
    ui.set_native_floating_window_mode(false);
    ui.set_native_floating_window_id("".into());
    ui.set_native_window_title("Zircon Editor".into());
    ui.set_native_window_bounds(FrameRect {
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
    });
}

pub(super) fn frame_rect(frame: ShellFrame) -> FrameRect {
    FrameRect {
        x: frame.x,
        y: frame.y,
        width: frame.width,
        height: frame.height,
    }
}
