use super::data::FrameRect;
use super::paint_frame::HostRgbaFrame;

mod clip;
mod image;
mod lines;
mod pixels;
mod shapes;
mod text_markers;

#[cfg(test)]
mod tests;

pub(in crate::ui::retained_host::host_contract) use image::{
    draw_rgba_image_clipped_with_atlas, draw_rgba_image_clipped_with_resource_key,
};

pub(super) fn draw_rect(frame: &mut HostRgbaFrame, rect: FrameRect, color: [u8; 4]) {
    shapes::draw_rect(frame, rect, color);
}

pub(in crate::ui::retained_host::host_contract) fn draw_rect_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    shapes::draw_rect_clipped(frame, rect, clip, color);
}

pub(in crate::ui::retained_host::host_contract) fn draw_rounded_rect_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    corner_radius: f32,
) {
    shapes::draw_rounded_rect_clipped(frame, rect, clip, color, corner_radius);
}

pub(super) fn draw_border(frame: &mut HostRgbaFrame, rect: FrameRect, color: [u8; 4]) {
    shapes::draw_border(frame, rect, color);
}

pub(in crate::ui::retained_host::host_contract) fn draw_border_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    shapes::draw_border_clipped(frame, rect, clip, color);
}

pub(in crate::ui::retained_host::host_contract) fn draw_rounded_border_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    border_width: f32,
    corner_radius: f32,
) {
    shapes::draw_rounded_border_clipped(frame, rect, clip, color, border_width, corner_radius);
}

pub(super) fn draw_separator_line(
    frame: &mut HostRgbaFrame,
    x: u32,
    y: u32,
    width: u32,
    color: [u8; 4],
) {
    lines::draw_separator_line(frame, x, y, width, color);
}

pub(super) fn draw_text_bars(
    frame: &mut HostRgbaFrame,
    x: f32,
    y: f32,
    text: &str,
    color: [u8; 4],
) {
    text_markers::draw_text_bars(frame, x, y, text, color);
}

pub(super) fn draw_text_bars_clipped(
    frame: &mut HostRgbaFrame,
    x: f32,
    y: f32,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    text_markers::draw_text_bars_clipped(frame, x, y, text, clip, color);
}

pub(super) fn draw_label_marker(
    frame: &mut HostRgbaFrame,
    target: &FrameRect,
    label: &str,
    color: [u8; 4],
) {
    text_markers::draw_label_marker(frame, target, label, color);
}
