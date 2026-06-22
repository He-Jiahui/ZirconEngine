use super::super::super::super::ChromeCommand;
use crate::ui::retained_host::host_contract::paint_frame::HostRgbaFrame;
use crate::ui::retained_host::host_contract::paint_primitives::{
    draw_rect_clipped, draw_rounded_rect_clipped,
};

pub(in super::super) fn paint_quad_command(
    frame: &mut HostRgbaFrame,
    command: &ChromeCommand,
    color: [u8; 4],
    corner_radius: f32,
) {
    if corner_radius > 0.0 {
        draw_rounded_rect_clipped(
            frame,
            command.frame.clone(),
            command.clip.as_ref(),
            color,
            corner_radius,
        )
    } else {
        draw_rect_clipped(frame, command.frame.clone(), command.clip.as_ref(), color)
    }
}
