use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::command::HostPaintCommand;
use super::dispatch::draw_host_paint_command;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_host_paint_commands(
    frame: &mut HostRgbaFrame,
    commands: &[HostPaintCommand],
) -> bool {
    let mut ordered = {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_collect_order");
        commands.iter().enumerate().collect::<Vec<_>>()
    };
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_sort");
        ordered.sort_by_key(|(index, command)| (command.z_index, *index));
    }

    let mut drew_any = false;
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_draw_ordered");
        for (_, command) in ordered {
            drew_any |= draw_host_paint_command(frame, command);
        }
    }
    drew_any
}
