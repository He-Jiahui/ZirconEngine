use super::super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::render_commands::HostPaintCommand;
use super::super::super::layers::search_text_order;
use super::super::super::layout::{command_palette_metrics, search_text_rect};
use super::super::super::palette::WorkbenchCommandPalettePalette;
use super::super::super::text::command_palette_search_text;

mod style;

use style::command_palette_search_text_style;

pub(super) fn push_command_palette_search_text(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    search_rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
    palette: &WorkbenchCommandPalettePalette,
) {
    let metrics = command_palette_metrics();
    let search_text = command_palette_search_text(node.search_query.as_str());
    let style = command_palette_search_text_style(palette, &metrics, search_text.placeholder);
    commands.push(HostPaintCommand::text(
        search_text_rect(search_rect),
        Some(clip.clone()),
        search_text_order(order),
        search_text.value.to_string(),
        style.color,
        style.font_size,
        style.line_height,
        style.paint_style,
        opacity,
    ));
}
