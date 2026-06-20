use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::render_commands::HostPaintCommand;
use super::segments::{local_rect, push_segments};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_tree_disclosure_glyph(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if node.expanded {
        push_down_chevron(commands, rect, clip, order, color, opacity);
    } else {
        push_right_chevron(commands, rect, clip, order, color, opacity);
    }
}

fn push_down_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 3.0, 4.0, 2.0, 2.0),
            local_rect(rect, 5.0, 6.0, 2.0, 2.0),
            local_rect(rect, 7.0, 4.0, 2.0, 2.0),
        ],
    );
}

fn push_right_chevron(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    push_segments(
        commands,
        clip,
        order,
        color,
        opacity,
        &[
            local_rect(rect, 4.0, 3.0, 2.0, 3.0),
            local_rect(rect, 6.0, 6.0, 2.0, 2.0),
            local_rect(rect, 4.0, 8.0, 2.0, 3.0),
        ],
    );
}
