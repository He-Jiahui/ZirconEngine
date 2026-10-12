use super::super::data::FrameRect;
use super::super::paint_frame::HostRgbaFrame;
use super::super::paint_geometry::PixelRect;
use super::pixels::fill_pixel_span;

pub(in crate::ui::retained_host::host_contract) fn draw_separator_line(
    frame: &mut HostRgbaFrame,
    x: u32,
    y: u32,
    width: u32,
    color: [u8; 4],
) {
    if y >= frame.height() || color[3] == 0 {
        return;
    }
    let mut start = x.min(frame.width());
    let mut end = x.saturating_add(width).min(frame.width());
    if let Some(clip) = frame.paint_clip() {
        let Some(clip_rect) = PixelRect::from_frame(clip, None, frame.width(), frame.height())
        else {
            return;
        };
        if y < clip_rect.y0 || y >= clip_rect.y1 {
            return;
        }
        start = start.max(clip_rect.x0);
        end = end.min(clip_rect.x1);
    }
    if start >= end {
        return;
    }

    if frame.is_recording() {
        frame.record_quad(
            FrameRect {
                x: start as f32,
                y: y as f32,
                width: end.saturating_sub(start) as f32,
                height: 1.0,
            },
            frame.paint_clip().cloned(),
            color,
            0.0,
        );
        if frame.record_only() {
            return;
        }
    }

    let frame_width = frame.width() as usize;
    let offset = ((y as usize * frame_width) + start as usize) * 4;
    let end_offset = ((y as usize * frame_width) + end as usize) * 4;
    fill_pixel_span(&mut frame.as_bytes_mut()[offset..end_offset], color);
}
