use super::super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{search_rect, search_text_rect, FONT_SIZE, LINE_HEIGHT, SEARCH_RADIUS};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SEARCH_PLACEHOLDER: &str = "Search commands";

pub(super) fn push_command_palette_search_field(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let search_rect = search_rect(rect);
    commands.push(HostPaintCommand::quad(
        search_rect.clone(),
        Some(clip.clone()),
        order,
        Some(PALETTE.surface_inset),
        Some(PALETTE.focus_ring),
        1.0,
        SEARCH_RADIUS,
        opacity,
    ));

    let query = node.search_query.as_str();
    let (search_text, search_color) = if query.trim().is_empty() {
        (SEARCH_PLACEHOLDER, PALETTE.text_muted)
    } else {
        (query, PALETTE.text)
    };
    commands.push(HostPaintCommand::text(
        search_text_rect(&search_rect),
        Some(clip.clone()),
        order + 1,
        search_text.to_string(),
        search_color,
        FONT_SIZE,
        LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
