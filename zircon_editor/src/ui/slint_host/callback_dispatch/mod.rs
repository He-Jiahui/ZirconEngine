mod asset;
mod common;
mod constants;
mod hierarchy;
mod inspector;
mod layout;
mod pane;
mod shared_pointer;
mod template_bridge;
mod viewport;
mod welcome;
mod workbench;

#[cfg(test)]
pub(crate) use asset::{dispatch_asset_item_selection, dispatch_asset_search};
pub(crate) use asset::{dispatch_builtin_asset_surface_control, dispatch_mesh_import_path_edit};
pub(crate) use constants::PANE_SURFACE_CONTROL_ID;
pub(crate) use hierarchy::dispatch_hierarchy_selection;
pub(crate) use inspector::dispatch_builtin_inspector_surface_control;
#[cfg(test)]
pub(crate) use inspector::{
    dispatch_inspector_apply, dispatch_inspector_delete_selected, dispatch_inspector_draft_field,
};
pub(crate) use layout::{
    dispatch_builtin_floating_window_focus, dispatch_builtin_floating_window_focus_for_source,
    dispatch_builtin_workbench_document_tab_activation,
    dispatch_builtin_workbench_document_tab_close, dispatch_builtin_workbench_drawer_toggle,
    dispatch_builtin_workbench_host_page_activation, dispatch_layout_command, dispatch_tab_drop,
    resolve_builtin_floating_window_close_instances,
};
pub(crate) use pane::dispatch_builtin_pane_surface_control;
pub(crate) use shared_pointer::{
    dispatch_shared_activity_rail_pointer_click, dispatch_shared_asset_content_pointer_click,
    dispatch_shared_asset_reference_pointer_click, dispatch_shared_asset_tree_pointer_click,
    dispatch_shared_document_tab_close_pointer_click, dispatch_shared_document_tab_pointer_click,
    dispatch_shared_drawer_header_pointer_click, dispatch_shared_hierarchy_pointer_click,
    dispatch_shared_host_page_pointer_click, dispatch_shared_menu_pointer_click,
    dispatch_shared_viewport_toolbar_pointer_click, dispatch_shared_welcome_recent_pointer_click,
};
#[cfg(test)]
pub(crate) use template_bridge::BuiltinWorkbenchDrawerSourceTemplateBridge;
pub(crate) use template_bridge::{
    BuiltinAssetSurfaceTemplateBridge, BuiltinFloatingWindowSourceFrames,
    BuiltinFloatingWindowSourceTemplateBridge, BuiltinInspectorSurfaceTemplateBridge,
    BuiltinPaneSurfaceTemplateBridge, BuiltinViewportToolbarTemplateBridge,
    BuiltinWelcomeSurfaceTemplateBridge, BuiltinWorkbenchRootShellFrames,
    BuiltinWorkbenchTemplateBridge,
};
#[cfg(test)]
pub(crate) use viewport::{dispatch_builtin_viewport_toolbar_control, dispatch_viewport_command};
pub(crate) use viewport::{
    dispatch_viewport_event, dispatch_viewport_pointer_event,
    dispatch_viewport_toolbar_pointer_route, viewport_event_from_command,
    SharedViewportPointerBridge,
};
pub(crate) use welcome::dispatch_builtin_welcome_surface_control;
pub(crate) use workbench::dispatch_workbench_menu_action_with_template_fallback;
#[cfg(test)]
pub(crate) use workbench::{
    dispatch_builtin_workbench_control, dispatch_builtin_workbench_menu_action,
    dispatch_menu_action,
};
