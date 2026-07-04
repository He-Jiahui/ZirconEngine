use super::super::super::super::data::FrameRect;
use super::super::super::render_commands::HostPaintCommand;
use super::super::layout::{command_palette_metrics, empty_text_rect};
use super::super::palette::command_palette_palette;
use zircon_runtime_interface::ui::surface::UiTextRunPaintStyle;

const EMPTY_MESSAGE: &str = "No commands found";

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_command_palette_empty_message(
    commands: &mut Vec<HostPaintCommand>,
    rect: &FrameRect,
    clip: &FrameRect,
    order: i32,
    opacity: f32,
) {
    let metrics = command_palette_metrics();
    let palette = command_palette_palette();
    commands.push(HostPaintCommand::text(
        empty_text_rect(rect),
        Some(clip.clone()),
        order,
        EMPTY_MESSAGE.to_string(),
        palette.empty_text,
        metrics.font_size,
        metrics.line_height,
        UiTextRunPaintStyle::default(),
        opacity,
    ));
}
