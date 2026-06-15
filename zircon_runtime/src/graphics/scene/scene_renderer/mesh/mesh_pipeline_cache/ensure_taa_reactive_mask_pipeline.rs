use crate::graphics::scene::resources::PipelineKey;

use super::super::mesh_pass::{MeshPassPipelineKind, MeshPipelineVariantId};
use super::super::mesh_pipeline::{
    create_taa_reactive_mask_mesh_pipeline, create_taa_reactive_material_mask_mesh_pipeline,
    FALLBACK_MESH_SHADER,
};
use super::MeshPipelineCache;

const TAA_REACTIVE_MASK_MESH_SHADER_KEY: &str = "zircon.builtin.taa-reactive-mask-mesh@1";

impl MeshPipelineCache {
    pub(crate) fn ensure_taa_reactive_mask_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        key: &PipelineKey,
    ) -> &'a wgpu::RenderPipeline {
        if !self
            .shader_modules
            .contains_key(TAA_REACTIVE_MASK_MESH_SHADER_KEY)
        {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-taa-reactive-mask-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(FALLBACK_MESH_SHADER.into()),
            });
            self.shader_modules
                .insert(TAA_REACTIVE_MASK_MESH_SHADER_KEY.to_string(), module);
        }
        if !self.taa_reactive_mask_mesh_pipelines.contains_key(key) {
            let shader = self
                .shader_modules
                .get(TAA_REACTIVE_MASK_MESH_SHADER_KEY)
                .expect("TAA reactive mask mesh shader module cached");
            let pipeline = create_taa_reactive_mask_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                wgpu::TextureFormat::R8Unorm,
                key,
            );
            self.taa_reactive_mask_mesh_pipelines
                .insert(key.clone(), pipeline);
        }
        self.taa_reactive_mask_mesh_pipelines
            .get(key)
            .expect("TAA reactive mask mesh pipeline cached")
    }

    pub(crate) fn ensure_taa_reactive_mask_pipeline_for_variant<'a>(
        &'a mut self,
        device: &wgpu::Device,
        variant_id: MeshPipelineVariantId,
    ) -> Option<&'a wgpu::RenderPipeline> {
        let (kind, pipeline_key) = self.pipeline_key_for_variant(variant_id)?;
        match kind {
            MeshPassPipelineKind::TaaReactiveMask => {
                Some(self.ensure_taa_reactive_mask_pipeline(device, &pipeline_key))
            }
            MeshPassPipelineKind::TaaReactiveMaterialMask => {
                Some(self.ensure_taa_reactive_material_mask_pipeline(device, &pipeline_key))
            }
            _ => None,
        }
    }

    fn ensure_taa_reactive_material_mask_pipeline<'a>(
        &'a mut self,
        device: &wgpu::Device,
        key: &PipelineKey,
    ) -> &'a wgpu::RenderPipeline {
        if !self
            .shader_modules
            .contains_key(TAA_REACTIVE_MASK_MESH_SHADER_KEY)
        {
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some("zircon-taa-reactive-mask-mesh-shader"),
                source: wgpu::ShaderSource::Wgsl(FALLBACK_MESH_SHADER.into()),
            });
            self.shader_modules
                .insert(TAA_REACTIVE_MASK_MESH_SHADER_KEY.to_string(), module);
        }
        if !self
            .taa_reactive_material_mask_mesh_pipelines
            .contains_key(key)
        {
            let shader = self
                .shader_modules
                .get(TAA_REACTIVE_MASK_MESH_SHADER_KEY)
                .expect("TAA reactive mask mesh shader module cached");
            let pipeline = create_taa_reactive_material_mask_mesh_pipeline(
                device,
                &self.mesh_pipeline_layout,
                shader,
                wgpu::TextureFormat::R8Unorm,
                key,
            );
            self.taa_reactive_material_mask_mesh_pipelines
                .insert(key.clone(), pipeline);
        }
        self.taa_reactive_material_mask_mesh_pipelines
            .get(key)
            .expect("TAA reactive material mask mesh pipeline cached")
    }
}
