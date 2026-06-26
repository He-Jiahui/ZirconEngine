pub(crate) struct DeferredSceneResources {
    pub(in crate::graphics::scene::scene_renderer::deferred) lighting_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::deferred) lighting_pipeline:
        wgpu::RenderPipeline,
    pub(in crate::graphics::scene::scene_renderer::deferred) shadow_compare_sampler: wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::deferred) shadow_atlas_fallback_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::deferred) shadow_atlas_fallback_slot_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::deferred) shadow_atlas_fallback_globals_buffer:
        wgpu::Buffer,
}
