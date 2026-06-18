use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::{is_visible_frame, PixelRect};
use super::clip::effective_clip;
use super::pixels::{
    clamped_corner_radius, fill_pixel_rect, fill_rounded_border_pixels, fill_rounded_pixel_rect,
    inset_frame,
};

pub(super) fn draw_rect(frame: &mut HostRgbaFrame, rect: FrameRect, color: [u8; 4]) {
    draw_rect_clipped(frame, rect, None, color);
}

pub(super) fn draw_rect_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    draw_solid_rect_clipped(frame, rect, clip, color, 0.0);
}

pub(super) fn draw_rounded_rect_clipped(
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

pub(super) fn draw_border(frame: &mut HostRgbaFrame, rect: FrameRect, color: [u8; 4]) {
    draw_border_clipped(frame, rect, None, color);
}

pub(super) fn draw_border_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    if !is_visible_frame(&rect) {
        return;
    }
    draw_rect_clipped(
        frame,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: rect.width,
            height: 1.0,
        },
        clip,
        color,
    );
    draw_rect_clipped(
        frame,
        FrameRect {
            x: rect.x,
            y: rect.y + rect.height - 1.0,
            width: rect.width,
            height: 1.0,
        },
        clip,
        color,
    );
    draw_rect_clipped(
        frame,
        FrameRect {
            x: rect.x,
            y: rect.y,
            width: 1.0,
            height: rect.height,
        },
        clip,
        color,
    );
    draw_rect_clipped(
        frame,
        FrameRect {
            x: rect.x + rect.width - 1.0,
            y: rect.y,
            width: 1.0,
            height: rect.height,
        },
        clip,
        color,
    );
}

pub(super) fn draw_rounded_border_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
    border_width: f32,
    corner_radius: f32,
) {
    if color[3] == 0 || !is_visible_frame(&rect) {
        return;
    }
    let border_width = border_width.ceil().max(1.0).min(8.0);
    let corner_radius = clamped_corner_radius(&rect, corner_radius);
    if corner_radius <= 0.0 {
        for offset in 0..(border_width as u32) {
            draw_border_clipped(frame, inset_frame(&rect, offset as f32), clip, color);
        }
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
    if frame.is_recording() {
        frame.record_border(
            rect.clone(),
            effective_clip,
            color,
            border_width,
            corner_radius,
        );
        if frame.record_only() {
            return;
        }
    }
    fill_rounded_border_pixels(frame, &target, &rect, color, border_width, corner_radius);
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
