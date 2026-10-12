#[path = "scene_props/cargo.rs"]
mod cargo;
#[path = "scene_props/primitives.rs"]
mod primitives;
#[path = "scene_props/property.rs"]
mod property;
#[path = "scene_props/rails.rs"]
mod rails;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use cargo::{
    push_cargo_detail, push_cargo_inner_frame,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use property::{
    push_prop_body_detail, push_prop_top_detail,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rails::{
    push_handrail, push_rack_detail,
};
