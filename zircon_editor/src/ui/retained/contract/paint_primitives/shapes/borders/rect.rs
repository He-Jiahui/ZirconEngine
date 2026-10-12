use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::draw_rounded_border_clipped;

pub(in crate::ui::retained_host::host_contract) fn draw_border(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    color: [u8; 4],
) {
    draw_border_clipped(frame, rect, None, color);
}

pub(in crate::ui::retained_host::host_contract) fn draw_border_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    draw_rounded_border_clipped(frame, rect, clip, color, 1.0, 0.0);
}
