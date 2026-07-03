use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiTextPaint};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::style::{
    aligned_text_x, text_paint_style_from_resolved_style,
};

pub(super) fn push_fallback_text_command(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    text: &UiTextPaint,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    color: [u8; 4],
) {
    let text_x = aligned_text_x(&frame, &text.source_text, &command.style);
    output.push(HostPaintCommand::text(
        FrameRect {
            x: text_x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        },
        clip_frame,
        z_index,
        text.source_text.clone(),
        color,
        text.font_size.max(1.0),
        text.line_height.max(text.font_size).max(1.0),
        text_paint_style_from_resolved_style(&command.style),
        command.opacity,
    ));
}
