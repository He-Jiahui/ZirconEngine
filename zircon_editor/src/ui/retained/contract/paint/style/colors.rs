#[path = "colors/border.rs"]
mod border;
#[path = "colors/surface.rs"]
mod surface;
#[path = "colors/text.rs"]
mod text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use border::border_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use surface::surface_color;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use text::text_color;
