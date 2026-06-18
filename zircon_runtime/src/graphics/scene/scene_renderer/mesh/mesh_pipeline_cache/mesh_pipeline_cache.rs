use std::collections::HashMap;

use crate::core::framework::render::{
    ShaderQualityTier, ShaderVariantKey, ShaderVariantMissReport,
};
use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};
use crate::graphics::shader::ShaderVariantCacheDisk;

use super::{MeshPipelineVariantRegistry, MeshPipelineVariantResolver};

pub(crate) struct MeshPipelineCache {
    pub(in crate::graphics::scene::scene_renderer::mesh) target_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_pipeline_layout: wgpu::PipelineLayout,
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
    pub(in crate::graphics::scene::scene_renderer::mesh) shader_modules:
        HashMap<String, wgpu::ShaderModule>,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_variant_pipelines:
        HashMap<MeshPipelineVariantId, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) velocity_mesh_pipelines:
        HashMap<PipelineKey, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) taa_reactive_mask_mesh_pipelines:
        HashMap<PipelineKey, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) taa_reactive_material_mask_mesh_pipelines:
        HashMap<PipelineKey, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) pipeline_variant_registry:
        MeshPipelineVariantRegistry,
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
        self.pipeline_variant_registry
            .resolve_variant(kind, pipeline_key, shader_quality)
    }

    pub(crate) fn pipeline_key_for_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> Option<(MeshPassPipelineKind, PipelineKey)> {
        let key = self.pipeline_variant_registry.key_for_variant(variant_id)?;
        Some((key.kind(), key.pipeline_key().clone()))
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

    pub(crate) fn reset_shader_variant_miss_report(&mut self) {
        self.pipeline_variant_registry.reset_miss_report();
    }

    pub(crate) const fn shader_variant_miss_report(&self) -> ShaderVariantMissReport {
        self.pipeline_variant_registry.miss_report()
    }

    pub(crate) fn record_shader_variant_disk_hit(&mut self) {
        self.pipeline_variant_registry.record_disk_hit();
    }

    pub(crate) fn record_shader_variant_disk_write(&mut self) {
        self.pipeline_variant_registry.record_disk_write();
    }

    pub(crate) fn record_shader_variant_disk_error(&mut self) {
        self.pipeline_variant_registry.record_disk_error();
    }

    pub(crate) fn record_shader_variant_compile_miss(&mut self) {
        self.pipeline_variant_registry.record_compile_miss();
    }
}

impl MeshPipelineVariantResolver for MeshPipelineCache {
    fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
        shader_quality: ShaderQualityTier,
    ) -> MeshPipelineVariantId {
        MeshPipelineCache::resolve_variant(self, kind, pipeline_key, shader_quality)
    }
}
