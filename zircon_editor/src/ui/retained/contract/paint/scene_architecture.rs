#[path = "scene_architecture/doors.rs"]
mod doors;
#[path = "scene_architecture/panels.rs"]
mod panels;
#[path = "scene_architecture/primitives.rs"]
mod primitives;
#[path = "scene_architecture/stairs.rs"]
mod stairs;
#[path = "scene_architecture/walls.rs"]
mod walls;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use doors::{
    push_back_door, push_door_core,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use panels::push_side_panel_detail;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use stairs::push_side_stairs;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use walls::{
    push_wall_column, push_wall_detail_lines,
};
