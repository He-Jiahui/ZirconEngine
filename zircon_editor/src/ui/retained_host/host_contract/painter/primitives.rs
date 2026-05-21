use super::super::data::FrameRect;
use super::frame::HostRgbaFrame;
use super::geometry::{intersect, is_visible_frame, PixelRect};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::text::draw_text;

pub(super) fn draw_rect(frame: &mut HostRgbaFrame, rect: FrameRect, color: [u8; 4]) {
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

pub(in crate::ui::retained_host::host_contract) fn draw_rgba_image_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) -> bool {
    draw_rgba_image_clipped_with_key(frame, rect, clip, None, image_width, image_height, rgba)
}

pub(in crate::ui::retained_host::host_contract) fn draw_rgba_image_clipped_with_resource_key(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    resource_key: &str,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) -> bool {
    draw_rgba_image_clipped_with_key(
        frame,
        rect,
        clip,
        Some(resource_key),
        image_width,
        image_height,
        rgba,
    )
}

fn draw_rgba_image_clipped_with_key(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    resource_key: Option<&str>,
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
    let Some(effective_clip) = effective_clip(frame, clip) else {
        return false;
    };
    let Some(target) = PixelRect::from_frame(
        &rect,
        effective_clip.as_ref(),
        frame.width(),
        frame.height(),
    ) else {
        return false;
    };
    if frame.is_recording() {
        let resource_key = resource_key
            .map(str::to_string)
            .unwrap_or_else(|| rgba_resource_key(image_width, image_height, rgba));
        frame.record_image(
            rect.clone(),
            effective_clip.clone(),
            resource_key,
            image_width,
            image_height,
            Some(rgba.to_vec()),
        );
        if frame.record_only() {
            return true;
        }
    }
    if try_copy_opaque_identity_image_rows(frame, &rect, &target, image_width, image_height, rgba) {
        return true;
    }

    draw_scaled_rgba_image_pixels(frame, &rect, &target, image_width, image_height, rgba);
    true
}

fn rgba_resource_key(image_width: u32, image_height: u32, rgba: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    image_width.hash(&mut hasher);
    image_height.hash(&mut hasher);
    rgba.hash(&mut hasher);
    format!("rgba:{image_width}x{image_height}:{:016x}", hasher.finish())
}

fn draw_scaled_rgba_image_pixels(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    target: &PixelRect,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) {
    let rect_width = rect.width.max(1.0);
    let rect_height = rect.height.max(1.0);
    let frame_width = frame.width() as usize;
    let bytes = frame.as_bytes_mut();

    for y in target.y0..target.y1 {
        let source_y = (((y as f32 + 0.5 - rect.y) / rect_height) * image_height as f32)
            .floor()
            .max(0.0)
            .min((image_height - 1) as f32) as u32;
        let destination_row = y as usize * frame_width * 4;
        for x in target.x0..target.x1 {
            let source_x = (((x as f32 + 0.5 - rect.x) / rect_width) * image_width as f32)
                .floor()
                .max(0.0)
                .min((image_width - 1) as f32) as u32;
            let source_offset =
                ((source_y as usize * image_width as usize) + source_x as usize) * 4;
            let destination_offset = destination_row + x as usize * 4;
            write_rgba_pixel(bytes, destination_offset, rgba, source_offset);
        }
    }
}

fn try_copy_opaque_identity_image_rows(
    frame: &mut HostRgbaFrame,
    rect: &FrameRect,
    target: &PixelRect,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) -> bool {
    if !is_identity_image_mapping(rect, image_width, image_height) {
        return false;
    }

    let source_x0 = (target.x0 as i64 - rect.x as i64).max(0) as usize;
    let source_y0 = (target.y0 as i64 - rect.y as i64).max(0) as usize;
    let width = (target.x1 - target.x0) as usize;
    let height = (target.y1 - target.y0) as usize;
    let image_width = image_width as usize;
    let image_height = image_height as usize;
    if width == 0
        || height == 0
        || source_x0 + width > image_width
        || source_y0 + height > image_height
    {
        return false;
    }

    for row in 0..height {
        let source_start = (((source_y0 + row) * image_width) + source_x0) * 4;
        let source_end = source_start + width * 4;
        if !rgba[source_start..source_end]
            .chunks_exact(4)
            .all(|pixel| pixel[3] == 255)
        {
            return false;
        }
    }

    let frame_width = frame.width() as usize;
    let bytes = frame.as_bytes_mut();
    for row in 0..height {
        let source_start = (((source_y0 + row) * image_width) + source_x0) * 4;
        let source_end = source_start + width * 4;
        let destination_start =
            (((target.y0 as usize + row) * frame_width) + target.x0 as usize) * 4;
        let destination_end = destination_start + width * 4;
        bytes[destination_start..destination_end].copy_from_slice(&rgba[source_start..source_end]);
    }
    true
}

fn is_identity_image_mapping(rect: &FrameRect, image_width: u32, image_height: u32) -> bool {
    rect.x.fract().abs() <= f32::EPSILON
        && rect.y.fract().abs() <= f32::EPSILON
        && (rect.width - image_width as f32).abs() <= f32::EPSILON
        && (rect.height - image_height as f32).abs() <= f32::EPSILON
}

#[inline]
fn write_rgba_pixel(
    bytes: &mut [u8],
    destination_offset: usize,
    rgba: &[u8],
    source_offset: usize,
) {
    let alpha = rgba[source_offset + 3];
    if alpha == 0 {
        return;
    }
    if alpha == 255 {
        bytes[destination_offset] = rgba[source_offset];
        bytes[destination_offset + 1] = rgba[source_offset + 1];
        bytes[destination_offset + 2] = rgba[source_offset + 2];
        bytes[destination_offset + 3] = 255;
        return;
    }

    let alpha = alpha as u32;
    let inverse = 255 - alpha;
    for channel in 0..3 {
        let source = rgba[source_offset + channel] as u32;
        let destination = bytes[destination_offset + channel] as u32;
        bytes[destination_offset + channel] =
            ((source * alpha + destination * inverse) / 255) as u8;
    }
    bytes[destination_offset + 3] = 255;
}

pub(super) fn draw_border(frame: &mut HostRgbaFrame, rect: FrameRect, color: [u8; 4]) {
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

pub(super) fn draw_separator_line(
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

fn fill_pixel_rect(frame: &mut HostRgbaFrame, rect: &PixelRect, color: [u8; 4]) {
    let frame_width = frame.width() as usize;
    let x0 = rect.x0 as usize;
    let x1 = rect.x1 as usize;
    let bytes = frame.as_bytes_mut();

    for y in rect.y0 as usize..rect.y1 as usize {
        let row_start = ((y * frame_width) + x0) * 4;
        let row_end = ((y * frame_width) + x1) * 4;
        fill_pixel_span(&mut bytes[row_start..row_end], color);
    }
}

fn fill_rounded_pixel_rect(
    frame: &mut HostRgbaFrame,
    target: &PixelRect,
    rect: &FrameRect,
    color: [u8; 4],
    corner_radius: f32,
) {
    for y in target.y0..target.y1 {
        for x in target.x0..target.x1 {
            if rounded_rect_contains_pixel(x, y, rect, corner_radius) {
                write_pixel(frame, x, y, color);
            }
        }
    }
}

fn fill_rounded_border_pixels(
    frame: &mut HostRgbaFrame,
    target: &PixelRect,
    rect: &FrameRect,
    color: [u8; 4],
    border_width: f32,
    corner_radius: f32,
) {
    let inner = inset_frame(rect, border_width);
    let has_inner = is_visible_frame(&inner);
    let inner_radius = (corner_radius - border_width).max(0.0);
    for y in target.y0..target.y1 {
        for x in target.x0..target.x1 {
            if !rounded_rect_contains_pixel(x, y, rect, corner_radius) {
                continue;
            }
            if has_inner && rounded_rect_contains_pixel(x, y, &inner, inner_radius) {
                continue;
            }
            write_pixel(frame, x, y, color);
        }
    }
}

fn fill_pixel_span(span: &mut [u8], color: [u8; 4]) {
    if color[3] == 255 {
        for pixel in span.chunks_exact_mut(4) {
            write_pixel_channels(pixel, color);
        }
        return;
    }

    let alpha = color[3] as u32;
    let inverse = 255 - alpha;
    for pixel in span.chunks_exact_mut(4) {
        blend_pixel(pixel, color, alpha, inverse);
    }
}

#[inline]
fn write_pixel_channels(pixel: &mut [u8], color: [u8; 4]) {
    pixel[0] = color[0];
    pixel[1] = color[1];
    pixel[2] = color[2];
    pixel[3] = color[3];
}

#[inline]
fn blend_pixel(pixel: &mut [u8], color: [u8; 4], alpha: u32, inverse: u32) {
    for channel in 0..3 {
        let source = color[channel] as u32;
        let destination = pixel[channel] as u32;
        pixel[channel] = ((source * alpha + destination * inverse) / 255) as u8;
    }
    pixel[3] = 255;
}

#[inline]
fn write_pixel(frame: &mut HostRgbaFrame, x: u32, y: u32, color: [u8; 4]) {
    if color[3] == 0 {
        return;
    }
    let offset = ((y as usize * frame.width() as usize) + x as usize) * 4;
    let bytes = frame.as_bytes_mut();
    if color[3] == 255 {
        write_pixel_channels(&mut bytes[offset..offset + 4], color);
        return;
    }

    let alpha = color[3] as u32;
    let inverse = 255 - alpha;
    blend_pixel(&mut bytes[offset..offset + 4], color, alpha, inverse);
}

fn effective_clip(frame: &HostRgbaFrame, clip: Option<&FrameRect>) -> Option<Option<FrameRect>> {
    match (frame.paint_clip(), clip) {
        (Some(active), Some(clip)) => intersect(active, clip).map(Some),
        (Some(active), None) => Some(Some(active.clone())),
        (None, Some(clip)) => Some(Some(clip.clone())),
        (None, None) => Some(None),
    }
}

fn clamped_corner_radius(rect: &FrameRect, corner_radius: f32) -> f32 {
    if !corner_radius.is_finite() {
        return 0.0;
    }
    corner_radius
        .max(0.0)
        .min(rect.width.min(rect.height).max(0.0) * 0.5)
}

fn rounded_rect_contains_pixel(x: u32, y: u32, rect: &FrameRect, corner_radius: f32) -> bool {
    if !is_visible_frame(rect) {
        return false;
    }
    let px = x as f32 + 0.5;
    let py = y as f32 + 0.5;
    let left = rect.x;
    let top = rect.y;
    let right = rect.x + rect.width;
    let bottom = rect.y + rect.height;
    if px < left || px >= right || py < top || py >= bottom {
        return false;
    }
    let radius = clamped_corner_radius(rect, corner_radius);
    if radius <= 0.0 {
        return true;
    }
    let center_x = clamp_to_ordered_range(px, left + radius, right - radius);
    let center_y = clamp_to_ordered_range(py, top + radius, bottom - radius);
    let dx = px - center_x;
    let dy = py - center_y;
    dx * dx + dy * dy <= radius * radius
}

fn clamp_to_ordered_range(value: f32, min: f32, max: f32) -> f32 {
    if min <= max {
        value.clamp(min, max)
    } else {
        (min + max) * 0.5
    }
}

fn inset_frame(rect: &FrameRect, amount: f32) -> FrameRect {
    FrameRect {
        x: rect.x + amount,
        y: rect.y + amount,
        width: (rect.width - amount * 2.0).max(0.0),
        height: (rect.height - amount * 2.0).max(0.0),
    }
}

#[cfg(test)]
mod tests {
    use super::super::frame::HostRecordedPaintKind;
    use super::*;

    #[test]
    fn draw_rect_clipped_fills_only_clipped_span() {
        let mut frame = HostRgbaFrame::filled(4, 3, [0, 0, 0, 255]);
        draw_rect_clipped(
            &mut frame,
            FrameRect {
                x: 0.0,
                y: 0.0,
                width: 4.0,
                height: 3.0,
            },
            Some(&FrameRect {
                x: 1.0,
                y: 1.0,
                width: 2.0,
                height: 1.0,
            }),
            [10, 20, 30, 255],
        );

        assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
        assert_eq!(&frame.as_bytes()[20..24], &[10, 20, 30, 255]);
        assert_eq!(&frame.as_bytes()[24..28], &[10, 20, 30, 255]);
        assert_eq!(&frame.as_bytes()[28..32], &[0, 0, 0, 255]);
    }

    #[test]
    fn draw_rect_clipped_blends_alpha_over_existing_pixels() {
        let mut frame = HostRgbaFrame::filled(2, 1, [10, 20, 30, 255]);
        draw_rect_clipped(
            &mut frame,
            FrameRect {
                x: 0.0,
                y: 0.0,
                width: 2.0,
                height: 1.0,
            },
            None,
            [110, 120, 130, 128],
        );

        assert_eq!(&frame.as_bytes()[0..4], &[60, 70, 80, 255]);
        assert_eq!(&frame.as_bytes()[4..8], &[60, 70, 80, 255]);
    }

    #[test]
    fn rounded_rect_center_clamp_tolerates_crossed_float_bounds() {
        let clamped = clamp_to_ordered_range(40.0, 40.0, 39.999_992);
        assert!((clamped - 39.999_996).abs() <= f32::EPSILON);
    }

    #[test]
    fn draw_rect_clipped_skips_disjoint_active_and_explicit_clips() {
        let mut frame = HostRgbaFrame::filled(4, 4, [0, 0, 0, 255]);
        frame.replace_paint_clip(Some(FrameRect {
            x: 0.0,
            y: 0.0,
            width: 2.0,
            height: 2.0,
        }));

        draw_rect_clipped(
            &mut frame,
            FrameRect {
                x: 2.0,
                y: 2.0,
                width: 2.0,
                height: 2.0,
            },
            Some(&FrameRect {
                x: 2.0,
                y: 2.0,
                width: 2.0,
                height: 2.0,
            }),
            [10, 20, 30, 255],
        );

        assert!(frame
            .as_bytes()
            .chunks_exact(4)
            .all(|pixel| pixel == [0, 0, 0, 255]));
    }

    #[test]
    fn draw_rgba_image_clipped_copies_opaque_identity_rows_inside_clip() {
        let mut frame = HostRgbaFrame::filled(3, 2, [0, 0, 0, 255]);
        let rgba = vec![
            1, 2, 3, 255, 4, 5, 6, 255, 7, 8, 9, 255, 10, 11, 12, 255, 13, 14, 15, 255, 16, 17, 18,
            255,
        ];

        let drew = draw_rgba_image_clipped(
            &mut frame,
            FrameRect {
                x: 0.0,
                y: 0.0,
                width: 3.0,
                height: 2.0,
            },
            Some(&FrameRect {
                x: 1.0,
                y: 0.0,
                width: 2.0,
                height: 2.0,
            }),
            3,
            2,
            &rgba,
        );

        assert!(drew);
        assert_eq!(&frame.as_bytes()[0..4], &[0, 0, 0, 255]);
        assert_eq!(&frame.as_bytes()[4..8], &[4, 5, 6, 255]);
        assert_eq!(&frame.as_bytes()[8..12], &[7, 8, 9, 255]);
        assert_eq!(&frame.as_bytes()[12..16], &[0, 0, 0, 255]);
        assert_eq!(&frame.as_bytes()[16..20], &[13, 14, 15, 255]);
        assert_eq!(&frame.as_bytes()[20..24], &[16, 17, 18, 255]);
    }

    #[test]
    fn draw_rgba_image_clipped_blends_translucent_scaled_pixels() {
        let mut frame = HostRgbaFrame::filled(2, 1, [10, 20, 30, 255]);
        let rgba = vec![110, 120, 130, 128];

        let drew = draw_rgba_image_clipped(
            &mut frame,
            FrameRect {
                x: 0.0,
                y: 0.0,
                width: 2.0,
                height: 1.0,
            },
            None,
            1,
            1,
            &rgba,
        );

        assert!(drew);
        assert_eq!(&frame.as_bytes()[0..4], &[60, 70, 80, 255]);
        assert_eq!(&frame.as_bytes()[4..8], &[60, 70, 80, 255]);
    }

    #[test]
    fn draw_rgba_image_clipped_records_content_scoped_resource_keys() {
        let mut frame = HostRgbaFrame::recording_only(2, 1);
        let red = vec![255, 0, 0, 255];
        let blue = vec![0, 0, 255, 255];

        assert!(draw_rgba_image_clipped(
            &mut frame,
            FrameRect {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            None,
            1,
            1,
            &red,
        ));
        assert!(draw_rgba_image_clipped(
            &mut frame,
            FrameRect {
                x: 1.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            },
            None,
            1,
            1,
            &blue,
        ));

        let resource_keys = frame
            .into_recorded_commands()
            .into_iter()
            .filter_map(|command| match command.kind {
                HostRecordedPaintKind::Image { resource_key, .. } => Some(resource_key),
                _ => None,
            })
            .collect::<Vec<_>>();

        assert_eq!(resource_keys.len(), 2);
        assert_ne!(resource_keys[0], resource_keys[1]);
        assert!(resource_keys.iter().all(|key| key.starts_with("rgba:1x1:")));
    }

    #[test]
    fn draw_rgba_image_clipped_skips_disjoint_active_and_explicit_clips() {
        let mut frame = HostRgbaFrame::filled(2, 2, [0, 0, 0, 255]);
        frame.replace_paint_clip(Some(FrameRect {
            x: 0.0,
            y: 0.0,
            width: 1.0,
            height: 1.0,
        }));
        let rgba = vec![10, 20, 30, 255];

        let drew = draw_rgba_image_clipped(
            &mut frame,
            FrameRect {
                x: 1.0,
                y: 1.0,
                width: 1.0,
                height: 1.0,
            },
            Some(&FrameRect {
                x: 1.0,
                y: 1.0,
                width: 1.0,
                height: 1.0,
            }),
            1,
            1,
            &rgba,
        );

        assert!(!drew);
        assert!(frame
            .as_bytes()
            .chunks_exact(4)
            .all(|pixel| pixel == [0, 0, 0, 255]));
    }
}
