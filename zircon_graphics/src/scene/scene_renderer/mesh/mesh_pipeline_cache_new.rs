use std::collections::HashMap;

use super::mesh_pipeline_cache::MeshPipelineCache;

impl MeshPipelineCache {
    pub(crate) fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        scene_layout: &wgpu::BindGroupLayout,
        model_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-mesh-layout"),
            bind_group_layouts: &[scene_layout, model_layout, texture_layout],
            push_constant_ranges: &[],
        });
        Self {
            target_format,
            mesh_pipeline_layout,
            shader_modules: HashMap::new(),
            mesh_pipelines: HashMap::new(),
        }
    }
}
