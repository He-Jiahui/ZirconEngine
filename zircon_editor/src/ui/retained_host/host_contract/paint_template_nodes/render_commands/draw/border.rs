use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::inset;
use super::super::super::super::paint_primitives::draw_border_clipped;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_border_width(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    border_width: f32,
) {
    let pixel_width = border_width.ceil().max(1.0).min(8.0) as u32;
    for offset in 0..pixel_width {
        draw_border_clipped(frame, inset(rect, offset as f32), clip, color);
    }
}
