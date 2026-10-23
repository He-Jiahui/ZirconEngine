#[path = "async_play.rs"]
mod async_play;
#[path = "generated_bottom_pane.rs"]
mod generated_bottom_pane;
#[path = "support.rs"]
mod support;
#[path = "ui_asset_builtin_palette.rs"]
mod ui_asset_builtin_palette;

pub(super) use support::*;

#[path = "child_window_focus.rs"]
mod child_window_focus;
#[path = "child_window_tabs.rs"]
mod child_window_tabs;
#[path = "close_prompt.rs"]
mod close_prompt;
#[path = "command_palette.rs"]
mod command_palette;
#[path = "componentized_workbench.rs"]
mod componentized_workbench;
#[path = "document_save.rs"]
mod document_save;
#[path = "drag_sources/mod.rs"]
mod drag_sources;
#[path = "floating_window_project.rs"]
mod floating_window_projection;
#[path = "hierarchy_native_capture.rs"]
mod hierarchy_native_capture;
#[path = "hierarchy_rename.rs"]
mod hierarchy_rename;
#[path = "hierarchy_search_actions.rs"]
mod hierarchy_search_actions;
#[path = "menu_pointer.rs"]
mod menu_pointer;
#[path = "native_game_keyboard.rs"]
mod native_game_keyboard;
#[path = "native_viewport_cancel.rs"]
mod native_viewport_cancel;
#[path = "opened_view_visibility.rs"]
mod opened_view_visibility;
#[path = "projection_geometry.rs"]
mod projection_geometry;
#[path = "retained_host_automation.rs"]
mod retained_host_automation;
#[path = "root_pointer_fallbacks.rs"]
mod root_pointer_fallbacks;
#[path = "runtime_diagnostics_drawer.rs"]
mod runtime_diagnostics_drawer;
#[path = "scoped_presentation.rs"]
mod scoped_presentation;
#[path = "serialized_startup_presentation.rs"]
mod serialized_startup_presentation;
#[path = "viewport_template_bridge.rs"]
mod viewport_template_bridge;

#[path = "save_batch_failure.rs"]
mod save_batch_failure;

#[path = "module_tools_menus.rs"]
mod module_tools_menus;

#[path = "module_plugin_details.rs"]
mod module_plugin_details;
