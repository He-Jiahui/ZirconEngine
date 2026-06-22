use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_dropdowns::{dropdown_paint_rect, push_dropdown_commands};
use super::super::super::template_popup_rows::push_template_popup_row_commands;

pub(super) fn push_dropdown_specialized_template_node_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    node_clip: &FrameRect,
    origin: &FrameRect,
    pane_clip: &FrameRect,
    order: i32,
    opacity: f32,
) -> bool {
    if !push_dropdown_commands(commands, node, rect, node_clip, order, opacity) {
        return false;
    }

    let popup_anchor = dropdown_paint_rect(node, rect);
    push_template_popup_row_commands(
        commands,
        node,
        &popup_anchor,
        origin,
        pane_clip,
        order,
        opacity,
    );
    true
}
