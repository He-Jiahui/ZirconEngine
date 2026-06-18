use zircon_runtime_interface::ui::surface::{
    UiRenderCommand, UiTextPaint, UiTextPaintDecorationKind, UiTextRunPaintStyle,
};

use super::super::super::data::FrameRect;
use super::super::render_commands::HostPaintCommand;
use super::style::{aligned_text_x, frame_from_ui, parse_style_color, runtime_foreground_color};

pub(super) fn push_text_paint_commands(
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
    if !text.runs.is_empty() {
        for run in &text.runs {
            let run_color = parse_style_color(run.color.as_deref()).unwrap_or(color);
            output.push(HostPaintCommand::text(
                frame_from_ui(run.frame),
                clip_frame.clone(),
                z_index,
                run.text.clone(),
                run_color,
                run.font_size.max(1.0),
                run.line_height.max(run.font_size).max(1.0),
                run.style,
                command.opacity,
            ));
        }
        push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
        return;
    }

    if let Some(shaped) = text.shaped.as_ref() {
        for line in &shaped.lines {
            output.push(HostPaintCommand::text(
                frame_from_ui(line.frame),
                clip_frame.clone(),
                z_index,
                line.text.clone(),
                color,
                text.font_size.max(1.0),
                text.line_height.max(text.font_size).max(1.0),
                UiTextRunPaintStyle::default(),
                command.opacity,
            ));
        }
        push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
        return;
    }

    let text_x = aligned_text_x(&frame, &text.source_text, &command.style);
    output.push(HostPaintCommand::text(
        FrameRect {
            x: text_x,
            y: frame.y,
            width: frame.width,
            height: frame.height,
        },
        clip_frame.clone(),
        z_index,
        text.source_text.clone(),
        color,
        text.font_size.max(1.0),
        text.line_height.max(text.font_size).max(1.0),
        UiTextRunPaintStyle::default(),
        command.opacity,
    ));
    push_text_decorations(output, text, clip_frame, z_index, command.opacity, false);
}

fn push_text_decorations(
    output: &mut Vec<HostPaintCommand>,
    text: &UiTextPaint,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
    before_text: bool,
) {
    for decoration in &text.decorations {
        let decoration_before_text =
            matches!(decoration.kind, UiTextPaintDecorationKind::Selection);
        if decoration_before_text != before_text {
            continue;
        }
        let color =
            parse_style_color(Some(decoration.color.as_str())).unwrap_or(match decoration.kind {
                UiTextPaintDecorationKind::Selection => [77, 137, 255, 102],
                UiTextPaintDecorationKind::CompositionUnderline => [77, 137, 255, 255],
                UiTextPaintDecorationKind::Caret => [232, 238, 247, 255],
                UiTextPaintDecorationKind::Outline => [232, 238, 247, 255],
            });
        let decoration_z = if decoration_before_text {
            z_index - 1
        } else {
            z_index + 1
        };
        output.push(HostPaintCommand::quad(
            frame_from_ui(decoration.frame),
            clip_frame.clone(),
            decoration_z,
            Some(color),
            None,
            0.0,
            0.0,
            opacity,
        ));
    }
}
