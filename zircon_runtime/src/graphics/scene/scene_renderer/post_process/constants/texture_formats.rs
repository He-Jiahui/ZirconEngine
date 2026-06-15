use crate::core::framework::render::{
    RenderPostProcessTextureFormat, INTERMEDIATE_HDR_FORMAT_DEFAULT, TONEMAPPED_SDR_FORMAT,
};

pub(crate) const POST_PROCESS_INTERMEDIATE_HDR_FORMAT: wgpu::TextureFormat =
    wgpu_post_process_texture_format(INTERMEDIATE_HDR_FORMAT_DEFAULT);
pub(crate) const POST_PROCESS_TONEMAPPED_FORMAT: wgpu::TextureFormat =
    wgpu_post_process_texture_format(TONEMAPPED_SDR_FORMAT);
pub(crate) const SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_FORMAT: wgpu::TextureFormat =
    POST_PROCESS_INTERMEDIATE_HDR_FORMAT;
pub(crate) const SCREEN_SPACE_REFLECTION_REFLECTION_PYRAMID_COARSE_FORMAT: wgpu::TextureFormat =
    POST_PROCESS_INTERMEDIATE_HDR_FORMAT;
pub(crate) const SCREEN_SPACE_REFLECTION_SPECULAR_OCCLUSION_FORMAT: wgpu::TextureFormat =
    wgpu::TextureFormat::Rgba8Unorm;

pub(crate) const fn wgpu_post_process_texture_format(
    format: RenderPostProcessTextureFormat,
) -> wgpu::TextureFormat {
    match format {
        RenderPostProcessTextureFormat::R8Unorm => wgpu::TextureFormat::R8Unorm,
        RenderPostProcessTextureFormat::Rg16Float => wgpu::TextureFormat::Rg16Float,
        RenderPostProcessTextureFormat::Rgba8Unorm => wgpu::TextureFormat::Rgba8Unorm,
        RenderPostProcessTextureFormat::Rgba8UnormSrgb => wgpu::TextureFormat::Rgba8UnormSrgb,
        RenderPostProcessTextureFormat::Rg11b10Ufloat => wgpu::TextureFormat::Rg11b10Ufloat,
        RenderPostProcessTextureFormat::Rgba16Float => wgpu::TextureFormat::Rgba16Float,
        RenderPostProcessTextureFormat::Rgba32Float => wgpu::TextureFormat::Rgba32Float,
    }
}
