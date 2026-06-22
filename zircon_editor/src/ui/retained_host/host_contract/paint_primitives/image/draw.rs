use super::super::super::data::FrameRect;
use super::super::super::paint_frame::{HostPaintAtlasImage, HostRgbaFrame};

mod pipeline;

use pipeline::draw_rgba_image_clipped_with_recording;

use super::recording::ImageRecordingMetadata;

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
