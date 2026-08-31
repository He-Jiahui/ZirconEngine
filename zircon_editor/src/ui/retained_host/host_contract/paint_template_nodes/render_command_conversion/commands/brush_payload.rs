use zircon_runtime_interface::ui::surface::{UiBrushSet, UiRenderCommand};

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::brush::{
    brush_border, brush_fill_color, image_brush_resource,
};
use crate::ui::retained_host::host_contract::paint_template_nodes::render_command_conversion::image::push_image_resource_command;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

pub(super) fn push_brush_paint_commands(
    output: &mut Vec<HostPaintCommand>,
    command: &UiRenderCommand,
    brushes: &UiBrushSet,
    frame: FrameRect,
    clip_frame: Option<FrameRect>,
    z_index: i32,
    opacity: f32,
) {
    if let Some((image_brush, physical_pixel_size)) =
        brushes.fill.as_ref().and_then(image_brush_resource)
    {
        push_image_resource_command(
            output,
            image_brush,
            physical_pixel_size,
            frame,
            clip_frame,
            z_index,
            opacity,
        );
        return;
    }

    let background_color = brushes.fill.as_ref().and_then(brush_fill_color);
    let (border_color, border_width) = brushes
        .border
        .as_ref()
        .and_then(brush_border)
        .unwrap_or((None, 0.0));
    output.push(HostPaintCommand::quad(
        frame,
        clip_frame,
        z_index,
        background_color,
        border_color,
        border_width,
        command.style.corner_radius.max(0.0),
        opacity,
    ));
}
