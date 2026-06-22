use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::PANEL_RADIUS;

pub(super) fn push_command_palette_panel_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(PALETTE.popup),
        Some(PALETTE.border),
        1.0,
        PANEL_RADIUS,
        opacity,
    ));
}
