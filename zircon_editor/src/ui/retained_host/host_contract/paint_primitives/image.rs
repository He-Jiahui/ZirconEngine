use super::super::data::FrameRect;
use super::super::paint_frame::{HostPaintAtlasImage, HostRgbaFrame};
use super::super::paint_geometry::PixelRect;

use super::clip::effective_clip;

mod raster;
mod recording;

use raster::{draw_scaled_rgba_image_pixels, try_copy_opaque_identity_image_rows};
use recording::ImageRecordingMetadata;

pub(in crate::ui::retained_host::host_contract) fn draw_rgba_image_clipped(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
) -> bool {
    draw_rgba_image_clipped_with_recording(
        frame,
        rect,
        clip,
        image_width,
        image_height,
        rgba,
        ImageRecordingMetadata::ResourceKey(None),
    )
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
    draw_rgba_image_clipped_with_recording(
        frame,
        rect,
        clip,
        image_width,
        image_height,
        rgba,
        ImageRecordingMetadata::ResourceKey(Some(resource_key)),
    )
}

pub(in crate::ui::retained_host::host_contract) fn draw_rgba_image_clipped_with_atlas(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    image_width: u32,
    image_height: u32,
    rgba: &[u8],
    atlas: &HostPaintAtlasImage,
) -> bool {
    if atlas.rgba.is_none() {
        return false;
    }
    draw_rgba_image_clipped_with_recording(
        frame,
        rect,
        clip,
        image_width,
        image_height,
        rgba,
        ImageRecordingMetadata::Atlas(atlas),
    )
}

fn draw_rgba_image_clipped_with_recording(
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

#[cfg(test)]
mod tests;
