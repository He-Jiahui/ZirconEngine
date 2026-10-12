use super::super::constants::DEPTH_FORMAT;
use super::super::target_extent::texture_extent;

// TODO: [CR-GRAPHICS-SCENECORE-0003] 确认此场景侧 depth 构造器是否仍有独立用途；当前检索仅见定义与重导出，实际 OffscreenTarget 使用 backend 自己的构造函数。
/// 为场景深度 pass 和后处理深度读取共用的深度纹理；尺寸使用有效物理 extent。
pub(crate) fn create_depth_texture(
    device: &wgpu::Device,
    size: crate::core::math::UVec2,
) -> wgpu::Texture {
    device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-depth"),
        size: texture_extent(size),
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: DEPTH_FORMAT,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    })
}
