#[path = "doors/back.rs"]
mod back;
#[path = "doors/core.rs"]
mod core;
#[path = "doors/palette.rs"]
mod palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use back::push_back_door;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use core::push_door_core;
