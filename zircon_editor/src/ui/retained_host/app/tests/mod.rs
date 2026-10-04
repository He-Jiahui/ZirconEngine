mod support;

pub(super) use support::*;

mod child_window_focus;
mod child_window_tabs;
mod close_prompt;
mod command_palette;
mod componentized_workbench;
mod document_save;
mod drag_sources;
mod floating_window_projection;
mod hierarchy_native_capture;
mod hierarchy_rename;
mod menu_pointer;
mod native_game_keyboard;
mod native_viewport_cancel;
mod opened_view_visibility;
mod projection_geometry;
mod retained_host_automation;
mod root_pointer_fallbacks;
mod runtime_diagnostics_drawer;
mod scoped_presentation;
mod serialized_startup_presentation;
mod viewport_template_bridge;
