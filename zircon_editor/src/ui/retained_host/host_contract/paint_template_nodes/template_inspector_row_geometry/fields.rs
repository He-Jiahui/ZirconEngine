use super::super::super::data::FrameRect;
use super::metrics::inspector_row_metrics;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn nested_select_field_rect(
    rect: &FrameRect,
) -> FrameRect {
    let metrics = inspector_row_metrics();
    let left_offset =
        metrics.nested_label_width + metrics.count_width + metrics.nested_select_offset_x;
    field_rect(rect, left_offset, rect.width - left_offset)
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn field_rect(
    rect: &FrameRect,
    left_offset: f32,
    width: f32,
) -> FrameRect {
    let metrics = inspector_row_metrics();
    let frame_width = finite_extent(rect.width);
    let frame_height = finite_extent(rect.height);
    let left = finite_extent(left_offset).min(frame_width);
    let available_width = (frame_width - left).max(0.0);
    let content_width = finite_extent(width).min(available_width);
    let inset_y = metrics.field_inset_y.min(frame_height * 0.5);
    FrameRect {
        x: finite_coordinate(rect.x) + left,
        y: finite_coordinate(rect.y) + inset_y,
        width: content_width,
        height: (frame_height - inset_y * 2.0).max(0.0),
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn leading_affordance_rect(
    field: &FrameRect,
) -> FrameRect {
    let metrics = inspector_row_metrics();
    let field_width = finite_extent(field.width);
    let field_height = finite_extent(field.height);
    let size = metrics.icon_size.min(field_width).min(field_height);
    let left_inset = metrics.field_text_x.min((field_width - size).max(0.0));
    FrameRect {
        x: finite_coordinate(field.x) + left_inset,
        y: finite_coordinate(field.y) + (field_height - size) * 0.5,
        width: size,
        height: size,
    }
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn chevron_rect(
    field: &FrameRect,
    size: f32,
) -> FrameRect {
    let metrics = inspector_row_metrics();
    let requested_size = if size.is_finite() && size > 0.0 {
        size
    } else {
        metrics.chevron_size
    };
    let field_width = finite_extent(field.width);
    let field_height = finite_extent(field.height);
    let size = requested_size.min(field_width).min(field_height).max(0.0);
    let right_pad = metrics.chevron_right_pad.min((field_width - size).max(0.0));
    FrameRect {
        x: finite_coordinate(field.x) + field_width - size - right_pad,
        y: finite_coordinate(field.y) + (field_height - size) * 0.5,
        width: size,
        height: size,
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
