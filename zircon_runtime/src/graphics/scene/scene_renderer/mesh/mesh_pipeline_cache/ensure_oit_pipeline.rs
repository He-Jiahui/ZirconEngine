use crate::graphics::scene::resources::ResourceStreamer;

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_oit_mesh_pipeline;
use super::MeshPipelineCache;
use super::shader_source::mesh_pipeline_shader_source_for_geometry_descriptor;

impl MeshPipelineCache {
    pub(crate) fn ensure_oit_pipeline_for_base_variant<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&'a wgpu::RenderPipeline> {
        if self.oit_mesh_variant_pipelines.contains_key(&variant_id) {
            return self.oit_mesh_variant_pipelines.get(&variant_id);
        }
        let (kind, pipeline_key, shader_variant_key) =
            self.pipeline_and_shader_key_for_variant(variant_id)?;
        if kind != MeshPassPipelineKind::Base || !pipeline_key.is_transparent() {
            return None;
        }
        let geometry_source = self.geometry_source_descriptor_for_variant(&shader_variant_key)?;
        let shader_source = mesh_pipeline_shader_source_for_geometry_descriptor(
            streamer,
            &pipeline_key,
            &geometry_source,
        )
        .ok()?
        .into_oit_fragment_store_source()?;
        let shader_key = format!(
            "{}@{}#{}#{}",
            pipeline_key.shader_id,
            pipeline_key.shader_revision,
            shader_variant_key.canonical_string(),
            shader_source.source_hash
        );
        if !self.shader_modules.contains_key(&shader_key) {
            let source =
                self.mesh_pipeline_shader_source_with_cache(shader_source, &shader_variant_key);
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-oit-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), module);
        }
        if !self.oit_mesh_variant_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shader module cached");
            let pipeline = create_oit_mesh_pipeline(
                device,
                &self.oit_mesh_pipeline_layout,
                shader,
                &pipeline_key,
            );
            self.oit_mesh_variant_pipelines.insert(variant_id, pipeline);
        }
        self.oit_mesh_variant_pipelines.get(&variant_id)
    }

    pub(crate) fn oit_fragment_store_layout(&self) -> &wgpu::BindGroupLayout {
        &self.oit_fragment_store_layout
    }
}
