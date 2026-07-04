mod axis;
mod labels;
mod metrics;
mod values;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use axis::{
    axis_field_rect, axis_label_rect,
};
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use labels::{
    label_text_rect, property_label_width,
};
#[cfg(test)]
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::component_property_label_width;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use metrics::property_row_metrics;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use values::{
    property_value_area_rect, scalar_field_rect, value_text_rect,
};
