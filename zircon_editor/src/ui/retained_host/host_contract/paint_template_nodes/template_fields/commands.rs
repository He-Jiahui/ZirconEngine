use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_field_stepper::workbench_field_stepper_metrics;
use super::geometry::{field_paint_rect, frame_is_within, has_paintable_field_extent};
use super::identity::{is_stepper_field, is_workbench_field};
use super::layers::{SEARCH_GLYPH_OFFSET, STEPPER_OFFSET, TEXT_OFFSET};
use super::style::{field_opacity, field_style};
use super::surface::push_field_surface;
use super::text::push_field_text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_field_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !is_workbench_field(node) {
        return false;
    }
    if !has_paintable_field_extent(rect) {
        return true;
    }
    let rect = field_paint_rect(node, rect);
    if !frame_is_within(&rect, clip) {
        return true;
    }
    let opacity = field_opacity(node, opacity);
    let style = field_style(node);
    let stepper = is_stepper_field(node) && field_can_paint_stepper(&rect);

    push_field_surface(commands, &rect, clip, order, opacity, &style);
    super::search::push_search_field_glyph(
        commands,
        node,
        &rect,
        clip,
        order + SEARCH_GLYPH_OFFSET,
        opacity,
        style.text,
    );
    if stepper {
        super::super::template_field_stepper::push_field_stepper(
            commands,
            &rect,
            clip,
            order + STEPPER_OFFSET,
            opacity,
            &style,
        );
    }
    push_field_text(
        commands,
        node,
        &rect,
        clip,
        order + TEXT_OFFSET,
        stepper,
        opacity,
        &style,
    );
    true
}

fn field_can_paint_stepper(rect: &FrameRect) -> bool {
    let metrics = workbench_field_stepper_metrics();
    let left = rect.x + rect.width - metrics.width;
    let divider = FrameRect {
        x: left,
        y: rect.y + metrics.divider_inset_y,
        width: metrics.divider_width,
        height: rect.height - metrics.divider_inset_y * 2.0,
    };
    let glyph = FrameRect {
        x: left + metrics.glyph_left_inset,
        y: rect.y + (rect.height - metrics.glyph_height).max(0.0) * 0.5,
        width: metrics.glyph_width,
        height: metrics.glyph_height,
    };
    frame_is_within(&divider, rect) && frame_is_within(&glyph, rect)
}
