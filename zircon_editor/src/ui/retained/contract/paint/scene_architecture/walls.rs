#[path = "walls/column.rs"]
mod column;
#[path = "walls/detail.rs"]
mod detail;
#[path = "walls/palette.rs"]
mod palette;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use column::push_wall_column;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use detail::push_wall_detail_lines;
