use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::RenderMaterialPropertyUniformPayload;

use super::super::fallback::{create_fallback_normal_texture, create_fallback_texture};
use super::super::{GpuMaterialUniformResource, OutputTargetWritebackConverter};
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn new(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            asset_manager,
            models: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            output_target_textures: HashMap::new(),
            post_process_lut_textures: HashMap::new(),
            shaders: HashMap::new(),
            fallback_texture: Arc::new(create_fallback_texture(device, queue, texture_layout)),
            fallback_normal_texture: Arc::new(create_fallback_normal_texture(
                device,
                queue,
                texture_layout,
            )),
            fallback_material_uniform: Arc::new(GpuMaterialUniformResource::from_payload(
                device,
                &RenderMaterialPropertyUniformPayload::default(),
            )),
            fallback_standard_material_uniform: Arc::new(
                GpuMaterialUniformResource::fallback_standard_material(device),
            ),
            output_target_writeback_converter: OutputTargetWritebackConverter::new(device),
            last_material_count: 0,
            last_material_ready_count: 0,
            last_material_fallback_count: 0,
            last_material_validation_error_count: 0,
            last_material_diagnostic_count: 0,
            last_sprite_count: 0,
            last_sprite_ready_count: 0,
            last_sprite_texture_fallback_count: 0,
            last_post_process_lut_request_count: 0,
            last_post_process_lut_ready_count: 0,
            last_post_process_lut_fallback_count: 0,
            last_post_process_lut_2d_strip_ready_count: 0,
            last_post_process_lut_3d_request_count: 0,
            last_post_process_lut_unsupported_shape_count: 0,
            last_output_target_graph_import_report: Default::default(),
            last_output_target_writeback_report: Default::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self::new(asset_manager, device, queue, texture_layout)
    }
}
