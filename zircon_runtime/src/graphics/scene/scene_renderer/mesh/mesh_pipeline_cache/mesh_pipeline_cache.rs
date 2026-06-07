use std::collections::HashMap;

use crate::graphics::scene::resources::PipelineKey;

pub(crate) struct MeshPipelineCache {
    pub(in crate::graphics::scene::scene_renderer::mesh) target_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_pipeline_layout: wgpu::PipelineLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_uniform_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_disabled_uniform_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_compare_sampler:
        wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::mesh) fallback_shadow_map_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::mesh) shader_modules:
        HashMap<String, wgpu::ShaderModule>,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_pipelines:
        HashMap<PipelineKey, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) motion_vector_mesh_pipelines:
        HashMap<PipelineKey, wgpu::RenderPipeline>,
}
