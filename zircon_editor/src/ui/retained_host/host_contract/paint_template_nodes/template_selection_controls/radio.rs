use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::super::template_selection_control_geometry::{
    centered_square, frame_is_within, label_rect_after_mark, leading_mark_rect, radio_dot_size,
};
use super::labels::push_selection_label;
use super::layers::{mark_content_order, mark_label_order};
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
    if frame_is_within(&mark, rect) {
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
    }
    if (node.checked || node.selected) && frame_is_within(&mark, rect) {
        let dot_size = radio_dot_size(node);
        let dot = centered_square(&mark, dot_size);
        if frame_is_within(&dot, &mark) {
            commands.push(HostPaintCommand::quad(
                dot,
                Some(clip.clone()),
                mark_content_order(order),
                Some(control_accent_color(node)),
                None,
                0.0,
                dot_size * 0.5,
                opacity,
            ));
        }
    }
    let label = label_rect_after_mark(node, rect, &mark);
    if frame_is_within(&label, rect) {
        push_selection_label(
            commands,
            node,
            label,
            clip,
            mark_label_order(order),
            selection_mark_label_color(node),
            opacity,
        );
    }
}
