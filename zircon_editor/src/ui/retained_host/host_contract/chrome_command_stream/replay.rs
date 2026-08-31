use super::ChromeCommandStream;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::ui_perf::{record_current_ui_perf_counter, UiPerfCounter};

mod commands;

use commands::{paint_chrome_command, paint_chrome_command_pair};

pub(in crate::ui::retained_host::host_contract) fn paint_chrome_command_stream_to_frame(
    width: u32,
    height: u32,
    stream: &ChromeCommandStream,
) -> HostRgbaFrame {
    let mut frame = HostRgbaFrame::filled(width, height, [0, 0, 0, 255]);
    paint_chrome_command_stream_into_frame(&mut frame, stream);
    frame
}

pub(in crate::ui::retained_host::host_contract) fn repaint_chrome_command_stream_region(
    frame: &mut HostRgbaFrame,
    stream: &ChromeCommandStream,
) -> Option<FrameRect> {
    let damage = stream.damage().cloned()?;
    let previous_clip = frame.replace_paint_clip(Some(damage.clone()));
    paint_chrome_command_stream_into_frame(frame, stream);
    frame.replace_paint_clip(previous_clip);
    Some(damage)
}

fn fallback_ordered_commands(commands: &[super::ChromeCommand]) -> Vec<&super::ChromeCommand> {
    let mut ordered = commands.iter().collect::<Vec<_>>();
    ordered.sort_by_key(|command| command.z_index);
    ordered
}

fn paint_chrome_command_stream_into_frame(frame: &mut HostRgbaFrame, stream: &ChromeCommandStream) {
    let commands = stream.commands();
    if commands
        .windows(2)
        .all(|pair| pair[0].z_index <= pair[1].z_index)
    {
        paint_ordered_commands(frame, stream, commands.iter());
        return;
    }

    record_current_ui_perf_counter(UiPerfCounter::FallbackSortCount, 1.0);
    paint_ordered_commands(frame, stream, fallback_ordered_commands(commands));
}

fn paint_ordered_commands<'a>(
    frame: &mut HostRgbaFrame,
    stream: &ChromeCommandStream,
    commands: impl IntoIterator<Item = &'a super::ChromeCommand>,
) {
    let mut commands = commands.into_iter().peekable();
    while let Some(command) = commands.next() {
        if commands
            .peek()
            .is_some_and(|next| paint_chrome_command_pair(frame, command, next))
        {
            commands.next();
            continue;
        }
        paint_chrome_command(frame, stream, command);
    }
}

#[cfg(test)]
#[path = "replay/stable_z_sort_tests.rs"]
mod stable_z_sort_tests;
