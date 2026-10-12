#[path = "colors/border.rs"]
mod border;
#[path = "colors/content.rs"]
mod content;
#[path = "../../../../../contract/paint/selector/dropdown/colors/declared.rs"]
mod declared;
#[path = "colors/surface.rs"]
mod surface;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use border::dropdown_border;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use content::{
    dropdown_chevron, dropdown_text,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use surface::dropdown_surface;
