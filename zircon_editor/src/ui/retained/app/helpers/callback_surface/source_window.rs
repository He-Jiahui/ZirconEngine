#[path = "../source_window/focus.rs"]
mod focus;
#[path = "../source_window/resolution.rs"]
mod resolution;

pub(crate) use resolution::resolve_callback_source_window_id;
