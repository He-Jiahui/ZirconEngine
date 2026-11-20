//! 离屏颜色产品与内部场景附件共用容器，但分别按输出与场景尺寸配置。
use crate::core::framework::render::RenderViewportHandle;
use crate::core::math::UVec2;
use crate::graphics::resource_identity::SampledTextureIdentity;
use crate::graphics::types::ViewportRenderOutputTarget;

pub(crate) struct OffscreenTarget {
    pub(crate) size: UVec2,
    pub(crate) render_size: UVec2,
    pub(crate) final_color: wgpu::Texture,
    pub(crate) final_color_view: wgpu::TextureView,
    pub(crate) final_color_viewport: Option<RenderViewportHandle>,
    pub(crate) final_color_output: Option<ViewportRenderOutputTarget>,
    pub(crate) global_illumination: wgpu::Texture,
    pub(crate) global_illumination_view: wgpu::TextureView,
    pub(crate) scene_color: wgpu::Texture,
    pub(crate) scene_color_view: wgpu::TextureView,
    pub(crate) scene_color_identity: SampledTextureIdentity,
    pub(crate) scene_velocity: Option<OffscreenTargetSceneVelocity>,
    pub(crate) bloom: wgpu::Texture,
    pub(crate) bloom_view: wgpu::TextureView,
    pub(crate) gbuffer_albedo: wgpu::Texture,
    pub(crate) gbuffer_albedo_view: wgpu::TextureView,
    pub(crate) gbuffer_emissive: wgpu::Texture,
    pub(crate) gbuffer_emissive_view: wgpu::TextureView,
    pub(crate) gbuffer_material: wgpu::Texture,
    pub(crate) gbuffer_material_view: wgpu::TextureView,
    pub(crate) normal: wgpu::Texture,
    pub(crate) normal_view: wgpu::TextureView,
    pub(crate) ambient_occlusion: wgpu::Texture,
    pub(crate) ambient_occlusion_view: wgpu::TextureView,
    pub(crate) depth: wgpu::Texture,
    pub(crate) depth_view: wgpu::TextureView,
    pub(crate) depth_identity: SampledTextureIdentity,
    pub(crate) cluster_dimensions: UVec2,
    pub(crate) cluster_buffer: wgpu::Buffer,
    pub(crate) cluster_buffer_bytes: usize,
}

impl OffscreenTarget {
    pub(crate) const RETAINED_FRAME_TEXTURE_COUNT: usize = 10;

    pub(crate) fn retained_frame_texture_count(&self) -> usize {
        let retained_textures = [
            &self.final_color,
            &self.global_illumination,
            &self.scene_color,
            &self.bloom,
            &self.gbuffer_albedo,
            &self.gbuffer_emissive,
            &self.gbuffer_material,
            &self.normal,
            &self.ambient_occlusion,
            &self.depth,
        ];
        retained_textures.len() + usize::from(self.scene_velocity.is_some())
    }
}

pub(crate) struct OffscreenTargetSceneVelocity {
    pub(crate) texture: wgpu::Texture,
    pub(crate) view: wgpu::TextureView,
    pub(crate) identity: SampledTextureIdentity,
}
