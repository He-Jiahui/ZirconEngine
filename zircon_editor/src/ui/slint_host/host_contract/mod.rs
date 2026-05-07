#![allow(dead_code)]

mod data;
mod diagnostics;
mod globals;
mod native_input_translation;
mod native_pointer;
mod painter;
mod presenter;
mod redraw;
mod surface_hit_test;
mod window;

pub(crate) use data::*;
pub(crate) use diagnostics::{HostInvalidationDiagnostics, STARTUP_REFRESH_DIAGNOSTICS_OVERLAY};
pub(crate) use globals::{PaneSurfaceHostContext, UiHostContext};
pub(crate) use native_input_translation::{
    native_ime_event_to_shared_input, native_keyboard_event_to_shared_input,
    native_mouse_wheel_event_to_shared_input,
};
#[cfg(test)]
pub(crate) use painter::{paint_runtime_render_commands_for_test, paint_template_nodes_for_test};
pub(crate) use surface_hit_test::build_pane_template_surface_frame;
pub(crate) use window::UiHostWindow;
