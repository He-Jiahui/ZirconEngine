use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::render_commands::HostPaintCommand;
use super::super::super::template_icon_assets::push_icon_asset_pixels;
use super::geometry::{chip_avatar_frame, chip_icon_frame};
use super::style::{chip_avatar_background_color, chip_foreground_color};

const CHIP_ADD_ICON: &str = "zircon_editor_shell/controls/add.svg";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_chip_avatar(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let frame = chip_avatar_frame(node, rect);
    if frame.width <= 0.0 || frame.height <= 0.0 {
        return;
    }
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
    let color = chip_foreground_color(node);
    let _ = push_icon_asset_pixels(
        commands,
        CHIP_ADD_ICON,
        &frame,
        clip,
        order,
        Some(color),
        opacity,
    );
}
