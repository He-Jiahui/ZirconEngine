use super::super::super::super::data::FrameRect;
use super::metrics::property_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn property_value_area_rect(
    rect: &FrameRect,
    label_width: f32,
) -> FrameRect {
    let metrics = property_row_metrics();
    FrameRect {
        x: rect.x + label_width,
        y: rect.y,
        width: (rect.width - label_width - metrics.property_text_inset_x).max(0.0),
        height: rect.height,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn scalar_field_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = property_row_metrics();
    let inset_y = metrics
        .property_field_inset_y
        .min(rect.height.max(0.0) * 0.5);
    FrameRect {
        x: rect.x,
        y: rect.y + inset_y,
        width: rect.width,
        height: (rect.height - inset_y * 2.0).max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn value_text_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = property_row_metrics();
    let inset_x = metrics.property_text_inset_x.min(rect.width.max(0.0) * 0.5);
    let inset_y = metrics
        .property_text_inset_y
        .min(rect.height.max(0.0) * 0.5);
    FrameRect {
        x: rect.x + inset_x,
        y: rect.y + inset_y,
        width: (rect.width - inset_x * 2.0).max(0.0),
        height: (rect.height - inset_y * 2.0).max(0.0),
    }
}
