use crate::graphics::scene::resources::PipelineKey;

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::{create_velocity_mesh_pipeline, FALLBACK_MESH_SHADER};
use super::MeshPipelineCache;

const VELOCITY_MESH_SHADER_KEY: &str = "zircon.builtin.velocity-mesh@1";

impl MeshPipelineCache {
    pub(crate) fn ensure_velocity_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        key: &PipelineKey,
    ) -> &'a wgpu::RenderPipeline {
        if !self.shader_modules.contains_key(VELOCITY_MESH_SHADER_KEY) {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-velocity-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(FALLBACK_MESH_SHADER.into()),
            });
            self.shader_modules
                .insert(VELOCITY_MESH_SHADER_KEY.to_string(), module);
        }
        if !self.velocity_mesh_pipelines.contains_key(key) {
            let shader = self
                .shader_modules
                .get(VELOCITY_MESH_SHADER_KEY)
                .expect("velocity mesh shader module cached");
            let pipeline = create_velocity_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                wgpu::TextureFormat::Rg16Float,
                key,
            );
            self.velocity_mesh_pipelines.insert(key.clone(), pipeline);
        }
        self.velocity_mesh_pipelines
            .get(key)
            .expect("velocity mesh pipeline cached")
    }

    pub(crate) fn ensure_velocity_pipeline_for_variant<'a>(
        &'a mut self,
        device: &wgpu::Device,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let (kind, pipeline_key) = self.pipeline_key_for_variant(variant_id)?;
        (kind == MeshPassPipelineKind::Velocity)
            .then(|| self.ensure_velocity_pipeline(device, &pipeline_key))
    }
}
