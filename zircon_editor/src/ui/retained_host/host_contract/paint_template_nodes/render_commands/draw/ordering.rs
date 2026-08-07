use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::command::HostPaintCommand;
use super::dispatch::draw_host_paint_command;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_host_paint_commands(
    frame: &mut HostRgbaFrame,
    commands: &[HostPaintCommand],
) -> bool {
    if z_indices_are_ordered(commands.iter().map(|command| command.z_index)) {
        return commands.iter().fold(false, |drew_any, command| {
            draw_host_paint_command(frame, command) || drew_any
        });
    }

    record_current_ui_perf_counter(UiPerfCounter::FallbackSortCount, 1.0);
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
mod tests {
    use super::z_indices_are_ordered;

    #[test]
    fn ordered_and_equal_layers_stay_on_the_zero_sort_path() {
        assert!(z_indices_are_ordered([0, 0, 2, 8].into_iter()));
        assert!(z_indices_are_ordered([].into_iter()));
    }

    #[test]
    fn descending_layer_requires_the_fallback_sort() {
        assert!(!z_indices_are_ordered([0, 4, 3, 8].into_iter()));
    }
}
