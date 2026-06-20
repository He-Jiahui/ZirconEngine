use zircon_runtime_interface::ui::surface::{UiRenderCommand, UiTextPaint};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::style::{
    parse_style_color, runtime_foreground_color,
};

use super::super::decorations::push_text_decorations;
use super::fallback::push_fallback_text_command;
use super::runs::push_text_run_commands;
use super::shaped::push_shaped_text_commands;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_text_paint_commands(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    text: &UiTextPaint,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
) {
    let color = parse_style_color(text.color.as_deref())
        .unwrap_or_else(|| runtime_foreground_color(&command.style));
    push_text_decorations(
        output,
        text,
        clip_frame.clone(),
        z_index,
        command.opacity,
        true,
    );
    if push_text_run_commands(
        output,
        text,
        clip_frame.clone(),
        z_index,
        command.opacity,
        color,
    ) {
        push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
        return;
    }

    if push_shaped_text_commands(
        output,
        text,
        clip_frame.clone(),
        z_index,
        command.opacity,
        color,
    ) {
        push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
        return;
    }

    push_fallback_text_command(
        output,
        command,
        text,
        frame,
        clip_frame.clone(),
        z_index,
        color,
    );
    push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
}
