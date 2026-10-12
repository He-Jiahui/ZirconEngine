#[path = "callback_surface/source_window.rs"]
mod source_window;
#[path = "callback_surface/surface_size.rs"]
mod surface_size;

pub(crate) use source_window::resolve_callback_source_window_id;
