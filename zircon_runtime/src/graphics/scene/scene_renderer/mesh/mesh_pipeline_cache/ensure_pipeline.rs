use crate::graphics::scene::resources::{PipelineKey, ResourceStreamer};

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::{create_mesh_pipeline, FALLBACK_MESH_SHADER};
use super::MeshPipelineCache;

impl MeshPipelineCache {
    pub(crate) fn pipeline_uses_builtin_fallback_shader(
        &self,
        streamer: &ResourceStreamer,
        key: &PipelineKey,
    ) -> bool {
        key.uses_fallback_shader() || streamer.shader_source(&key.shader_id).is_none()
    }

    pub(crate) fn ensure_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        key: &PipelineKey,
    ) -> &'a wgpu::RenderPipeline {
        let shader_key = format!("{}@{}", key.shader_id, key.shader_revision);
        if !self.shader_modules.contains_key(&shader_key) {
            let source = mesh_pipeline_shader_source(streamer, key);
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            self.shader_modules.insert(shader_key.clone(), module);
        }
        if !self.mesh_pipelines.contains_key(key) {
            let shader = self
                .shader_modules
                .get(&shader_key)
                .expect("shader module cached");
            let pipeline = create_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                self.target_format,
                key,
            );
            self.mesh_pipelines.insert(key.clone(), pipeline);
        }
        self.mesh_pipelines.get(key).expect("mesh pipeline cached")
    }

    pub(crate) fn ensure_pipeline_for_variant<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let (kind, pipeline_key) = self.pipeline_key_for_variant(variant_id)?;
        (kind == MeshPassPipelineKind::Base)
            .then(|| self.ensure_pipeline(device, streamer, &pipeline_key))
    }
}

fn mesh_pipeline_shader_source<'a>(streamer: &'a ResourceStreamer, key: &PipelineKey) -> &'a str {
    if key.uses_fallback_shader() {
        FALLBACK_MESH_SHADER
    } else {
        streamer
            .shader_source(&key.shader_id)
            .unwrap_or(FALLBACK_MESH_SHADER)
    }
}
