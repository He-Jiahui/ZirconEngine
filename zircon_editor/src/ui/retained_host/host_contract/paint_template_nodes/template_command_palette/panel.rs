use super::super::super::data::{FrameRect, TemplatePaneNodeData};
use super::super::super::paint_theme::PALETTE;
use super::super::render_commands::HostPaintCommand;
use super::layout::{
    empty_text_rect, search_rect, search_text_rect, FONT_SIZE, LINE_HEIGHT, PANEL_RADIUS,
    SEARCH_RADIUS,
};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const SEARCH_PLACEHOLDER: &str = "Search commands";
const EMPTY_MESSAGE: &str = "No commands found";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_palette_panel_commands(
    commands: &mut Vec<HostPaintCommand>,
    node: &TemplatePaneNodeData,
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

    let search_rect = search_rect(rect);
    commands.push(HostPaintCommand::quad(
        search_rect.clone(),
        Some(clip.clone()),
        order + 1,
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
        order + 2,
        search_text.to_string(),
        search_color,
        FONT_SIZE,
        LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_palette_empty_message(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    commands.push(HostPaintCommand::text(
        empty_text_rect(rect),
        Some(clip.clone()),
        order,
        EMPTY_MESSAGE.to_string(),
        PALETTE.text_muted,
        FONT_SIZE,
        LINE_HEIGHT,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
