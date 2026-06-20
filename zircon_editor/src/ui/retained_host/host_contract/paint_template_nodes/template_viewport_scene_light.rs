mod ambient;
mod fixtures;
mod primitives;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use ambient::{
    push_floor_reflection, push_soft_light, push_soft_shadow,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use fixtures::{
    push_beacon, push_wall_light,
};
