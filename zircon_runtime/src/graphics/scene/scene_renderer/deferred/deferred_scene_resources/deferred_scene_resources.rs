pub(crate) struct DeferredSceneResources {
    pub(in crate::graphics::scene::scene_renderer::deferred) deferred_lighting_profile:
        crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile,
    pub(in crate::graphics::scene::scene_renderer::deferred) lighting_bind_group_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::deferred) lighting_pipelines:
        super::super::lighting_pipeline::DeferredLightingPipelineCache,
    pub(in crate::graphics::scene::scene_renderer::deferred) shadow_compare_sampler: wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::deferred) shadow_atlas_fallback_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::deferred) shadow_atlas_fallback_slot_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::deferred) shadow_atlas_fallback_globals_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::deferred) reflection_probe_bindings:
        crate::graphics::scene::scene_renderer::environment::ReflectionProbeGpuBindings,
    pub(in crate::graphics::scene::scene_renderer::deferred) lightmap_bindings:
        crate::graphics::scene::scene_renderer::environment::LightmapGpuBindings,
    pub(in crate::graphics::scene::scene_renderer::deferred) volumetric_apply:
        crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources,
}

impl DeferredSceneResources {
    pub(in crate::graphics::scene::scene_renderer) fn set_lightmap_bindings(
        &mut self,
        bindings: crate::graphics::scene::scene_renderer::environment::LightmapGpuBindings,
    ) {
        self.lightmap_bindings = bindings;
    }
}
