use std::collections::HashMap;

use super::MeshPipelineCache;

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
            bind_group_layouts: &[Some(scene_layout), Some(model_layout), Some(texture_layout)],
            immediate_size: 0,
        });
        Self {
            target_format,
            mesh_pipeline_layout,
            shader_modules: HashMap::new(),
            mesh_pipelines: HashMap::new(),
        }
    }
}
