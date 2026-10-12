#[path = "rails/handrail.rs"]
mod handrail;
#[path = "rails/palette.rs"]
mod palette;
#[path = "rails/rack.rs"]
mod rack;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use handrail::push_handrail;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use rack::push_rack_detail;
