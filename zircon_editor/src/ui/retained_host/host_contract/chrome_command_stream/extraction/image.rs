use std::sync::Arc;

use super::super::command::{ChromeImagePayload, ChromeImageUvRect};
use crate::ui::retained_host::host_contract::paint_frame::HostPaintAtlasImage;

pub(super) fn chrome_image_payload_from_recorded_image(
    resource_key: String,
    width: u32,
    height: u32,
    rgba: Option<Arc<[u8]>>,
    atlas: Option<HostPaintAtlasImage>,
    include_image_bytes: bool,
) -> ChromeImagePayload {
    if let Some(atlas) = atlas {
        return chrome_atlas_image_payload(atlas, include_image_bytes);
    }

    let upload_bytes = rgba
        .as_ref()
        .map(|rgba| rgba.len() as u64)
        .unwrap_or_else(|| u64::from(width) * u64::from(height) * 4);
    ChromeImagePayload {
        resource_key,
        resource_generation: 0,
        width,
        height,
        upload_bytes,
        rgba: include_image_bytes.then_some(rgba).flatten(),
        atlas_uv: None,
    }
}

fn chrome_atlas_image_payload(
    atlas: HostPaintAtlasImage,
    include_image_bytes: bool,
) -> ChromeImagePayload {
    let atlas_rgba: Option<Arc<[u8]>> = include_image_bytes
        .then_some(atlas.rgba)
        .flatten()
        .map(Arc::<[u8]>::from);
    let upload_bytes = atlas_rgba
        .as_ref()
        .map(|rgba| rgba.len() as u64)
        .unwrap_or_else(|| u64::from(atlas.width) * u64::from(atlas.height) * 4);
    ChromeImagePayload {
        resource_key: atlas.resource_key,
        resource_generation: atlas.resource_generation,
        width: atlas.width,
        height: atlas.height,
        upload_bytes,
        rgba: atlas_rgba,
        atlas_uv: Some(ChromeImageUvRect {
            min: atlas.uv.min,
            max: atlas.uv.max,
        }),
    }
}
