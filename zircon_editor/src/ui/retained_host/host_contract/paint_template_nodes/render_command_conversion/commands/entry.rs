use zircon_runtime_interface::ui::surface::UiRenderCommand;

use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_template_nodes::render_commands::HostPaintCommand;

use super::command::push_runtime_command;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn runtime_render_commands_to_host(
    commands: &[UiRenderCommand],
    clip_frame: Option<&FrameRect>,
) -> Vec<HostPaintCommand> {
    let mut host_commands = Vec::new();
    for command in commands {
        push_runtime_command(&mut host_commands, command, clip_frame);
    }
    host_commands
}
