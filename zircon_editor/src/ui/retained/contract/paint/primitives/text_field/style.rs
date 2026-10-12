#[path = "style/fill.rs"]
mod fill;
#[path = "../../text_field/style/radius.rs"]
mod radius;
#[path = "../../text_field/style/stroke.rs"]
mod stroke;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use fill::*;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use radius::*;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use stroke::*;
