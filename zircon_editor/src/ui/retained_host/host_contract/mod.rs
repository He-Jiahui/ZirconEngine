#![allow(dead_code)]

mod chrome_command_stream;
mod data;
mod diagnostics;
mod frame_geometry;
mod globals;
mod native_input_translation;
mod native_keyboard;
mod native_pointer;
mod native_popup_dismiss;
mod paint_close_prompt;
mod paint_debug_reflector_overlay;
mod paint_diagnostics;
mod paint_frame;
mod paint_geometry;
mod paint_primitives;
mod paint_recording;
mod paint_template_nodes;
mod paint_text;
mod paint_theme;
mod paint_workbench;
mod paint_workbench_impl;
mod presenter;
mod profiling_artifacts;
mod profiling_hit_routes;
mod redraw;
mod surface_hit_test;
mod template_activation_semantics;
mod template_component_family;
mod template_geometry;
mod template_input_semantics;
mod template_popup_layout;
mod window;
mod workbench_context_menu;

pub(crate) use data::*;
pub(crate) use diagnostics::{HostInvalidationDiagnostics, STARTUP_REFRESH_DIAGNOSTICS_OVERLAY};
pub(crate) use globals::{PaneSurfaceHostContext, UiHostContext};
#[cfg(test)]
pub(crate) use native_input_translation::{
    native_ime_event_to_shared_input, native_keyboard_event_to_shared_input,
    native_mouse_wheel_event_to_shared_input,
};
#[cfg(test)]
pub(crate) use paint_template_nodes::{
    paint_runtime_render_commands_for_test, paint_template_nodes_for_test,
    paint_template_nodes_for_test_with_background,
};
pub(crate) use surface_hit_test::build_pane_template_surface_frame;
pub(crate) use window::UiHostWindow;
