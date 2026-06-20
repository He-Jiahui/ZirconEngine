use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_selection_control_geometry::{
    centered_square, label_rect_after_mark, leading_mark_rect, radio_dot_size,
};
use super::labels::push_selection_label;
use super::style::{
    control_accent_color, radio_background, radio_border_color, selection_mark_label_color,
};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_radio(
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
        Some(radio_background(node)),
        Some(radio_border_color(node)),
        1.0,
        mark.height * 0.5,
        opacity,
    ));
    if node.checked || node.selected {
        let dot_size = radio_dot_size(node);
        let dot = centered_square(&mark, dot_size);
        commands.push(HostPaintCommand::quad(
            dot,
            Some(clip.clone()),
            order + 1,
            Some(control_accent_color(node)),
            None,
            0.0,
            dot_size * 0.5,
            opacity,
        ));
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
