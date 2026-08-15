mod binding;
mod fallback;
mod lifecycle;
mod redraw;
mod resize;

pub(in crate::entry::runtime_entry_app) use resize::surface_resize_changes_viewport;
