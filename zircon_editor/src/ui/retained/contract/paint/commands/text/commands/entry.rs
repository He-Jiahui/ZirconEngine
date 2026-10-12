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

/// 按文字运行、整行字形布局、简单回退的顺序选一路输出，装饰层仍保留原元素的绘制层级。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn push_text_paint_commands(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    text: &UiTextPaint,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
) {
    use crate::ui::retained_host::host_contract::paint_text::{
        font_face_for_paint_style, font_request_for_face, HostTextFontRequestScope,
    };
    // A typed runtime command already carries resolved typography. Resolve its
    // source request once for measurement and every emitted text command.
    let paint_style =
        super::super::super::style::text_paint_style_from_resolved_style(&command.style);
    let mut request = {
        let _legacy = HostTextFontRequestScope::enter(None);
        font_request_for_face(font_face_for_paint_style(paint_style))
    };
    if let Some(family) = text
        .font_family
        .as_deref()
        .or(command.style.font_family.as_deref())
        .filter(|family| !family.trim().is_empty())
    {
        request.family = family.to_owned();
    }
    request.weight = zircon_runtime_interface::ui::surface::UiResolvedStyle::normalized_font_weight(
        text.font_weight,
    );
    let _request = HostTextFontRequestScope::enter(Some(&request));
    let first = output.len();
    push_resolved_text_paint_commands(output, command, text, frame, clip_frame, z_index);
    for emitted in &mut output[first..] {
        if emitted.text.is_some() && emitted.text_font_request.is_none() {
            emitted.text_font_request = Some(request.clone());
        }
    }
}

fn push_resolved_text_paint_commands(
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
