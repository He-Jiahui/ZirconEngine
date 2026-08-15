use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::{is_visible_frame, PixelRect};
use super::super::super::clip::effective_clip;
use super::super::rects::draw_rect_clipped;

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
    if color[3] == 0 || !is_visible_frame(&rect) {
        return;
    }
    if frame.record_only() {
        let Some(effective_clip) = effective_clip(frame, clip) else {
            return;
        };
        if PixelRect::from_frame(
            &rect,
            effective_clip.as_ref(),
            frame.width(),
            frame.height(),
        )
        .is_none()
        {
            return;
        }
        frame.record_border(rect, effective_clip, color, 1.0, 0.0);
        return;
    }
    draw_rect_clipped(frame, border_top(&rect), clip, color);
    draw_rect_clipped(frame, border_bottom(&rect), clip, color);
    draw_rect_clipped(frame, border_left(&rect), clip, color);
    draw_rect_clipped(frame, border_right(&rect), clip, color);
}

fn border_top(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: rect.width,
        height: 1.0,
    }
}

fn border_bottom(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y + rect.height - 1.0,
        width: rect.width,
        height: 1.0,
    }
}

fn border_left(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x,
        y: rect.y,
        width: 1.0,
        height: rect.height,
    }
}

fn border_right(rect: &FrameRect) -> FrameRect {
    FrameRect {
        x: rect.x + rect.width - 1.0,
        y: rect.y,
        width: 1.0,
        height: rect.height,
    }
}
