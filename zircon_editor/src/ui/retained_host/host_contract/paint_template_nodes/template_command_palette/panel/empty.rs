use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_theme::PALETTE;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{empty_text_rect, FONT_SIZE, LINE_HEIGHT};
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const EMPTY_MESSAGE: &str = "No commands found";

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
