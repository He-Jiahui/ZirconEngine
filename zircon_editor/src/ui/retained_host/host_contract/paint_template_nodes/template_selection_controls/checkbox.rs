mod tick;

use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_selection_control_geometry::{label_rect_after_mark, leading_mark_rect};
use super::labels::push_selection_label;
use super::style::{checkbox_background, checkbox_border_color, selection_mark_label_color};
use tick::push_checkbox_tick;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_checkbox(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let mark = leading_mark_rect(node, rect);
    commands.push(HostPaintCommand::quad(
        mark.clone(),
        Some(clip.clone()),
        order,
        Some(checkbox_background(node)),
        Some(checkbox_border_color(node)),
        1.0,
        3.0,
        opacity,
    ));
    if node.checked || node.selected {
        push_checkbox_tick(commands, &mark, clip, order + 1, opacity);
    }
    push_selection_label(
        commands,
        node,
        label_rect_after_mark(node, rect, &mark),
        clip,
        order + 2,
        selection_mark_label_color(node),
        opacity,
    );
}
