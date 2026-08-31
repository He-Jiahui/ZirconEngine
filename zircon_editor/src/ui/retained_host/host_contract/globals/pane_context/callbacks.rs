use std::rc::Rc;

use crate::ui::retained_host::primitives::SharedString;
use zircon_runtime_interface::ui::dispatch::UiPointerComponentEvent;

use super::super::super::data::WorkbenchContextMenuRequestData;
use super::PaneSurfaceHostContext;

impl PaneSurfaceHostContext<'_> {
    callback_methods!(pane_callbacks, on_welcome_recent_pointer_clicked, invoke_welcome_recent_pointer_clicked, welcome_recent_pointer_clicked, (x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_welcome_recent_pointer_moved, invoke_welcome_recent_pointer_moved, welcome_recent_pointer_moved, (x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_welcome_recent_pointer_scrolled, invoke_welcome_recent_pointer_scrolled, welcome_recent_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_hierarchy_pointer_clicked, invoke_hierarchy_pointer_clicked, hierarchy_pointer_clicked, (x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_hierarchy_pointer_moved, invoke_hierarchy_pointer_moved, hierarchy_pointer_moved, (x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_hierarchy_pointer_scrolled, invoke_hierarchy_pointer_scrolled, hierarchy_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_hierarchy_pointer_event, invoke_hierarchy_pointer_event, hierarchy_pointer_event, (kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_console_pointer_scrolled, invoke_console_pointer_scrolled, console_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_inspector_pointer_scrolled, invoke_inspector_pointer_scrolled, inspector_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_inspector_reference_pointer_event, invoke_inspector_reference_pointer_event, inspector_reference_pointer_event, (kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_inspector_control_changed, invoke_inspector_control_changed, inspector_control_changed, (control_id: SharedString, value: SharedString));
    callback_methods!(pane_callbacks, on_inspector_control_clicked, invoke_inspector_control_clicked, inspector_control_clicked, (control_id: SharedString));
    callback_methods!(pane_callbacks, on_surface_control_clicked, invoke_surface_control_clicked, surface_control_clicked, (control_id: SharedString, action_id: SharedString));
    callback_methods!(pane_callbacks, on_pointer_component_event, invoke_pointer_component_event, pointer_component_event, (event: UiPointerComponentEvent));
    callback_methods!(pane_callbacks, on_template_table_row_selected, invoke_template_table_row_selected, template_table_row_selected, (pane_id: SharedString, control_id: SharedString, source_index: i32, identity_kind: SharedString, identity_text: SharedString));
    callback_methods!(pane_callbacks, on_workbench_context_menu_requested, invoke_workbench_context_menu_requested, workbench_context_menu_requested, (request: WorkbenchContextMenuRequestData));
    callback_methods!(pane_callbacks, on_surface_control_edited, invoke_surface_control_edited, surface_control_edited, (control_id: SharedString, binding_id: SharedString, native_value: SharedString));
    callback_methods!(pane_callbacks, on_component_showcase_control_activated, invoke_component_showcase_control_activated, component_showcase_control_activated, (control_id: SharedString, action_id: SharedString));
    callback_methods!(pane_callbacks, on_component_showcase_control_drag_delta, invoke_component_showcase_control_drag_delta, component_showcase_control_drag_delta, (control_id: SharedString, action_id: SharedString, delta: f32));
    callback_methods!(pane_callbacks, on_component_showcase_control_edited, invoke_component_showcase_control_edited, component_showcase_control_edited, (control_id: SharedString, action_id: SharedString, value: SharedString));
    callback_methods!(pane_callbacks, on_component_showcase_control_context_requested, invoke_component_showcase_control_context_requested, component_showcase_control_context_requested, (control_id: SharedString, action_id: SharedString, x: f32, y: f32));
    callback_methods!(pane_callbacks, on_component_showcase_option_selected, invoke_component_showcase_option_selected, component_showcase_option_selected, (control_id: SharedString, action_id: SharedString, option_id: SharedString));
    callback_methods!(pane_callbacks, on_mesh_import_path_edited, invoke_mesh_import_path_edited, mesh_import_path_edited, (value: SharedString));
    callback_methods!(pane_callbacks, on_asset_control_changed, invoke_asset_control_changed, asset_control_changed, (source: SharedString, control_id: SharedString, value: SharedString));
    callback_methods!(pane_callbacks, on_asset_control_clicked, invoke_asset_control_clicked, asset_control_clicked, (source: SharedString, control_id: SharedString));
    callback_methods!(pane_callbacks, on_asset_tree_pointer_clicked, invoke_asset_tree_pointer_clicked, asset_tree_pointer_clicked, (surface_mode: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_tree_pointer_event, invoke_asset_tree_pointer_event, asset_tree_pointer_event, (surface_mode: SharedString, kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_tree_pointer_moved, invoke_asset_tree_pointer_moved, asset_tree_pointer_moved, (surface_mode: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_tree_pointer_scrolled, invoke_asset_tree_pointer_scrolled, asset_tree_pointer_scrolled, (surface_mode: SharedString, x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_content_pointer_clicked, invoke_asset_content_pointer_clicked, asset_content_pointer_clicked, (surface_mode: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_content_pointer_event, invoke_asset_content_pointer_event, asset_content_pointer_event, (surface_mode: SharedString, kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32, origin_x: f32, origin_y: f32));
    callback_methods!(pane_callbacks, on_asset_content_pointer_moved, invoke_asset_content_pointer_moved, asset_content_pointer_moved, (surface_mode: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_content_pointer_scrolled, invoke_asset_content_pointer_scrolled, asset_content_pointer_scrolled, (surface_mode: SharedString, x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_clicked, invoke_asset_reference_pointer_clicked, asset_reference_pointer_clicked, (surface_mode: SharedString, list_kind: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_event, invoke_asset_reference_pointer_event, asset_reference_pointer_event, (surface_mode: SharedString, list_kind: SharedString, kind: i32, button: i32, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_moved, invoke_asset_reference_pointer_moved, asset_reference_pointer_moved, (surface_mode: SharedString, list_kind: SharedString, x: f32, y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_left, invoke_asset_reference_pointer_left, asset_reference_pointer_left, (surface_mode: SharedString));
    callback_methods!(pane_callbacks, on_asset_reference_pointer_scrolled, invoke_asset_reference_pointer_scrolled, asset_reference_pointer_scrolled, (surface_mode: SharedString, list_kind: SharedString, x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_browser_asset_details_pointer_scrolled, invoke_browser_asset_details_pointer_scrolled, browser_asset_details_pointer_scrolled, (x: f32, y: f32, delta: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_welcome_control_changed, invoke_welcome_control_changed, welcome_control_changed, (control_id: SharedString, value: SharedString));
    callback_methods!(pane_callbacks, on_welcome_control_clicked, invoke_welcome_control_clicked, welcome_control_clicked, (control_id: SharedString));
    callback_methods!(pane_callbacks, on_scene_viewport_pointer_event, invoke_scene_viewport_pointer_event, scene_viewport_pointer_event, (kind: i32, button: i32, x: f32, y: f32, delta: f32, shift: bool, control: bool));
    callback_methods!(pane_callbacks, on_game_viewport_pointer_event, invoke_game_viewport_pointer_event, game_viewport_pointer_event, (kind: i32, button: i32, x: f32, y: f32, delta: f32, shift: bool, control: bool));
    callback_methods!(pane_callbacks, on_viewport_toolbar_pointer_clicked, invoke_viewport_toolbar_pointer_clicked, viewport_toolbar_pointer_clicked, (surface_key: SharedString, point_x: f32, point_y: f32, width: f32, height: f32));
    callback_methods!(pane_callbacks, on_ui_asset_action, invoke_ui_asset_action, ui_asset_action, (instance_id: SharedString, action_id: SharedString));
    callback_methods!(pane_callbacks, on_ui_asset_detail_event, invoke_ui_asset_detail_event, ui_asset_detail_event, (instance_id: SharedString, detail_id: SharedString, action_id: SharedString, item_index: i32, primary: SharedString, secondary: SharedString));
    callback_methods!(pane_callbacks, on_ui_asset_collection_event, invoke_ui_asset_collection_event, ui_asset_collection_event, (instance_id: SharedString, collection_id: SharedString, event_kind: SharedString, item_index: i32));
}
