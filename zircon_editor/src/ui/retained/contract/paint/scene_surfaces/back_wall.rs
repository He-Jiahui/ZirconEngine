#[path = "back_wall/haze.rs"]
mod haze;
#[path = "back_wall/palette.rs"]
mod palette;
#[path = "back_wall/panels.rs"]
mod panels;
#[path = "back_wall/shadow.rs"]
mod shadow;
#[path = "back_wall/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use surface::push_back_wall_surface;
