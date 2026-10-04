use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_primitives::draw_rounded_border_clipped;

pub(in crate::ui::retained_host::host_contract::paint_template_nodes) fn draw_border_width(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    border_width: f32,
) {
    draw_rounded_border_clipped(frame, rect.clone(), clip, color, border_width, 0.0);
}

#[cfg(test)]
#[path = "tests/border.rs"]
mod tests;
