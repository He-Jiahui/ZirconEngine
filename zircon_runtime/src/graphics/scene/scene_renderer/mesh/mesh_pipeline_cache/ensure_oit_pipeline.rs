use crate::graphics::scene::resources::ResourceStreamer;

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::create_oit_mesh_pipeline;
use super::shader_source::mesh_pipeline_shader_source_for_geometry_descriptor;
use super::{MeshPipelineCache, PipelineCreationTarget};

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
        let shader_source = match mesh_pipeline_shader_source_for_geometry_descriptor(
            streamer,
            &pipeline_key,
            &geometry_source,
        ) {
            Ok(source) => source.into_oit_fragment_store_source()?,
            Err(error) => {
                self.record_shader_variant_assembly_error(&shader_variant_key, error);
                return None;
            }
        };
        let shader_key = format!(
            "{}@{}#{}#{}",
            pipeline_key.shader_id,
            pipeline_key.shader_revision,
            shader_variant_key.canonical_string(),
            shader_source.source_hash
        );
        let error_scope = device.push_error_scope(wgpu::ErrorFilter::Validation);
        if !self.shader_modules.contains_key(&shader_key) {
            let source =
                self.mesh_pipeline_shader_source_with_cache(shader_source, &shader_variant_key)?;
            let creation_started = std::time::Instant::now();
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-oit-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            let creation_elapsed = creation_started.elapsed();
            self.shader_modules.insert(shader_key.clone(), module);
            self.record_shader_module_creation(creation_elapsed);
        }
        if !self.oit_mesh_variant_pipelines.contains_key(&variant_id) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shader module cached");
            let creation_started = std::time::Instant::now();
            let pipeline = create_oit_mesh_pipeline(
                device,
                &self.oit_mesh_pipeline_layout,
                shader,
                &pipeline_key,
                self.runtime_pipeline_cache.cache(),
            );
            let creation_elapsed = creation_started.elapsed();
            self.oit_mesh_variant_pipelines.insert(variant_id, pipeline);
            self.record_render_pipeline_creation(creation_elapsed);
        }
        self.track_pipeline_creation_error_scope(
            &shader_variant_key,
            PipelineCreationTarget::Oit,
            variant_id,
            shader_key,
            error_scope,
        );
        self.oit_mesh_variant_pipelines.get(&variant_id)
    }

    pub(crate) fn oit_fragment_store_layout(&self) -> &wgpu::BindGroupLayout {
        &self.oit_fragment_store_layout
    }
}
