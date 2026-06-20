use zircon_runtime_interface::ui::surface::{UiPaintElement, UiPaintPayload, UiRenderCommand};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_geometry::is_visible_frame;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::style::frame_from_ui;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::text::push_text_paint_commands;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::brush_payload::push_brush_paint_commands;

pub(super) fn push_runtime_paint_element(
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
        UiPaintPayload::Brush { brushes } => push_brush_paint_commands(
            output,
            command,
            brushes,
            frame,
            clip_frame,
            element.z_index,
            element.effects.opacity,
        ),
        UiPaintPayload::Text { text } => {
            push_text_paint_commands(output, command, text, frame, clip_frame, element.z_index)
        }
    }
}
