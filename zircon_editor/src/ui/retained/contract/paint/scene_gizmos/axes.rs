#[path = "axes/cap.rs"]
mod cap;
#[path = "axes/colors.rs"]
mod colors;
#[path = "axes/line.rs"]
mod line;
#[path = "axes/origin.rs"]
mod origin;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use line::push_axis_line;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use origin::push_axis_origin;
