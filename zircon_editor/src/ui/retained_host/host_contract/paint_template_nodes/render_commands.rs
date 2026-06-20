use zircon_runtime_interface::ui::surface::UiRenderCommand;

use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::render_command_conversion::runtime_render_commands_to_host;

mod command;
mod draw;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use command::HostPaintCommand;
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) use draw::draw_host_paint_commands;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_runtime_render_commands(
    frame: &mut HostRgbaFrame,
    commands: &[UiRenderCommand],
    clip_frame: Option<&FrameRect>,
) -> bool {
    let host_commands = runtime_render_commands_to_host(commands, clip_frame);
    draw_host_paint_commands(frame, &host_commands)
}

#[cfg(test)]
pub(crate) fn paint_runtime_render_commands_for_test(
    width: u32,
    height: u32,
    commands: &[UiRenderCommand],
) -> Vec<u8> {
    let mut frame = HostRgbaFrame::filled(width, height, [0, 0, 0, 255]);
    draw_runtime_render_commands(&mut frame, commands, None);
    frame.into_bytes()
}
