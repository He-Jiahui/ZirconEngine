use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::mesh::{
    BuiltMeshDraws, CachedMeshDrawCommands, PendingMeshCommandCacheExtractionContext,
    build_mesh_draws,
};
use crate::graphics::scene::scene_renderer::shadow::ShadowLightSlotAssignments;
use crate::graphics::types::ViewportRenderFrame;

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn build_mesh_draws(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        material_texture_layout: &wgpu::BindGroupLayout,
        gpu_scene: &mut GpuScene,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        virtual_geometry_enabled: bool,
        shadow_light_slots: Option<&ShadowLightSlotAssignments>,
    ) -> BuiltMeshDraws {
        let virtual_geometry_enabled = virtual_geometry_enabled && self.virtual_geometry_enabled();
        let volumetric_fog_enabled = self.volumetric_fog_enabled();
        build_mesh_draws(
            device,
            queue,
            encoder,
            material_texture_layout,
            gpu_scene,
            streamer,
            frame,
            virtual_geometry_enabled,
            volumetric_fog_enabled,
            shadow_light_slots,
            None,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn build_mesh_draws_with_command_cache(
        &self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        material_texture_layout: &wgpu::BindGroupLayout,
        gpu_scene: &mut GpuScene,
        streamer: &ResourceStreamer,
        frame: &ViewportRenderFrame,
        virtual_geometry_enabled: bool,
        shadow_light_slots: Option<&ShadowLightSlotAssignments>,
        command_cache: &mut CachedMeshDrawCommands,
        mesh_pipelines: &mut MeshPipelineCache,
        generation: u64,
        shader_quality: crate::core::framework::render::ShaderQualityTier,
    ) -> BuiltMeshDraws {
        let virtual_geometry_enabled = virtual_geometry_enabled && self.virtual_geometry_enabled();
        let volumetric_fog_enabled = self.volumetric_fog_enabled();
        build_mesh_draws(
            device,
            queue,
            encoder,
            material_texture_layout,
            gpu_scene,
            streamer,
            frame,
            virtual_geometry_enabled,
            volumetric_fog_enabled,
            shadow_light_slots,
            Some(PendingMeshCommandCacheExtractionContext::new(
                command_cache,
                mesh_pipelines,
                generation,
                shader_quality,
            )),
        )
    }
}
