use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_inspector_row_geometry::{
    inspector_row_metrics, nested_label_rect,
};
use super::text::push_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    color: [u8; 4],
    opacity: f32,
) {
    let metrics = inspector_row_metrics();
    let row_width = finite_extent(rect.width);
    let row_height = finite_extent(rect.height);
    let left = metrics.border_width.min(row_width);
    let inset_y = metrics.row_text_y.min(row_height * 0.5);
    push_text(
        commands,
        FrameRect {
            x: finite_coordinate(rect.x) + left,
            y: finite_coordinate(rect.y) + inset_y,
            width: (metrics.label_width - metrics.gap_s)
                .max(0.0)
                .min((row_width - left).max(0.0)),
            height: (row_height - inset_y * 2.0).max(0.0),
        },
        clip,
        order,
        label,
        color,
        opacity,
    );
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_nested_label(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    label: &str,
    color: [u8; 4],
    opacity: f32,
) {
    push_text(
        commands,
        nested_label_rect(rect),
        clip,
        order,
        label,
        color,
        opacity,
    );
}

fn finite_extent(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}

fn finite_coordinate(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}
