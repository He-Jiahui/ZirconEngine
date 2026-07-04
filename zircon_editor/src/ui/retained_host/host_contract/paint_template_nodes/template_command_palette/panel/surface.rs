use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::command_palette_metrics;
use super::super::palette::command_palette_palette;

pub(super) fn push_command_palette_panel_surface(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = command_palette_metrics();
    let palette = command_palette_palette();
    commands.push(HostPaintCommand::quad(
        rect.clone(),
        Some(clip.clone()),
        order,
        Some(palette.panel_surface),
        Some(palette.panel_border),
        1.0,
        metrics.panel_radius,
        opacity,
    ));
}
