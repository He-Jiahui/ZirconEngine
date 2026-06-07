use std::collections::HashMap;

use super::forward_shadow_receiver::{
    create_fallback_shadow_map_view, create_forward_shadow_compare_sampler,
    create_forward_shadow_receiver_layout, create_forward_shadow_receiver_uniform_buffer,
};
use super::MeshPipelineCache;

impl MeshPipelineCache {
    pub(crate) fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        scene_layout: &wgpu::BindGroupLayout,
        model_layout: &wgpu::BindGroupLayout,
        texture_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let forward_shadow_receiver_layout = create_forward_shadow_receiver_layout(device);
        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-mesh-layout"),
            bind_group_layouts: &[
                Some(scene_layout),
                Some(model_layout),
                Some(texture_layout),
                Some(material_layout),
                Some(&forward_shadow_receiver_layout),
            ],
            immediate_size: 0,
        });
        let forward_shadow_receiver_uniform_buffer = create_forward_shadow_receiver_uniform_buffer(
            device,
            "zircon-forward-shadow-receiver-uniform",
        );
        let forward_shadow_receiver_disabled_uniform_buffer =
            create_forward_shadow_receiver_uniform_buffer(
                device,
                "zircon-forward-shadow-receiver-disabled-uniform",
            );
        let forward_shadow_compare_sampler = create_forward_shadow_compare_sampler(device);
        let fallback_shadow_map_view = create_fallback_shadow_map_view(device);
        Self {
            target_format,
            mesh_pipeline_layout,
            forward_shadow_receiver_layout,
            forward_shadow_receiver_uniform_buffer,
            forward_shadow_receiver_disabled_uniform_buffer,
            forward_shadow_compare_sampler,
            fallback_shadow_map_view,
            shader_modules: HashMap::new(),
            mesh_pipelines: HashMap::new(),
            motion_vector_mesh_pipelines: HashMap::new(),
        }
    }
}
