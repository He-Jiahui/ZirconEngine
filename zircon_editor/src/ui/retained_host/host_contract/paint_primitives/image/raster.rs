use super::super::super::data::FrameRect;
use super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::paint_geometry::PixelRect;

pub(super) fn draw_scaled_rgba_image_pixels(
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

pub(super) fn try_copy_opaque_identity_image_rows(
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
