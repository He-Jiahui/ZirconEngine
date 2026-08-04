use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_geometry::intersect;
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::{horizontal_line_frame, vertical_line_frame};
use super::style::divider_color;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_horizontal_line(
    commands: &mut Vec<HostPaintCommand>,
    left: f32,
    right: f32,
    y: f32,
    clip: &FrameRect,
    order: i32,
    node: &TemplatePaneNodeData,
    opacity: f32,
) {
    let Some(frame) = horizontal_line_frame(left, right, y) else {
        return;
    };
    push_quad(commands, frame, clip, order, divider_color(node), opacity);
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_vertical_line(
    commands: &mut Vec<HostPaintCommand>,
    x: f32,
    top: f32,
    bottom: f32,
    clip: &FrameRect,
    order: i32,
    node: &TemplatePaneNodeData,
    opacity: f32,
) {
    let Some(frame) = vertical_line_frame(x, top, bottom) else {
        return;
    };
    push_quad(commands, frame, clip, order, divider_color(node), opacity);
}

fn push_quad(
    commands: &mut Vec<HostPaintCommand>,
    frame: FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    if intersect(&frame, clip).is_none() {
        return;
    }
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        0.0,
        opacity,
    ));
}
