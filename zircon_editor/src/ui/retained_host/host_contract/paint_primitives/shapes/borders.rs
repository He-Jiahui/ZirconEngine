use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::{is_visible_frame, PixelRect};
use super::super::clip::effective_clip;
use super::super::pixels::{clamped_corner_radius, fill_rounded_border_pixels, inset_frame};
use super::rects::draw_rect_clipped;

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
