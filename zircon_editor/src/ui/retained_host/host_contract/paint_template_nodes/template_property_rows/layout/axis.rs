use super::super::super::super::data::FrameRect;
use super::metrics::{
    PROPERTY_AXIS_GAP, PROPERTY_AXIS_WIDTH, PROPERTY_FIELD_INSET_Y, PROPERTY_GROUP_GAP,
    PROPERTY_TEXT_INSET_Y,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_label_rect(
    rect: &FrameRect,
    count: usize,
    index: usize,
) -> FrameRect {
    let metrics = axis_group_metrics(rect, count, index);
    FrameRect {
        x: metrics.group_x,
        y: rect.y + PROPERTY_TEXT_INSET_Y,
        width: PROPERTY_AXIS_WIDTH,
        height: (rect.height - PROPERTY_TEXT_INSET_Y * 2.0).max(1.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn axis_field_rect(
    rect: &FrameRect,
    count: usize,
    index: usize,
) -> FrameRect {
    let metrics = axis_group_metrics(rect, count, index);
    FrameRect {
        x: metrics.group_x + PROPERTY_AXIS_WIDTH + PROPERTY_AXIS_GAP,
        y: rect.y + PROPERTY_FIELD_INSET_Y,
        width: (metrics.group_width - PROPERTY_AXIS_WIDTH - PROPERTY_AXIS_GAP).max(1.0),
        height: metrics.field_height,
    }
}

struct AxisGroupMetrics {
    group_x: f32,
    group_width: f32,
    field_height: f32,
}

fn axis_group_metrics(rect: &FrameRect, count: usize, index: usize) -> AxisGroupMetrics {
    let group_gap_total = PROPERTY_GROUP_GAP * count.saturating_sub(1) as f32;
    let group_width = ((rect.width - group_gap_total) / count as f32).max(1.0);
    AxisGroupMetrics {
        group_x: rect.x + (group_width + PROPERTY_GROUP_GAP) * index as f32,
        group_width,
        field_height: (rect.height - PROPERTY_FIELD_INSET_Y * 2.0).max(1.0),
    }
}
