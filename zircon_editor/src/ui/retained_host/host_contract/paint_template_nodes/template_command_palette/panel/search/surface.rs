use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::layout::command_palette_metrics;
use super::super::super::palette::WorkbenchCommandPalettePalette;

mod style;

use style::command_palette_search_surface_style;

pub(super) fn push_command_palette_search_surface(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    search_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: &WorkbenchCommandPalettePalette,
) {
    let metrics = command_palette_metrics();
    let style = command_palette_search_surface_style(palette, &metrics, node.focused);
    commands.push(HostPaintCommand::quad(
        search_rect.clone(),
        Some(clip.clone()),
        order,
        Some(style.fill),
        Some(style.border),
        style.border_width,
        style.radius,
        opacity,
    ));
}
