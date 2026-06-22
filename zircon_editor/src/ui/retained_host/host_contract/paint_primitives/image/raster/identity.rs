use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;

pub(in crate::ui::retained_host::host_contract) fn try_copy_opaque_identity_image_rows(
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
