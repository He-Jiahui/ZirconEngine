use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::geometry::local_rect;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_folder_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 2.0, 5.0, 10.0, 7.0),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        1.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 3.0, 3.0, 5.0, 3.0),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        1.0,
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_save_adornment(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    color: [u8; 4],
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 2.0, 2.0, 10.0, 10.0),
        Some(clip.clone()),
        order,
        Some(color),
        None,
        0.0,
        1.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 4.0, 3.0, 5.0, 3.0),
        Some(clip.clone()),
        order + 1,
        Some(PALETTE.popup),
        None,
        0.0,
        0.5,
        opacity,
    ));
    commands.push(HostPaintCommand::quad(
        local_rect(rect, 4.0, 9.0, 6.0, 2.0),
        Some(clip.clone()),
        order + 1,
        Some(PALETTE.popup),
        None,
        0.0,
        0.5,
        opacity,
    ));
}
