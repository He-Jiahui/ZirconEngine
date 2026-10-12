#[path = "scene_floor/grate.rs"]
mod grate;
#[path = "scene_floor/grid.rs"]
mod grid;
#[path = "scene_floor/panels.rs"]
mod panels;
#[path = "scene_floor/primitives.rs"]
mod primitives;
#[path = "scene_floor/seams.rs"]
mod seams;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use grate::push_floor_grate_slots;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use grid::push_floor_grid_line;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use panels::push_floor_panel_detail;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use seams::push_floor_seam;
