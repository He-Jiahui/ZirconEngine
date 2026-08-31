mod binding;
mod lifecycle;
mod redraw;
mod reference_cpu;
mod resize;

pub(in crate::entry::runtime_entry_app) use resize::surface_resize_changes_viewport;
