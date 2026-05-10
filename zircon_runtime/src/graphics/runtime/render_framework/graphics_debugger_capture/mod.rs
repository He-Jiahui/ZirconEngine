mod environment;
mod graphics_debugger_state;
mod query_graphics_debugger_status;
mod request_graphics_debugger_capture;
mod submit_capture;

pub(in crate::graphics::runtime::render_framework) use environment::renderdoc_capture_next_from_env;
#[cfg(test)]
pub(crate) use environment::renderdoc_capture_next_from_value;
pub(in crate::graphics::runtime::render_framework) use graphics_debugger_state::GraphicsDebuggerState;
pub(in crate::graphics::runtime::render_framework) use query_graphics_debugger_status::query_graphics_debugger_status;
pub(in crate::graphics::runtime::render_framework) use request_graphics_debugger_capture::request_graphics_debugger_capture;
pub(in crate::graphics::runtime::render_framework) use submit_capture::{
    begin_graphics_debugger_capture, fail_pending_graphics_debugger_capture,
    finish_active_capture_and_relock,
};
