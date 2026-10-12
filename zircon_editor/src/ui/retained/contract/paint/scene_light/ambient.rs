#[path = "../ambient/floor_reflection.rs"]
mod floor_reflection;
#[path = "ambient/soft_light.rs"]
mod soft_light;
#[path = "ambient/soft_shadow.rs"]
mod soft_shadow;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use floor_reflection::push_floor_reflection;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use soft_light::push_soft_light;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use soft_shadow::push_soft_shadow;
