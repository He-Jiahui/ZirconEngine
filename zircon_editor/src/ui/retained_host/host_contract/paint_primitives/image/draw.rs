use std::sync::Arc;

use super::super::super::data::FrameRect;
use super::super::super::paint_frame::{HostPaintAtlasImage, HostRgbaFrame};
use super::super::clip::effective_clip;

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

pub(in crate::ui::retained_host::host_contract) fn draw_shared_rgba_image_clipped_with_resource_key(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    resource_key: &str,
    image_width: u32,
    image_height: u32,
    rgba: &Arc<[u8]>,
) -> bool {
    draw_rgba_image_clipped_with_recording(
        frame,
        rect,
        clip,
        image_width,
        image_height,
        rgba.as_ref(),
        ImageRecordingMetadata::SharedResourceKey(Some(resource_key), rgba),
    )
}

pub(in crate::ui::retained_host::host_contract) fn draw_gpu_image_clipped_with_resource_key(
    frame: &mut HostRgbaFrame,
    rect: FrameRect,
    clip: Option<&FrameRect>,
    resource_key: &str,
    image_width: u32,
    image_height: u32,
) -> bool {
    if resource_key.is_empty() || image_width == 0 || image_height == 0 {
        return false;
    }
    let Some(effective_clip) = effective_clip(frame, clip) else {
        return false;
    };
    if !frame.is_recording() {
        return false;
    }
    frame.record_image(
        rect,
        effective_clip,
        resource_key,
        image_width,
        image_height,
        None,
        None,
    );
    true
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
