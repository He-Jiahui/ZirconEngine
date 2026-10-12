use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::super::super::clip::effective_clip;
use super::super::super::pixels::{
    clamped_corner_radius, fill_rect_pixel_coverage, fill_rounded_pixel_rect,
};

pub(super) fn draw_solid_rect_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    corner_radius: f32,
) {
    if color[3] == 0 {
        return;
    }
    let Some(effective_clip) = effective_clip(frame, clip) else {
        return;
    };
    let Some(target) = PixelRect::from_frame(
        &rect,
        effective_clip.as_ref(),
        frame.width(),
        frame.height(),
    ) else {
        return;
    };
    let corner_radius = clamped_corner_radius(&rect, corner_radius);
    if frame.is_recording() {
        frame.record_quad(rect.clone(), effective_clip, color, corner_radius);
        if frame.record_only() {
            return;
        }
    }
    if corner_radius > 0.0 {
        fill_rounded_pixel_rect(frame, &target, &rect, color, corner_radius);
    } else {
        fill_rect_pixel_coverage(frame, &target, &rect, color);
    }
}
