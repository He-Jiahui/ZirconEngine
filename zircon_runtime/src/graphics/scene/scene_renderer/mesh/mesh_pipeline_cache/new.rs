use std::collections::HashMap;

use super::forward_shadow_receiver::{
    create_fallback_shadow_atlas_view, create_forward_light_grid_empty_tile_masks_buffer,
    create_forward_light_grid_empty_zbins_buffer, create_forward_light_grid_params_buffer,
    create_forward_shadow_atlas_fallback_globals_buffer,
    create_forward_shadow_atlas_fallback_slot_buffer, create_forward_shadow_compare_sampler,
    create_forward_shadow_receiver_layout,
};
use super::{MeshPipelineCache, MeshPipelineVariantRegistry};

impl MeshPipelineCache {
    pub(crate) fn new(
        device: &wgpu::Device,
        target_format: wgpu::TextureFormat,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let forward_shadow_receiver_layout = create_forward_shadow_receiver_layout(device);
        let mesh_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("zircon-mesh-layout"),
            bind_group_layouts: &[
                Some(scene_layout),
                Some(&forward_shadow_receiver_layout),
                Some(material_layout),
                Some(gpu_scene_layout),
            ],
            immediate_size: 0,
        });
        let forward_shadow_compare_sampler = create_forward_shadow_compare_sampler(device);
        let forward_light_grid_params_buffer = create_forward_light_grid_params_buffer(device);
        let forward_light_grid_empty_zbins_buffer =
            create_forward_light_grid_empty_zbins_buffer(device);
        let forward_light_grid_empty_tile_masks_buffer =
            create_forward_light_grid_empty_tile_masks_buffer(device);
        let forward_shadow_atlas_fallback_slot_buffer =
            create_forward_shadow_atlas_fallback_slot_buffer(device);
        let forward_shadow_atlas_fallback_globals_buffer =
            create_forward_shadow_atlas_fallback_globals_buffer(device);
        let fallback_shadow_atlas_view = create_fallback_shadow_atlas_view(device);
        Self {
            target_format,
            mesh_pipeline_layout,
            forward_shadow_receiver_layout,
            forward_shadow_compare_sampler,
            forward_light_grid_params_buffer,
            forward_light_grid_empty_zbins_buffer,
            forward_light_grid_empty_tile_masks_buffer,
            forward_shadow_atlas_fallback_slot_buffer,
            forward_shadow_atlas_fallback_globals_buffer,
            fallback_shadow_atlas_view,
            shader_modules: HashMap::new(),
            mesh_pipelines: HashMap::new(),
            velocity_mesh_pipelines: HashMap::new(),
            taa_reactive_mask_mesh_pipelines: HashMap::new(),
            taa_reactive_material_mask_mesh_pipelines: HashMap::new(),
            pipeline_variant_registry: MeshPipelineVariantRegistry::default(),
        }
    }
}
