use super::super::super::super::data::FrameRect;
use super::metrics::property_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_rect(
    rect: &FrameRect,
    count: usize,
    index: usize,
) -> FrameRect {
    let metrics = axis_group_metrics(rect, count, index);
    FrameRect {
        x: metrics.group_x,
        y: rect.y + metrics.text_inset_y,
        width: metrics.axis_label_width,
        height: (rect.height - metrics.text_inset_y * 2.0).max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_rect(
    rect: &FrameRect,
    count: usize,
    index: usize,
) -> FrameRect {
    let metrics = axis_group_metrics(rect, count, index);
    FrameRect {
        x: metrics.field_x,
        y: rect.y + metrics.field_inset_y,
        width: metrics.field_width,
        height: metrics.field_height,
    }
}

struct AxisGroupMetrics {
    group_x: f32,
    axis_label_width: f32,
    field_x: f32,
    field_width: f32,
    text_inset_y: f32,
    field_inset_y: f32,
    field_height: f32,
}

fn axis_group_metrics(rect: &FrameRect, count: usize, index: usize) -> AxisGroupMetrics {
    let row_metrics = property_row_metrics();
    let count = count.max(1);
    let index = index.min(count - 1);
    let group_gap = row_metrics
        .property_group_gap
        .min(rect.width.max(0.0) / count as f32);
    let group_gap_total = group_gap * count.saturating_sub(1) as f32;
    let group_width = ((rect.width.max(0.0) - group_gap_total) / count as f32).max(0.0);
    let axis_label_width = row_metrics
        .property_axis_width
        .min(group_width * AXIS_LABEL_MAX_GROUP_RATIO);
    let remaining_width = (group_width - axis_label_width).max(0.0);
    let axis_gap = row_metrics
        .property_axis_gap
        .min(remaining_width * AXIS_GAP_MAX_REMAINING_RATIO);
    let field_x = rect.x + (group_width + group_gap) * index as f32 + axis_label_width + axis_gap;
    let field_inset_y = row_metrics
        .property_field_inset_y
        .min(rect.height.max(0.0) * 0.5);
    AxisGroupMetrics {
        group_x: rect.x + (group_width + group_gap) * index as f32,
        axis_label_width,
        field_x,
        field_width: (remaining_width - axis_gap).max(0.0),
        text_inset_y: row_metrics
            .property_text_inset_y
            .min(rect.height.max(0.0) * 0.5),
        field_inset_y,
        field_height: (rect.height - field_inset_y * 2.0).max(0.0),
    }
}

const AXIS_LABEL_MAX_GROUP_RATIO: f32 = 0.35;
const AXIS_GAP_MAX_REMAINING_RATIO: f32 = 0.2;
