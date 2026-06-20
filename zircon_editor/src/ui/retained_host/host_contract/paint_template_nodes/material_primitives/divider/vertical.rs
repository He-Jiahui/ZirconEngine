use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::{vertical_divider_extent, vertical_label_bounds, vertical_line_x};
use super::labels::{divider_label, push_vertical_divider_label};
use super::lines::push_vertical_line;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_vertical_divider(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let (line_top, line_bottom) = vertical_divider_extent(node, rect);
    let line_x = vertical_line_x(rect);
    let label = divider_label(node);
    if label.is_empty() {
        push_vertical_line(
            commands,
            line_x,
            line_top,
            line_bottom,
            clip,
            order,
            node,
            opacity,
        );
        return;
    }

    let (label_top, label_bottom) = vertical_label_bounds(node, rect, line_bottom);

    push_vertical_line(
        commands, line_x, line_top, label_top, clip, order, node, opacity,
    );
    push_vertical_line(
        commands,
        line_x,
        label_bottom,
        line_bottom,
        clip,
        order,
        node,
        opacity,
    );
    push_vertical_divider_label(
        commands,
        node,
        &label,
        label_top,
        label_bottom,
        rect,
        clip,
        order + 1,
        opacity,
    );
}
