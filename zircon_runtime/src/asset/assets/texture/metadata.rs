use crate::core::framework::render::{
    RenderImageColorSpace, RenderImageDescriptor, RenderImageFallbackKind, RenderImageUsage,
    RenderSamplerDescriptor,
};

use super::{TextureAsset, TexturePayload};

pub fn render_image_descriptor(texture: &TextureAsset) -> RenderImageDescriptor {
    let (format, mip_count, array_layer_count) = match &texture.payload {
        TexturePayload::Rgba8 => ("rgba8unorm_srgb".to_string(), 1, 1),
        TexturePayload::Container {
            format,
            mip_count,
            array_layers,
            ..
        } => (format.clone(), (*mip_count).max(1), (*array_layers).max(1)),
    };

    RenderImageDescriptor {
        width: texture.width,
        height: texture.height,
        format,
        color_space: RenderImageColorSpace::Srgb,
        sampler: RenderSamplerDescriptor::default(),
        usage: vec![RenderImageUsage::Sampled, RenderImageUsage::CopyDst],
        mip_count,
        array_layer_count,
        fallback: RenderImageFallbackKind::MissingImage,
    }
}
