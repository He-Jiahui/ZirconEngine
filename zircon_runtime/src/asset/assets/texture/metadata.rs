//! 纹理描述符把作者纹理元数据转换为渲染图像契约；GPU 上传需要以此与实际 payload 的格式和尺寸保持一致。

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
    texture
        .descriptor
        .clone()
        .unwrap_or_else(|| TextureAssetDescriptor::from_payload(&texture.payload))
        .into_render_image_descriptor(texture.width, texture.height)
}
