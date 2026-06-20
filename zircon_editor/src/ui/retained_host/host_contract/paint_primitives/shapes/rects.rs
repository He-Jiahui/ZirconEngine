use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;
use super::super::clip::effective_clip;
use super::super::pixels::{clamped_corner_radius, fill_pixel_rect, fill_rounded_pixel_rect};

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

fn draw_solid_rect_clipped(
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
        let recorded_frame = if corner_radius > 0.0 {
            rect.clone()
        } else {
            target.to_frame()
        };
        frame.record_quad(recorded_frame, effective_clip, color, corner_radius);
        if frame.record_only() {
            return;
        }
    }
    if corner_radius > 0.0 {
        fill_rounded_pixel_rect(frame, &target, &rect, color, corner_radius);
    } else {
        fill_pixel_rect(frame, &target, color);
    }
}

trait PixelRectExt {
    fn to_frame(&self) -> FrameRect;
}

impl PixelRectExt for PixelRect {
    fn to_frame(&self) -> FrameRect {
        FrameRect {
            x: self.x0 as f32,
            y: self.y0 as f32,
            width: self.x1.saturating_sub(self.x0) as f32,
            height: self.y1.saturating_sub(self.y0) as f32,
        }
    }
}
