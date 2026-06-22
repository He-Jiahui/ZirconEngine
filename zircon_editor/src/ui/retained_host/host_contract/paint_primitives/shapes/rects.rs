mod solid;

use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use solid::draw_solid_rect_clipped;

pub(in crate::ui::retained_host::host_contract) fn draw_rect(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    color: [u8; 4],
) {
    draw_rect_clipped(frame, rect, None, color);
}

pub(in crate::ui::retained_host::host_contract) fn draw_rect_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    draw_solid_rect_clipped(frame, rect, clip, color, 0.0);
}

pub(in crate::ui::retained_host::host_contract) fn draw_rounded_rect_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    corner_radius: f32,
) {
    if color[3] == 0 {
        return;
    }
    draw_solid_rect_clipped(frame, rect, clip, color, corner_radius.max(0.0));
}
