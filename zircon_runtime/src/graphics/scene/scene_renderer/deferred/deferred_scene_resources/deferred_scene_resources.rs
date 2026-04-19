pub(crate) struct DeferredSceneResources {
    pub(in crate::graphics::scene::scene_renderer::deferred) geometry_pipeline: wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::deferred) lighting_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::deferred) lighting_pipeline: wgpu::RenderPipeline,
}
