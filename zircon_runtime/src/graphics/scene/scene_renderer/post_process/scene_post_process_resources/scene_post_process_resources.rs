pub(crate) struct ScenePostProcessResources {
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) ssao_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_pipeline: wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) ssao_pipeline: wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_pipeline: wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_pipeline: wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process) bloom_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) ssao_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) cluster_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) post_process_params_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) light_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_probe_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) hybrid_gi_trace_region_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) reflection_probe_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process) black_texture_view: wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::post_process) white_texture_view: wgpu::TextureView,
}
