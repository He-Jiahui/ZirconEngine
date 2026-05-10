use super::super::data::FrameRect;
use super::frame::HostRgbaFrame;
use super::geometry::{intersect, is_visible_frame, PixelRect};
use super::text::draw_text;

pub(super) fn draw_rect(frame: &mut HostRgbaFrame, rect: FrameRect, color: [u8; 4]) {
    draw_rect_clipped(frame, rect, None, color);
}

pub(super) fn draw_rect_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    let effective_clip = effective_clip(frame, clip);
    let Some(rect) = PixelRect::from_frame(
        &rect,
        effective_clip.as_ref(),
        frame.width(),
        frame.height(),
    ) else {
        return;
    };
    for y in rect.y0..rect.y1 {
        for x in rect.x0..rect.x1 {
            write_pixel(frame, x, y, color);
        }
    }
}

pub(super) fn draw_rgba_image_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) -> bool {
    if image_width == 0
        || image_height == 0
        || rgba.len() != image_width as usize * image_height as usize * 4
    {
        return false;
    }
    let effective_clip = effective_clip(frame, clip);
    let Some(target) = PixelRect::from_frame(
        &rect,
        effective_clip.as_ref(),
        frame.width(),
        frame.height(),
    ) else {
        return false;
    };
    let rect_width = rect.width.max(1.0);
    let rect_height = rect.height.max(1.0);

    for y in target.y0..target.y1 {
        let source_y = (((y as f32 + 0.5 - rect.y) / rect_height) * image_height as f32)
            .floor()
            .max(0.0)
            .min((image_height - 1) as f32) as u32;
        for x in target.x0..target.x1 {
            let source_x = (((x as f32 + 0.5 - rect.x) / rect_width) * image_width as f32)
                .floor()
                .max(0.0)
                .min((image_width - 1) as f32) as u32;
            let offset = ((source_y as usize * image_width as usize) + source_x as usize) * 4;
            write_pixel(
                frame,
                x,
                y,
                [
                    rgba[offset],
                    rgba[offset + 1],
                    rgba[offset + 2],
                    rgba[offset + 3],
                ],
            );
        }
    }
    true
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

pub(super) fn draw_separator_line(
    frame: &mut HostRgbaFrame,
    x: u32,
    y: u32,
    width: u32,
    color: [u8; 4],
) {
    if y >= frame.height() {
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
    for px in start..end {
        write_pixel(frame, px, y, color);
    }
}

pub(super) fn draw_text_bars(
    frame: &mut HostRgbaFrame,
    x: f32,
    y: f32,
    text: &str,
    color: [u8; 4],
) {
    draw_text_bars_clipped(frame, x, y, text, None, color);
}

pub(super) fn draw_text_bars_clipped(
    frame: &mut HostRgbaFrame,
    x: f32,
    y: f32,
    text: &str,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    draw_text(
        frame,
        FrameRect {
            x,
            y,
            width: (text.chars().count() as f32 * 8.0).max(1.0),
            height: 16.0,
        },
        text,
        clip,
        color,
    );
}

pub(super) fn draw_label_marker(
    frame: &mut HostRgbaFrame,
    target: &FrameRect,
    label: &str,
    color: [u8; 4],
) {
    if !is_visible_frame(target) {
        return;
    }
    draw_text(
        frame,
        FrameRect {
            x: target.x + 12.0,
            y: target.y + ((target.height - 16.0).max(0.0) * 0.5),
            width: (target.width - 24.0).max(1.0),
            height: target.height.min(18.0).max(1.0),
        },
        label,
        Some(target),
        color,
    );
}

fn write_pixel(frame: &mut HostRgbaFrame, x: u32, y: u32, color: [u8; 4]) {
    if color[3] == 0 {
        return;
    }
    let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
    let bytes = frame.as_bytes_mut();
    if color[3] == 255 {
        bytes[offset..offset + 4].copy_from_slice(&color);
        return;
    }

    let alpha = color[3] as u32;
    let inverse = 255 - alpha;
    for channel in 0..3 {
        let source = color[channel] as u32;
        let destination = bytes[offset + channel] as u32;
        bytes[offset + channel] = ((source * alpha + destination * inverse) / 255) as u8;
    }
    bytes[offset + 3] = 255;
}

fn effective_clip(frame: &HostRgbaFrame, clip: Option<&FrameRect>) -> Option<FrameRect> {
    match (frame.paint_clip(), clip) {
        (Some(active), Some(clip)) => intersect(active, clip),
        (Some(active), None) => Some(active.clone()),
        (None, Some(clip)) => Some(clip.clone()),
        (None, None) => None,
    }
}
