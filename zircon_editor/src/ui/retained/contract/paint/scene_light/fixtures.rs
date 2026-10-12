#[path = "fixtures/beacon.rs"]
mod beacon;
#[path = "fixtures/palette.rs"]
mod palette;
#[path = "fixtures/wall_light.rs"]
mod wall_light;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use beacon::push_beacon;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use wall_light::push_wall_light;
