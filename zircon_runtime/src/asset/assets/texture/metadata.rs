use crate::core::framework::render::RenderImageDescriptor;

use super::{TextureAsset, TextureAssetDescriptor};

pub fn texture_asset_descriptor(texture: &TextureAsset) -> TextureAssetDescriptor {
    texture
        .descriptor
        .clone()
        .unwrap_or_else(|| TextureAssetDescriptor::from_payload(&texture.payload))
        .normalized()
}

pub fn render_image_descriptor(texture: &TextureAsset) -> RenderImageDescriptor {
    texture_asset_descriptor(texture).to_render_image_descriptor(texture.width, texture.height)
}
