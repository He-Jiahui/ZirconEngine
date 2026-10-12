#[path = "fields/axis.rs"]
mod axis;
#[path = "fields/scalar.rs"]
mod scalar;
#[path = "fields/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use axis::push_axis_value_commands;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use scalar::push_scalar_value_commands;
