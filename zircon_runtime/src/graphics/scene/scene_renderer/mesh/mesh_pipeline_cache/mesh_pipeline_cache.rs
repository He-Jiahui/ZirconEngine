use std::collections::HashMap;

use crate::graphics::scene::resources::PipelineKey;
use crate::graphics::scene::scene_renderer::mesh::mesh_pass::{
    MeshPassPipelineKind, MeshPipelineVariantId,
};

use super::{MeshPipelineVariantRegistry, MeshPipelineVariantResolver};

pub(crate) struct MeshPipelineCache {
    pub(in crate::graphics::scene::scene_renderer::mesh) target_format: wgpu::TextureFormat,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_pipeline_layout: wgpu::PipelineLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_layout:
        wgpu::BindGroupLayout,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_uniform_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_receiver_disabled_uniform_buffer:
        wgpu::Buffer,
    pub(in crate::graphics::scene::scene_renderer::mesh) forward_shadow_compare_sampler:
        wgpu::Sampler,
    pub(in crate::graphics::scene::scene_renderer::mesh) fallback_shadow_map_view:
        wgpu::TextureView,
    pub(in crate::graphics::scene::scene_renderer::mesh) shader_modules:
        HashMap<String, wgpu::ShaderModule>,
    pub(in crate::graphics::scene::scene_renderer::mesh) mesh_pipelines:
        HashMap<PipelineKey, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) motion_vector_mesh_pipelines:
        HashMap<PipelineKey, wgpu::RenderPipeline>,
    pub(in crate::graphics::scene::scene_renderer::mesh) pipeline_variant_registry:
        MeshPipelineVariantRegistry,
}

impl MeshPipelineCache {
    pub(crate) fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
    ) -> MeshPipelineVariantId {
        self.pipeline_variant_registry
            .resolve_variant(kind, pipeline_key)
    }

    pub(crate) fn pipeline_key_for_variant(
        &self,
        variant_id: MeshPipelineVariantId,
    ) -> Option<(MeshPassPipelineKind, PipelineKey)> {
        let key = self.pipeline_variant_registry.key_for_variant(variant_id)?;
        Some((key.kind(), key.pipeline_key().clone()))
    }
}

impl MeshPipelineVariantResolver for MeshPipelineCache {
    fn resolve_variant(
        &mut self,
        kind: MeshPassPipelineKind,
        pipeline_key: &PipelineKey,
    ) -> MeshPipelineVariantId {
        MeshPipelineCache::resolve_variant(self, kind, pipeline_key)
    }
}
