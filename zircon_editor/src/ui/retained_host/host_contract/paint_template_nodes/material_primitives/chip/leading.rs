use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::geometry::{chip_avatar_frame, chip_icon_frame};
use super::style::{chip_avatar_background_color, chip_foreground_color};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_avatar(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = chip_avatar_frame(node, rect);
    let corner_radius = frame.height * 0.5;
    commands.push(HostPaintCommand::quad(
        frame,
        Some(clip.clone()),
        order,
        Some(chip_avatar_background_color(node)),
        None,
        0.0,
        corner_radius,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_icon(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = chip_icon_frame(node, rect);
    let center_y = frame.y + frame.height * 0.5;
    let color = chip_foreground_color(node);
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: frame.x,
            y: center_y - 1.0,
            width: frame.width,
            height: 2.0,
        },
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        FrameRect {
            x: frame.x + frame.width * 0.5 - 1.0,
            y: frame.y,
            width: 2.0,
            height: frame.height,
        },
        Some(clip.clone()),
        order + 1,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
}
