#[path = "scene_surfaces/back_wall.rs"]
mod back_wall;
#[path = "scene_surfaces/backdrop.rs"]
mod backdrop;
#[path = "scene_surfaces/ceiling.rs"]
mod ceiling;
#[path = "scene_surfaces/floor.rs"]
mod floor;
#[path = "scene_surfaces/primitives.rs"]
mod primitives;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use back_wall::push_back_wall_surface;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use backdrop::push_backdrop_surface;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use ceiling::push_ceiling_surface;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use floor::push_floor_surface;
