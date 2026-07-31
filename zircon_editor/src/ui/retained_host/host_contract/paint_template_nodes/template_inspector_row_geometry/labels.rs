use super::super::super::data::FrameRect;
use super::metrics::inspector_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn nested_label_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = inspector_row_metrics();
    let row_width = finite_extent(rect.width);
    let row_height = finite_extent(rect.height);
    let left = (metrics.nested_label_base_x + metrics.nested_label_offset_x).min(row_width);
    let inset_y = metrics.row_text_y.min(row_height * 0.5);
    let available_width = (row_width - left).max(0.0);
    FrameRect {
        x: finite_coordinate(rect.x) + left,
        y: finite_coordinate(rect.y) + inset_y,
        width: (metrics.nested_label_width - left - metrics.gap_s)
            .max(0.0)
            .min(available_width),
        height: (row_height - inset_y * 2.0).max(0.0),
    }
}

fn finite_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() {
        value
    } else {
        0.0
    }
}
