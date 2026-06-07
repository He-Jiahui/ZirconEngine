use crate::graphics::scene::resources::PipelineKey;

use super::super::mesh_pipeline::{
    create_motion_vector_mesh_pipeline, FALLBACK_MESH_SHADER,
};
use super::MeshPipelineCache;

const MOTION_VECTOR_SHADER_KEY: &str = "zircon.builtin.motion-vector-mesh@1";

impl MeshPipelineCache {
    pub(crate) fn ensure_motion_vector_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        key: &PipelineKey,
    ) -> &'a wgpu::RenderPipeline {
        if !self.shader_modules.contains_key(MOTION_VECTOR_SHADER_KEY) {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-motion-vector-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(FALLBACK_MESH_SHADER.into()),
            });
            self.shader_modules
                .insert(MOTION_VECTOR_SHADER_KEY.to_string(), module);
        }
        if !self.motion_vector_mesh_pipelines.contains_key(key) {
            let shader = self
                .shader_modules
                .get(MOTION_VECTOR_SHADER_KEY)
                .expect("motion vector mesh shader module cached");
            let pipeline = create_motion_vector_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                wgpu::TextureFormat::Rgba16Float,
                key,
            );
            self.motion_vector_mesh_pipelines
                .insert(key.clone(), pipeline);
        }
        self.motion_vector_mesh_pipelines
            .get(key)
            .expect("motion vector mesh pipeline cached")
    }
}
