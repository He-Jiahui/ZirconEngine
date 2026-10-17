#[path = "../../workbench/ability_editor_menu.rs"]
mod ability_editor_menu;
#[path = "../../workbench/ability_workspace_state.rs"]
mod ability_workspace_state;
#[path = "../../workbench/asset_creation_menu.rs"]
mod asset_creation_menu;
#[path = "asset_editor_menu.rs"]
mod asset_editor_menu;
#[path = "../../workbench/assets_workspace_state.rs"]
mod assets_workspace_state;
#[path = "../../workbench/behavior_workspace_state.rs"]
mod behavior_workspace_state;
#[path = "blend_space_search.rs"]
mod blend_space_search;
#[path = "../../workbench/blend_space_selection.rs"]
mod blend_space_selection;
#[path = "../../workbench/blend_space_transport.rs"]
mod blend_space_transport;
#[path = "bridge.rs"]
mod bridge;
#[path = "command_palette.rs"]
mod command_palette;
#[path = "../../workbench/component_lab_field_edit.rs"]
mod component_lab_field_edit;
#[path = "../../workbench/component_property_rows.rs"]
mod component_property_rows;
#[path = "../../workbench/componentized_window.rs"]
mod componentized_window;
#[path = "context_menu.rs"]
mod context_menu;
#[path = "control_state.rs"]
mod control_state;
#[path = "data_sync.rs"]
mod data_sync;
#[path = "drawer_layout.rs"]
mod drawer_layout;
#[path = "../../workbench/effect_workspace_state.rs"]
mod effect_workspace_state;
#[path = "error.rs"]
mod error;
#[path = "feedback.rs"]
mod extension_module_feedback;
#[path = "navigation.rs"]
mod extension_module_navigation;
#[path = "../../../../dispatch/workbench/generated_bottom_panel_actions.rs"]
mod generated_bottom_panel_actions;
#[path = "../../../../dispatch/workbench/generated_bottom_panel_feedback.rs"]
mod generated_bottom_panel_feedback;
#[path = "../../../../dispatch/workbench/generated_bottom_panel_lifecycle.rs"]
mod generated_bottom_panel_lifecycle;
#[path = "../../../../dispatch/workbench/generated_bottom_panel_navigation.rs"]
mod generated_bottom_panel_navigation;
#[path = "host_projection.rs"]
mod host_projection;
#[path = "hud_editor_menu.rs"]
mod hud_editor_menu;
#[path = "../../workbench/hud_workspace_state.rs"]
mod hud_workspace_state;
#[path = "icon_tooltip.rs"]
mod icon_tooltip;
#[path = "../../workbench/inspector_component_source.rs"]
mod inspector_component_source;
#[path = "inspector_component_add.rs"]
mod inspector_component_add;
#[path = "inspector_filter.rs"]
mod inspector_filter;
#[path = "layout_frames.rs"]
mod layout_frames;
#[path = "layout_menu.rs"]
mod layout_menu;
#[path = "main_menu_items.rs"]
mod main_menu_items;
#[path = "../../workbench/material_workspace_state.rs"]
mod material_workspace_state;
#[path = "../../workbench/module_command_feedback.rs"]
mod module_command_feedback;
#[path = "module_field_edit.rs"]
mod module_field_edit;
#[path = "module_navigation.rs"]
mod module_navigation;
#[path = "../../workbench/module_overflow_menu.rs"]
mod module_overflow_menu;
#[path = "notifications.rs"]
mod notifications;
#[path = "outer_shell_frames.rs"]
mod outer_shell_frames;
#[path = "../../workbench/perception_workspace_state.rs"]
mod perception_workspace_state;
#[path = "pointer_feedback.rs"]
mod pointer_feedback;
#[path = "popup_state.rs"]
mod popup_state;
#[path = "property_edit.rs"]
mod property_edit;
#[path = "../../workbench/reference_menu_actions.rs"]
mod reference_menu_actions;
#[path = "render_editor_menu.rs"]
mod render_editor_menu;
#[path = "render_layer_edit.rs"]
mod render_layer_edit;
#[path = "../../workbench/render_workspace_state.rs"]
mod render_workspace_state;
#[path = "responsive_layout.rs"]
mod responsive_layout;
#[path = "root_shell_frames.rs"]
mod root_shell_frames;
#[path = "run_mode_menu.rs"]
mod run_mode_menu;
#[path = "../../workbench/scene_hierarchy_fragment.rs"]
mod scene_hierarchy_fragment;
#[path = "../../workbench/scene_hierarchy_project.rs"]
mod scene_hierarchy_projection;
#[path = "scene_tree_rows.rs"]
mod scene_tree_rows;
#[path = "../../workbench/search_clear_action.rs"]
mod search_clear_action;
#[path = "settings_window.rs"]
mod settings_window;
#[path = "status_bar.rs"]
mod status_bar;
#[path = "../../workbench/tags_workspace_state.rs"]
mod tags_workspace_state;
#[path = "toolbar_layout.rs"]
mod toolbar_layout;
#[path = "toolbar_run_state.rs"]
mod toolbar_run_state;
#[path = "transform_edit.rs"]
mod transform_edit;
#[path = "../../workbench/unreal_content_drawer.rs"]
mod unreal_content_drawer;
#[path = "../../../../dispatch/workbench/unreal_content_drawer_adapter.rs"]
mod unreal_content_drawer_adapter;
#[path = "../../workbench/unreal_viewport_overflow.rs"]
mod unreal_viewport_overflow;
#[path = "../../workbench/unreal_viewport_toolbar.rs"]
mod unreal_viewport_toolbar;
#[path = "../../workbench/vfx_workspace_state.rs"]
mod vfx_workspace_state;
#[path = "window_menu_state.rs"]
mod window_menu_state;

pub(crate) use unreal_content_drawer_adapter::{
    unreal_content_drawer_asset_browser_data, UnrealContentDrawerAssetBrowserData,
};

pub(crate) use bridge::BuiltinHostWindowTemplateBridge;
pub(crate) use command_palette::WorkbenchCommandPaletteOpenState;
pub(crate) use componentized_window::BuiltinWorkbenchWindowTemplateSurfaceBridge;
pub(crate) use context_menu::WORKBENCH_CONTEXT_MENU_CONTROL_ID;
pub(crate) use error::BuiltinHostWindowTemplateBridgeError;
pub(crate) use layout_frames::BuiltinWorkbenchWindowLayoutFrames;
pub(crate) use notifications::{
    WORKBENCH_NOTIFICATION_CENTER_CONTROL_ID, WORKBENCH_TOAST_CONTROL_ID,
};
pub(crate) use outer_shell_frames::BuiltinHostOuterShellFrames;
pub(crate) use root_shell_frames::BuiltinHostRootShellFrames;
pub(crate) use settings_window::{
    WorkbenchSettingsEditorKind, WorkbenchSettingsOpenState, WorkbenchSettingsWindowRevision,
    WORKBENCH_SETTINGS_WINDOW_CONTROL_ID,
};

#[cfg(test)]
#[path = "../../workbench/module_tools_test_support.rs"]
mod module_tools_test_support;
