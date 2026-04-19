pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) struct PipelineBundle {
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) bloom_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) ssao_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) cluster_pipeline:
        wgpu::ComputePipeline,
    pub(in crate::graphics::scene::scene_renderer::post_process::resources::new) post_process_pipeline:
        wgpu::RenderPipeline,
}
