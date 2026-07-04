use super::super::super::super::data::FrameRect;
use super::metrics::property_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_rect(
    rect: &FrameRect,
    count: usize,
    index: usize,
) -> FrameRect {
    let row_metrics = property_row_metrics();
    let metrics = axis_group_metrics(rect, count, index);
    FrameRect {
        x: metrics.group_x,
        y: rect.y + row_metrics.property_text_inset_y,
        width: row_metrics.property_axis_width,
        height: (rect.height - row_metrics.property_text_inset_y * 2.0).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_rect(
    rect: &FrameRect,
    count: usize,
    index: usize,
) -> FrameRect {
    let row_metrics = property_row_metrics();
    let metrics = axis_group_metrics(rect, count, index);
    FrameRect {
        x: metrics.group_x + row_metrics.property_axis_width + row_metrics.property_axis_gap,
        y: rect.y + row_metrics.property_field_inset_y,
        width: (metrics.group_width
            - row_metrics.property_axis_width
            - row_metrics.property_axis_gap)
            .max(1.0),
        height: metrics.field_height,
    }
}

struct AxisGroupMetrics {
    group_x: f32,
    group_width: f32,
    field_height: f32,
}

fn axis_group_metrics(rect: &FrameRect, count: usize, index: usize) -> AxisGroupMetrics {
    let row_metrics = property_row_metrics();
    let group_gap_total = row_metrics.property_group_gap * count.saturating_sub(1) as f32;
    let group_width = ((rect.width - group_gap_total) / count as f32).max(1.0);
    AxisGroupMetrics {
        group_x: rect.x + (group_width + row_metrics.property_group_gap) * index as f32,
        group_width,
        field_height: (rect.height - row_metrics.property_field_inset_y * 2.0).max(1.0),
    }
}
