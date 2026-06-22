mod rect;

use super::super::super::super::ChromeCommand;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_primitives::draw_rounded_border_clipped;

use rect::paint_rect_border_command;

pub(in super::super) fn paint_border_command(
    frame: &mut HostRgbaFrame,
    command: &ChromeCommand,
    color: [u8; 4],
    width: f32,
    corner_radius: f32,
) {
    if corner_radius > 0.0 {
        draw_rounded_border_clipped(
            frame,
            command.frame.clone(),
            command.clip.as_ref(),
            color,
            width,
            corner_radius,
        )
    } else {
        paint_rect_border_command(frame, &command.frame, command.clip.as_ref(), color, width)
    }
}
