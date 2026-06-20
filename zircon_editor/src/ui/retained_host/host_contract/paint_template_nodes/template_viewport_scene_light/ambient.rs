mod floor_reflection;
mod soft_light;
mod soft_shadow;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use floor_reflection::push_floor_reflection;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use soft_light::push_soft_light;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use soft_shadow::push_soft_shadow;
