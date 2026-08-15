use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::MeshPipelineCache;
use crate::graphics::scene::scene_renderer::mesh::{
    build_mesh_draws, BuiltMeshDraws, CachedMeshDrawCommands,
    PendingMeshCommandCacheExtractionContext,
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
        uses_direct_lights: bool,
        shadow_light_slots: Option<&ShadowLightSlotAssignments>,
    ) -> BuiltMeshDraws {
        let virtual_geometry_enabled = virtual_geometry_enabled && self.virtual_geometry_enabled();
        let volumetric_fog_enabled = self.volumetric_fog_enabled();
        let direct_lighting_preparation =
            uses_direct_lights.then_some(frame.preview().lighting_enabled);
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
            direct_lighting_preparation,
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
            Some(frame.preview().lighting_enabled),
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

#[cfg(test)]
mod tests {
    #[test]
    fn direct_mesh_preparation_combines_profile_and_preview_light_policies() {
        let source = include_str!("build_mesh_draws.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("mesh preparation source should retain a test-module boundary");
        let direct_method = production
            .split("fn build_mesh_draws(")
            .nth(1)
            .and_then(|source| {
                source
                    .split("fn build_mesh_draws_with_command_cache(")
                    .next()
            })
            .expect("direct mesh preparation method");

        assert!(direct_method.contains("uses_direct_lights: bool,"));
        assert!(direct_method
            .contains("uses_direct_lights.then_some(frame.preview().lighting_enabled)"));
        assert!(direct_method
            .contains("volumetric_fog_enabled,\n            direct_lighting_preparation,"));
    }

    #[test]
    fn compiled_mesh_preparation_retains_its_preview_light_policy() {
        let source = include_str!("build_mesh_draws.rs");
        let production = source
            .split_once("#[cfg(test)]")
            .map(|(production, _)| production)
            .expect("mesh preparation source should retain a test-module boundary");
        let compiled_method = production
            .split("fn build_mesh_draws_with_command_cache(")
            .nth(1)
            .expect("compiled mesh preparation method");

        assert!(compiled_method.contains(
            "volumetric_fog_enabled,\n            Some(frame.preview().lighting_enabled),"
        ));
    }
}
