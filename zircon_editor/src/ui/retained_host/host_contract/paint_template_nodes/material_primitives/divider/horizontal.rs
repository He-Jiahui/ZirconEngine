use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::{horizontal_divider_extent, horizontal_label_bounds, horizontal_line_y};
use super::labels::{divider_label, push_horizontal_divider_label};
use super::lines::push_horizontal_line;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_horizontal_divider(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let (line_start, line_end) = horizontal_divider_extent(node, rect);
    let line_y = horizontal_line_y(rect);
    let label = divider_label(node);
    if label.is_empty() {
        push_horizontal_line(
            commands, line_start, line_end, line_y, clip, order, node, opacity,
        );
        return;
    }

    let (label_left, label_right) = horizontal_label_bounds(node, line_start, line_end, &label);

    push_horizontal_line(
        commands, line_start, label_left, line_y, clip, order, node, opacity,
    );
    push_horizontal_line(
        commands,
        label_right,
        line_end,
        line_y,
        clip,
        order,
        node,
        opacity,
    );
    push_horizontal_divider_label(
        commands,
        node,
        &label,
        label_left,
        label_right,
        rect,
        clip,
        order + 1,
        opacity,
    );
}
