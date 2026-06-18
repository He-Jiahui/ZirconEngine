use zircon_runtime_interface::ui::surface::{UiPaintElement, UiPaintPayload, UiRenderCommand};

use super::super::data::FrameRect;
use super::super::paint_geometry::is_visible_frame;
use super::render_commands::HostPaintCommand;

mod brush;
mod image;
mod style;
mod text;

use brush::{brush_border, brush_fill_color, image_brush_resource};
use image::push_image_resource_command;
use style::frame_from_ui;
use text::push_text_paint_commands;

pub(super) fn runtime_render_commands_to_host(
    commands: &[UiRenderCommand],
    clip_frame: Option<&FrameRect>,
) -> Vec<HostPaintCommand> {
    let mut host_commands = Vec::new();
    for command in commands {
        push_runtime_command(&mut host_commands, command, clip_frame);
    }
    host_commands
}

fn push_runtime_command(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    parent_clip: Option<&FrameRect>,
) {
    let command_clip = command
        .clip_frame
        .map(frame_from_ui)
        .or_else(|| parent_clip.cloned());

    for element in command.to_paint_elements(0) {
        push_runtime_paint_element(output, command, &element, command_clip.clone());
    }
}

fn push_runtime_paint_element(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    element: &UiPaintElement,
    clip_frame: Option<FrameRect>,
) {
    let frame = frame_from_ui(element.geometry.render_bounds);
    if !is_visible_frame(&frame) || element.effects.opacity <= 0.0 {
        return;
    }

    match &element.payload {
        UiPaintPayload::Empty => output.push(HostPaintCommand::group(
            frame,
            clip_frame,
            element.z_index,
            element.effects.opacity,
        )),
        UiPaintPayload::Brush { brushes } => {
            if let Some(image_brush) = brushes.fill.as_ref().and_then(image_brush_resource) {
                push_image_resource_command(
                    output,
                    image_brush,
                    frame,
                    clip_frame,
                    element.z_index,
                    element.effects.opacity,
                );
            } else {
                let background_color = brushes.fill.as_ref().and_then(brush_fill_color);
                let (border_color, border_width) = brushes
                    .border
                    .as_ref()
                    .and_then(brush_border)
                    .unwrap_or((None, 0.0));
                output.push(HostPaintCommand::quad(
                    frame,
                    clip_frame,
                    element.z_index,
                    background_color,
                    border_color,
                    border_width,
                    command.style.corner_radius.max(0.0),
                    element.effects.opacity,
                ));
            }
        }
        UiPaintPayload::Text { text } => {
            push_text_paint_commands(output, command, text, frame, clip_frame, element.z_index)
        }
    }
}
