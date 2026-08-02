use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_inspector_row_geometry::inspector_row_metrics;
use super::super::template_inspector_row_glyphs::push_inspector_down_chevron;
use super::primitives::push_text;
use super::style::{resource_glyph_color, resource_label_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_disclosure_row(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = inspector_row_metrics();
    let row_width = finite_extent(rect.width);
    let row_height = finite_extent(rect.height);
    let left = (metrics.border_width * 2.0).min(row_width);
    let requested_size = (metrics.check_size - metrics.border_width * 2.0).max(0.0);
    let chevron_size = requested_size
        .min((row_width - left).max(0.0))
        .min(row_height);
    let inset_y = metrics.row_text_y.min(row_height * 0.5);
    let chevron = FrameRect {
        x: finite_coordinate(rect.x) + left,
        y: finite_coordinate(rect.y) + (row_height - chevron_size) * 0.5,
        width: chevron_size,
        height: chevron_size,
    };
    if chevron_size > 0.0 {
        push_inspector_down_chevron(
            commands,
            &chevron,
            clip,
            order,
            resource_glyph_color(node),
            opacity,
        );
    }
    let text_left = left + chevron_size + metrics.chevron_right_pad;
    push_text(
        commands,
        FrameRect {
            x: finite_coordinate(rect.x) + text_left,
            y: finite_coordinate(rect.y) + inset_y,
            width: (row_width - text_left - metrics.chevron_right_pad).max(0.0),
            height: (row_height - inset_y * 2.0).max(0.0),
        },
        clip,
        order + 1,
        node.text.trim(),
        resource_label_color(node),
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
