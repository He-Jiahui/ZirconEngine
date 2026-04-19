pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) struct BufferBundle {
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) bloom_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) ssao_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) cluster_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) post_process_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) light_buffer: wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) hybrid_gi_probe_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) hybrid_gi_trace_region_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) reflection_probe_buffer:
        wgpu::Buffer,
}
