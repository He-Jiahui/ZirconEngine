use super::ChromeCommandStream;
use crate::ui::retained_host::host_contract::data::FrameRect;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;

mod commands;

use commands::paint_chrome_command;

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

fn paint_chrome_command_stream_into_frame(frame: &mut HostRgbaFrame, stream: &ChromeCommandStream) {
    let commands = stream.commands();
    if commands
        .windows(2)
        .all(|pair| pair[0].z_index <= pair[1].z_index)
    {
        for command in commands {
            paint_chrome_command(frame, stream, command);
        }
        return;
    }

    let mut ordered = commands.iter().enumerate().collect::<Vec<_>>();
    ordered.sort_by_key(|(index, command)| (command.z_index, *index));
    for (_, command) in ordered {
        paint_chrome_command(frame, stream, command);
    }
}
