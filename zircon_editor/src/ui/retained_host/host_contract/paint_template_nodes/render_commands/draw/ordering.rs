use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::evidence::{record_box_shadow_command, record_drawn_command};
use super::super::command::HostPaintCommand;
use super::dispatch::draw_host_paint_command;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

/// 已单调排列的命令直接绘制；乱序时稳定排序，保持同层模板节点的原提交顺序。
pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_host_paint_commands(
    frame: &mut HostRgbaFrame,
    commands: &[HostPaintCommand],
) -> bool {
    if z_indices_are_ordered(commands.iter().map(|command| command.z_index)) {
        return commands.iter().fold(false, |drew_any, command| {
            record_box_shadow_command(command);
            let drew = draw_host_paint_command(frame, command);
            if drew {
                record_drawn_command(command);
            }
            drew || drew_any
        });
    }

    record_current_ui_perf_counter(UiPerfCounter::FallbackSortCount, 1.0);
    let ordered = fallback_ordered_commands(commands);

    let mut drew_any = false;
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_draw_ordered");
        for command in ordered {
            record_box_shadow_command(command);
            let drew = draw_host_paint_command(frame, command);
            if drew {
                record_drawn_command(command);
            }
            drew_any |= drew;
        }
    }
    drew_any
}

fn fallback_ordered_commands(commands: &[HostPaintCommand]) -> Vec<&HostPaintCommand> {
    let mut ordered = {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_collect_order");
        commands.iter().collect::<Vec<_>>()
    };
    {
        zircon_runtime::profile_scope!("editor", "host_painter", "paint_commands_sort");
        ordered.sort_by_key(|command| command.z_index);
    }
    ordered
}

fn z_indices_are_ordered(mut indices: impl Iterator<Item = i32>) -> bool {
    let Some(mut previous) = indices.next() else {
        return true;
    };
    indices.all(|current| {
        let ordered = previous <= current;
        previous = current;
        ordered
    })
}

#[cfg(test)]
#[path = "tests/ordering.rs"]
mod tests;

#[cfg(test)]
#[path = "ordering/tests/stable_z_sort_tests.rs"]
mod stable_z_sort_tests;
