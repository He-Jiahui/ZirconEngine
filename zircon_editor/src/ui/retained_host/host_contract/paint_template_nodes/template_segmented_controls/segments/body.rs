use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_segmented_control_geometry::{
    segment_radius, segment_rect, segmented_body_rect,
};
use super::super::labels::{push_segment_label, push_segmented_group_label};
use super::super::options::{option_is_selected, selected_segment_value};
use super::super::style::segmented_control_style;
use super::divider::push_segment_divider;
use super::selected::push_selected_segment;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_segmented_control(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    options: &[String],
) {
    push_segmented_group_label(commands, node, rect, clip, order + 3, opacity);
    let body_rect = segmented_body_rect(node, rect);
    let style = segmented_control_style(node);
    commands.push(HostPaintCommand::quad(
        body_rect.clone(),
        Some(clip.clone()),
        order,
        style.background,
        style.border,
        style.border_width,
        segment_radius(),
        opacity,
    ));

    let selected = selected_segment_value(node);
    for (index, option) in options.iter().enumerate() {
        let segment = segment_rect(&body_rect, index, options.len());
        if index > 0 {
            push_segment_divider(commands, &segment, clip, order + 1, opacity);
        }
        let is_selected = option_is_selected(option, selected.as_deref());
        if is_selected {
            push_selected_segment(commands, node, &segment, clip, order + 2, opacity);
        }
        push_segment_label(
            commands,
            node,
            option,
            &segment,
            clip,
            order + 4,
            is_selected,
            opacity,
        );
    }
}
