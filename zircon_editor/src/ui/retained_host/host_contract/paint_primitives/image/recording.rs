use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use super::super::super::data::FrameRect;
use super::super::super::paint_frame::{HostPaintAtlasImage, HostRgbaFrame};

#[derive(Clone, Copy)]
pub(in crate::ui::retained_host::host_contract) enum ImageRecordingMetadata<'a> {
    ResourceKey(Option<&'a str>),
    Atlas(&'a HostPaintAtlasImage),
}

impl ImageRecordingMetadata<'_> {
    pub(in crate::ui::retained_host::host_contract) fn is_valid(self) -> bool {
        match self {
            Self::ResourceKey(_) => true,
            Self::Atlas(atlas) => {
                atlas.width > 0
                    && atlas.height > 0
                    && atlas.rgba.as_ref().is_some_and(|rgba| {
                        rgba.len() == atlas.width as usize * atlas.height as usize * 4
                    })
            }
        }
    }

    pub(in crate::ui::retained_host::host_contract) fn record(
        self,
        frame: &mut HostRgbaFrame,
        rect: FrameRect,
        clip: Option<FrameRect>,
        image_width: u32,
        image_height: u32,
        rgba: &[u8],
    ) {
        match self {
            Self::ResourceKey(resource_key) => {
                let resource_key = resource_key
                    .map(str::to_string)
                    .unwrap_or_else(|| rgba_resource_key(image_width, image_height, rgba));
                frame.record_image(
                    rect,
                    clip,
                    resource_key,
                    image_width,
                    image_height,
                    Some(rgba.to_vec()),
                    None,
                );
            }
            Self::Atlas(atlas) => {
                frame.record_image(
                    rect,
                    clip,
                    atlas.resource_key.clone(),
                    atlas.width,
                    atlas.height,
                    atlas.rgba.clone(),
                    Some(atlas.clone()),
                );
            }
        }
    }
}

fn rgba_resource_key(image_width: u32, image_height: u32, rgba: &[u8]) -> String {
    let mut hasher = DefaultHasher::new();
    image_width.hash(&mut hasher);
    image_height.hash(&mut hasher);
    rgba.hash(&mut hasher);
    format!("rgba:{image_width}x{image_height}:{:016x}", hasher.finish())
}
