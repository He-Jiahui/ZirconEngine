use crate::scene::resources::{PipelineKey, ResourceStreamer};

use super::create_mesh_pipeline::create_mesh_pipeline;
use super::fallback_mesh_shader_source::FALLBACK_MESH_SHADER;
use super::mesh_pipeline_cache::MeshPipelineCache;

impl MeshPipelineCache {
    pub(crate) fn ensure_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        streamer: &ResourceStreamer,
        key: &PipelineKey,
    ) -> &'a wgpu::RenderPipeline {
        let shader_key = format!("{}@{}", key.shader_id, key.shader_revision);
        if !self.shader_modules.contains_key(&shader_key) {
            let source = streamer
                .shader_source(&key.shader_id)
                .unwrap_or(FALLBACK_MESH_SHADER);
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
}
