use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::identity::is_component_property_row;
use super::metrics::property_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_label_width(
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
) -> f32 {
    let metrics = property_row_metrics();
    let preferred = if is_component_property_row(node) {
        metrics.component_property_label_width
    } else {
        metrics.property_label_width
    };
    preferred
        .max(metrics.property_label_min_width)
        .min(rect.width * metrics.property_label_max_width_ratio)
        .max(1.0)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn label_text_rect(
    rect: &FrameRect,
    label_width: f32,
) -> FrameRect {
    let metrics = property_row_metrics();
    FrameRect {
        x: rect.x + metrics.property_text_inset_x,
        y: rect.y + metrics.property_text_inset_y,
        width: (label_width - metrics.property_text_inset_x * 1.5).max(1.0),
        height: (rect.height - metrics.property_text_inset_y * 2.0).max(1.0),
    }
}
