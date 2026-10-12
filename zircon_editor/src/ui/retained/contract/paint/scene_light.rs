#[path = "scene_light/ambient.rs"]
mod ambient;
#[path = "scene_light/fixtures.rs"]
mod fixtures;
#[path = "scene_light/primitives.rs"]
mod primitives;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use ambient::{
    push_floor_reflection, push_soft_light, push_soft_shadow,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use fixtures::{
    push_beacon, push_wall_light,
};
