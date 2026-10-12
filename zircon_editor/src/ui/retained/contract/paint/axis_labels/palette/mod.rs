#[path = "model.rs"]
mod model;
#[path = "projection.rs"]
mod projection;
#[path = "rgb.rs"]
mod rgb;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use model::AxisLabelPalette;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use projection::{
    axis_label_palette, axis_label_palette_from_host,
};
