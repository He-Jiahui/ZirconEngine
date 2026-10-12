use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::{is_visible_frame, PixelRect};
use super::super::super::clip::effective_clip;
use super::super::super::pixels::{
    clamped_corner_radius, fill_rect_border_pixels, fill_rounded_border_pixels,
    fill_rounded_box_pixels,
};

pub(in crate::ui::retained_host::host_contract) fn draw_rounded_box_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    fill_color: [u8; 4],
    border_color: [u8; 4],
    border_width: f32,
    corner_radius: f32,
) {
    if fill_color[3] == 0 || border_color[3] == 0 || !is_visible_frame(&rect) {
        return;
    }
    let border_width = clamped_border_width(&rect, border_width);
    if border_width <= 0.0 {
        return;
    }
    let corner_radius = clamped_corner_radius(&rect, corner_radius);
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
        frame.record_quad(
            rect.clone(),
            effective_clip.clone(),
            fill_color,
            corner_radius,
        );
        frame.record_border(
            rect.clone(),
            effective_clip,
            border_color,
            border_width,
            corner_radius,
        );
        if frame.record_only() {
            return;
        }
    }
    fill_rounded_box_pixels(
        frame,
        &target,
        &rect,
        fill_color,
        border_color,
        border_width,
        corner_radius,
    );
}

pub(in crate::ui::retained_host::host_contract) fn draw_rounded_border_clipped(
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
    let border_width = clamped_border_width(&rect, border_width);
    if border_width <= 0.0 {
        return;
    }
    let corner_radius = clamped_corner_radius(&rect, corner_radius);
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
    if corner_radius <= 0.0 {
        fill_rect_border_pixels(frame, &target, &rect, color, border_width);
    } else {
        fill_rounded_border_pixels(frame, &target, &rect, color, border_width, corner_radius);
    }
}

fn clamped_border_width(rect: &FrameRect, border_width: f32) -> f32 {
    if !border_width.is_finite() {
        return 0.0;
    }
    border_width
        .max(0.0)
        .min(8.0)
        .min(rect.width.min(rect.height).max(0.0) * 0.5)
}
