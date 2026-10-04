use super::scene_renderer_advanced_plugin_resources::SceneRendererAdvancedPluginResources;
use crate::core::framework::render::RenderViewportPickPolicy;
use crate::graphics::backend::RenderBackend;
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::ResourceStreamer;
use crate::graphics::scene::scene_renderer::mesh::{
    build_mesh_draws, BuiltMeshDraws, CachedMeshDrawCommands, MeshHitProxyTokenSource,
    PendingMeshCommandCacheExtractionContext,
};
use crate::graphics::scene::scene_renderer::mesh::{MaterialPipelineFeatureSet, MeshPipelineCache};
use crate::graphics::scene::scene_renderer::shadow::ShadowLightSlotAssignments;
use crate::graphics::types::{GraphicsError, ViewportRenderFrame};

impl SceneRendererAdvancedPluginResources {
    pub(in crate::graphics::scene::scene_renderer::core) fn build_mesh_draws(
        &self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        material_texture_layout: &wgpu::BindGroupLayout,
        gpu_scene: &mut GpuScene,
        streamer: &mut ResourceStreamer,
        mesh_pipelines: &mut MeshPipelineCache,
        frame: &ViewportRenderFrame,
        virtual_geometry_enabled: bool,
        uses_direct_lights: bool,
        shadow_light_slots: Option<&ShadowLightSlotAssignments>,
    ) -> Result<BuiltMeshDraws, GraphicsError> {
        let virtual_geometry_enabled = virtual_geometry_enabled && self.virtual_geometry_enabled();
        let volumetric_fog_enabled = self.volumetric_fog_enabled();
        let direct_lighting_preparation =
            uses_direct_lights.then_some(frame.preview().lighting_enabled);
        let material_pipeline_features =
            MaterialPipelineFeatureSet::direct(shadow_light_slots.is_some());
        build_mesh_draws(
            backend,
            encoder,
            material_texture_layout,
            gpu_scene,
            streamer,
            mesh_pipelines,
            frame,
            virtual_geometry_enabled,
            volumetric_fog_enabled,
            material_pipeline_features,
            direct_lighting_preparation,
            shadow_light_slots,
            None,
            None,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn build_hit_proxy_mesh_draws(
        &self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        material_texture_layout: &wgpu::BindGroupLayout,
        gpu_scene: &mut GpuScene,
        streamer: &mut ResourceStreamer,
        mesh_pipelines: &mut MeshPipelineCache,
        frame: &ViewportRenderFrame,
        virtual_geometry_enabled: bool,
        policy: RenderViewportPickPolicy,
        hit_proxy_tokens: &dyn MeshHitProxyTokenSource,
    ) -> Result<BuiltMeshDraws, GraphicsError> {
        let virtual_geometry_enabled = virtual_geometry_enabled && self.virtual_geometry_enabled();
        build_mesh_draws(
            backend,
            encoder,
            material_texture_layout,
            gpu_scene,
            streamer,
            mesh_pipelines,
            frame,
            virtual_geometry_enabled,
            false,
            MaterialPipelineFeatureSet::hit_proxy(policy),
            None,
            None,
            None,
            Some(hit_proxy_tokens),
        )
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn build_environment_capture_mesh_draws(
        &self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        material_texture_layout: &wgpu::BindGroupLayout,
        gpu_scene: &mut GpuScene,
        streamer: &mut ResourceStreamer,
        mesh_pipelines: &mut MeshPipelineCache,
        frame: &ViewportRenderFrame,
    ) -> Result<BuiltMeshDraws, GraphicsError> {
        build_mesh_draws(
            backend,
            encoder,
            material_texture_layout,
            gpu_scene,
            streamer,
            mesh_pipelines,
            frame,
            false,
            false,
            MaterialPipelineFeatureSet::environment_capture(),
            None,
            None,
            None,
            None,
        )
    }

    pub(in crate::graphics::scene::scene_renderer::core) fn build_mesh_draws_with_command_cache(
        &self,
        backend: &RenderBackend,
        encoder: &mut wgpu::CommandEncoder,
        material_texture_layout: &wgpu::BindGroupLayout,
        gpu_scene: &mut GpuScene,
        streamer: &mut ResourceStreamer,
        frame: &ViewportRenderFrame,
        virtual_geometry_enabled: bool,
        material_pipeline_features: MaterialPipelineFeatureSet,
        shadow_light_slots: Option<&ShadowLightSlotAssignments>,
        command_cache: &mut CachedMeshDrawCommands,
        mesh_pipelines: &mut MeshPipelineCache,
        generation: u64,
        shader_quality: crate::core::framework::render::ShaderQualityTier,
    ) -> Result<BuiltMeshDraws, GraphicsError> {
        let virtual_geometry_enabled = virtual_geometry_enabled && self.virtual_geometry_enabled();
        let volumetric_fog_enabled = self.volumetric_fog_enabled();
        build_mesh_draws(
            backend,
            encoder,
            material_texture_layout,
            gpu_scene,
            streamer,
            mesh_pipelines,
            frame,
            virtual_geometry_enabled,
            volumetric_fog_enabled,
            material_pipeline_features,
            Some(frame.preview().lighting_enabled),
            shadow_light_slots,
            Some(PendingMeshCommandCacheExtractionContext::new(
                command_cache,
                generation,
                shader_quality,
            )),
            None,
        )
    }
}

#[cfg(test)]
#[path = "tests/build_mesh_draws.rs"]
mod tests;
