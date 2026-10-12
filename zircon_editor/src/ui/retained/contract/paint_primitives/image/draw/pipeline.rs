use super::super::super::super::data::FrameRect;
use super::super::super::super::paint_frame::HostRgbaFrame;
use super::super::super::super::paint_geometry::PixelRect;
use super::super::super::clip::effective_clip;
use super::super::raster::{draw_scaled_rgba_image_pixels, try_copy_opaque_identity_image_rows};
use super::super::recording::ImageRecordingMetadata;

pub(super) fn draw_rgba_image_clipped_with_recording(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
    recording: ImageRecordingMetadata<'_>,
) -> bool {
    if image_width == 0
        || image_height == 0
        || rgba.len() != image_width as usize * image_height as usize * 4
        || !recording.is_valid()
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
        recording.record(
            frame,
            rect.clone(),
            effective_clip.clone(),
            image_width,
            image_height,
            rgba,
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
