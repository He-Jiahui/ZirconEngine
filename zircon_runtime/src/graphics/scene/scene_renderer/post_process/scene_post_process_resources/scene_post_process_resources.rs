use super::super::resources::depth_sampling_mode::PostProcessDepthSamplingMode;

pub(crate) struct ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_sampling_mode:
        PostProcessDepthSamplingMode,
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) ssao_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_of_field_prepare_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_camera_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_tile_max_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_neighbor_max_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) ssao_pipeline:
        std::sync::OnceLock<wgpu::ComputePipeline>,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_of_field_prepare_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_camera_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_tile_max_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_neighbor_max_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_depth_pyramid_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_depth_pyramid_coarse_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_reflection_pyramid_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_reflection_pyramid_coarse_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_resolve_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) screen_space_reflection_specular_occlusion_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) ssao_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) depth_of_field_prepare_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) motion_vector_camera_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) light_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_probe_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_trace_region_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) reflection_probe_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) black_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) white_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) hzb_source_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_lut_texture_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_lut_texture_3d_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) effect_lut_sampler: wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::post_process) scene_depth_sampler: wgpu::Sampler,
}

impl ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer) fn black_texture_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.black_texture_view
    }

    pub(in crate::graphics::scene::scene_renderer) fn white_texture_view(
        &self,
    ) -> &wgpu::TextureView {
        &self.white_texture_view
    }
}
