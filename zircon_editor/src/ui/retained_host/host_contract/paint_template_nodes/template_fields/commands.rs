use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::geometry::field_paint_rect;
use super::identity::{is_stepper_field, is_workbench_field};
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
    let rect = field_paint_rect(node, rect);
    if rect.width <= 0.0 || rect.height <= 0.0 {
        return true;
    }
    let opacity = field_opacity(node, opacity);
    let style = field_style(node);
    let stepper = is_stepper_field(node);

    push_field_surface(commands, &rect, clip, order, opacity, &style);
    super::search::push_search_field_glyph(
        commands,
        node,
        &rect,
        clip,
        order + 1,
        opacity,
        style.text,
    );
    if stepper {
        super::super::template_field_stepper::push_field_stepper(
            commands,
            &rect,
            clip,
            order + 2,
            opacity,
            &style,
        );
    }
    push_field_text(
        commands,
        node,
        &rect,
        clip,
        order + 3,
        stepper,
        opacity,
        &style,
    );
    true
}
