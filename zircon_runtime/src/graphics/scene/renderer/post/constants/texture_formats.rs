//! 将框架后处理格式映射为 wgpu 纹理格式，并集中声明中间资源的 GPU 格式契约。
//! 框架格式经统一映射供分配与管线使用；SSR 和 SMAA 的内部中间格式另由这里固定。
use crate::core::framework::render::{
    RenderPostProcessTextureFormat, COLOR_LUT_FORMAT, INTERMEDIATE_HDR_FORMAT_DEFAULT,
    TONEMAPPED_SDR_FORMAT,
};

pub(crate) const POST_PROCESS_COLOR_LUT_FORMAT: wgpu::TextureFormat =
    wgpu_post_process_texture_format(COLOR_LUT_FORMAT);
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
pub(crate) const SMAA_STAGE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8Unorm;

/// 将渲染框架的后处理格式转换为 wgpu 设备格式，供资源创建和管线描述共用。
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
