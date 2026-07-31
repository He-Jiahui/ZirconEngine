use std::collections::HashMap;

use crate::core::framework::render::{
    GEOMETRY_SOURCE_ID_STATIC_MESH, GeometrySourceDescriptor, GeometrySourceId, ShaderQualityTier,
    ShaderVariantKey, ShaderVariantMissReport,
};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::environment::{
    SceneLightmapResources, SceneReflectionProbeResources,
};
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};
use crate::graphics::shader::ShaderVariantCacheDisk;

use super::{MeshPipelineVariantRegistry, MeshPipelineVariantResolver};

pub(crate) struct MeshPipelineCache {
    pub(in crate::graphics::scene::scene_renderer::mesh) target_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_pipeline_layout: wgpu::PipelineLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_fragment_store_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_mesh_pipeline_layout:
        wgpu::PipelineLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_compare_sampler:
        wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_light_grid_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_light_grid_empty_zbins_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_light_grid_empty_tile_masks_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_atlas_fallback_slot_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_atlas_fallback_globals_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) fallback_shadow_atlas_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_volumetric_apply:
        crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_volumetric_disabled_params_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) transmission_scene_color:
        crate::graphics::scene::scene_renderer::advanced_lighting::transmission::TransmissionSceneColorFallbackResources,
    pub(in crate::graphics::scene::scene_renderer) light_cookies:
        crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::LightCookieAtlasResources,
    pub(in crate::graphics::scene::scene_renderer) irradiance_volume:
        crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::IrradianceVolumeResources,
    pub(in crate::graphics::scene::scene_renderer) reflection_probes: SceneReflectionProbeResources,
    pub(in crate::graphics::scene::scene_renderer) lightmaps: SceneLightmapResources,
    pub(in crate::graphics::scene::scene_renderer::mesh) shader_modules:
        HashMap<String, wgpu::ShaderModule>,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_variant_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) oit_mesh_variant_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) gbuffer_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) depth_prepass_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) velocity_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) shadow_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) taa_reactive_mask_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) taa_reactive_material_mask_mesh_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) pipeline_variant_registry:
        MeshPipelineVariantRegistry,
    pub(in crate::graphics::scene::scene_renderer::mesh) geometry_source_descriptors:
        HashMap<GeometrySourceId, GeometrySourceDescriptor>,
    pub(in crate::graphics::scene::scene_renderer::mesh) shader_variant_disk_cache:
        ShaderVariantCacheDisk,
}

impl MeshPipelineCache {
    pub(crate) fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        self.resolve_variant_for_geometry(
            kind,
            pipeline_key,
            GEOMETRY_SOURCE_ID_STATIC_MESH,
            shader_quality,
        )
    }

    pub(crate) fn resolve_variant_for_geometry(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        self.pipeline_variant_registry.resolve_variant_for_geometry(
            kind,
            pipeline_key,
            geometry_source,
            shader_quality,
        )
    }

    pub(crate) fn pipeline_and_shader_key_for_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> Option<(MeshPassPipelineKind, PipelineKey, ShaderVariantKey)> {
        let key = self.pipeline_variant_registry.key_for_variant(variant_id)?;
        Some((
            key.kind(),
            key.pipeline_key().clone(),
            key.shader_variant_key().clone(),
        ))
    }

    pub(crate) fn register_geometry_source_descriptor(
        &mut self,
        descriptor: GeometrySourceDescriptor,
    ) {
        self.geometry_source_descriptors
            .insert(descriptor.id, descriptor);
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn geometry_source_descriptor(
        &self,
        geometry_source: GeometrySourceId,
    ) -> Option<GeometrySourceDescriptor> {
        self.geometry_source_descriptors
            .get(&geometry_source)
            .cloned()
    }

    pub(in crate::graphics::scene::scene_renderer::mesh) fn geometry_source_descriptor_for_variant(
        &mut self,
        key: &ShaderVariantKey,
    ) -> Option<GeometrySourceDescriptor> {
        match self.geometry_source_descriptor(key.geometry_source) {
            Some(descriptor) => Some(descriptor),
            None => {
                self.record_shader_variant_disk_error(key);
                None
            }
        }
    }

    pub(crate) fn reset_shader_variant_miss_report(&mut self) {
        self.pipeline_variant_registry.reset_miss_report();
    }

    pub(crate) fn shader_variant_miss_report(&self) -> ShaderVariantMissReport {
        self.pipeline_variant_registry.miss_report()
    }

    pub(crate) fn record_shader_variant_disk_hit(&mut self, key: &ShaderVariantKey) {
        self.pipeline_variant_registry.record_disk_hit(key);
    }

    pub(crate) fn record_shader_variant_disk_write(&mut self, key: &ShaderVariantKey) {
        self.pipeline_variant_registry.record_disk_write(key);
    }

    pub(crate) fn record_shader_variant_disk_error(&mut self, key: &ShaderVariantKey) {
        self.pipeline_variant_registry.record_disk_error(key);
    }

    pub(crate) fn record_shader_variant_compile_miss(&mut self, key: &ShaderVariantKey) {
        self.pipeline_variant_registry.record_compile_miss(key);
    }

    #[cfg(test)]
    pub(crate) fn replace_shader_variant_disk_cache_for_tests(
        &mut self,
        cache: ShaderVariantCacheDisk,
    ) {
        self.shader_variant_disk_cache = cache;
    }
}

impl MeshPipelineVariantResolver for MeshPipelineCache {
    fn resolve_variant_for_geometry(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        geometry_source: GeometrySourceId,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        MeshPipelineCache::resolve_variant_for_geometry(
            self,
            kind,
            pipeline_key,
            geometry_source,
            shader_quality,
        )
    }
}

#[cfg(test)]
mod tests {
    fn assert_cache_hit_precedes_variant_projection(
        source: &str,
        function_name: &str,
        cache_lookup: &str,
    ) {
        let function = source
            .split_once(function_name)
            .map(|(_, function)| function)
            .expect("ensure function must exist");
        let cache_lookup = function
            .find(cache_lookup)
            .expect("ensure function must check its pipeline cache");
        let variant_projection = function
            .find("pipeline_and_shader_key_for_variant")
            .expect("ensure function must project its variant on a cache miss");

        assert!(
            cache_lookup < variant_projection,
            "pipeline cache hit must return before variant cloning and shader source assembly"
        );
    }

    #[test]
    fn mesh_pipeline_cache_hits_precede_variant_and_shader_projection() {
        let cases = [
            (
                include_str!("ensure_pipeline.rs"),
                "ensure_pipeline_for_variant",
                "mesh_variant_pipelines",
            ),
            (
                include_str!("ensure_oit_pipeline.rs"),
                "ensure_oit_pipeline_for_base_variant",
                "oit_mesh_variant_pipelines",
            ),
            (
                include_str!("ensure_gbuffer_pipeline.rs"),
                "ensure_gbuffer_pipeline_for_variant",
                "gbuffer_mesh_pipelines",
            ),
            (
                include_str!("ensure_depth_prepass_pipeline.rs"),
                "ensure_depth_prepass_pipeline_for_variant",
                "depth_prepass_mesh_pipelines",
            ),
            (
                include_str!("ensure_shadow_pipeline.rs"),
                "ensure_shadow_pipeline_for_variant",
                "shadow_mesh_pipelines",
            ),
            (
                include_str!("ensure_velocity_pipeline.rs"),
                "ensure_velocity_pipeline_for_variant",
                "velocity_mesh_pipelines",
            ),
            (
                include_str!("ensure_taa_reactive_mask_pipeline.rs"),
                "ensure_taa_reactive_mask_pipeline_for_variant",
                "cached_taa_reactive_pipeline",
            ),
        ];

        for (source, function_name, cache_lookup) in cases {
            assert_cache_hit_precedes_variant_projection(source, function_name, cache_lookup);
        }
    }
}
