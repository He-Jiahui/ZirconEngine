use super::super::data::FrameRect;
use super::frame::HostRgbaFrame;
use super::geometry::{is_visible_frame, PixelRect};

pub(super) fn draw_rect(frame: &mut HostRgbaFrame, rect: FrameRect, color: [u8; 4]) {
    draw_rect_clipped(frame, rect, None, color);
}

pub(super) fn draw_rect_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    color: [u8; 4],
) {
    let Some(rect) = PixelRect::from_frame(&rect, clip, frame.width(), frame.height()) else {
        return;
    };
    for y in rect.y0..rect.y1 {
        for x in rect.x0..rect.x1 {
            write_pixel(frame, x, y, color);
        }
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
    let end = x.saturating_add(width).min(frame.width());
    for px in x..end {
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
    let seed = text.bytes().fold(0_u32, |sum, byte| sum + byte as u32);
    let bar_count = (text.len().max(1).min(18) as u32).max(3);
    for index in 0..bar_count {
        let width = 4.0 + ((seed + index * 7) % 9) as f32;
        draw_rect_clipped(
            frame,
            FrameRect {
                x: x + index as f32 * 8.0,
                y,
                width,
                height: 3.0,
            },
            clip,
            color,
        );
    }
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
    draw_text_bars_clipped(
        frame,
        target.x + 12.0,
        target.y + target.height.min(28.0) * 0.5,
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
