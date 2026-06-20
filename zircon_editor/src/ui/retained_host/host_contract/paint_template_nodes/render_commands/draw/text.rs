use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_text::draw_text_with_size_and_style;
use super::super::super::super::paint_theme::PALETTE;
use super::super::command::HostPaintCommand;
use super::color::color_with_opacity;

const FALLBACK_TEXT: [u8; 4] = PALETTE.text;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_text_command(
    frame: &mut HostRgbaFrame,
    command: &HostPaintCommand,
) -> bool {
    let Some(text) = command.text.as_ref() else {
        return false;
    };
    let color = color_with_opacity(
        command.foreground_color.unwrap_or(FALLBACK_TEXT),
        command.opacity,
    );
    draw_text_with_size_and_style(
        frame,
        command.frame.clone(),
        text,
        command.clip_frame.as_ref(),
        color,
        command.font_size,
        command.line_height,
        command.text_style,
    );
    true
}
