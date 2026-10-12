#[path = "floor/depth.rs"]
mod depth;
#[path = "floor/palette.rs"]
mod palette;
#[path = "floor/shadows.rs"]
mod shadows;
#[path = "floor/sheen.rs"]
mod sheen;
#[path = "floor/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use surface::push_floor_surface;
