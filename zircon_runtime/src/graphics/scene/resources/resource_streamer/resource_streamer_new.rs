use std::collections::HashMap;
use std::sync::Arc;

use crate::asset::pipeline::manager::ProjectAssetManager;
use crate::core::framework::render::RenderMaterialPropertyUniformPayload;

use super::super::fallback::create_fallback_texture;
use super::super::GpuMaterialUniformResource;
use super::ResourceStreamer;

impl ResourceStreamer {
    pub(crate) fn new(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self {
            asset_manager,
            material_bind_group_layout: material_layout.clone(),
            models: HashMap::new(),
            meshes: HashMap::new(),
            materials: HashMap::new(),
            textures: HashMap::new(),
            shaders: HashMap::new(),
            fallback_texture: Arc::new(create_fallback_texture(device, queue, texture_layout)),
            fallback_material_uniform: Arc::new(GpuMaterialUniformResource::from_payload(
                device,
                material_layout,
                &RenderMaterialPropertyUniformPayload::default(),
            )),
            last_material_count: 0,
            last_material_ready_count: 0,
            last_material_fallback_count: 0,
            last_material_validation_error_count: 0,
            last_material_diagnostic_count: 0,
            last_sprite_count: 0,
            last_sprite_ready_count: 0,
            last_sprite_texture_fallback_count: 0,
        }
    }

    #[cfg(test)]
    pub(crate) fn new_for_test(
        asset_manager: Arc<ProjectAssetManager>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        let material_layout = create_test_material_bind_group_layout(device);
        Self::new(
            asset_manager,
            device,
            queue,
            texture_layout,
            &material_layout,
        )
    }
}

#[cfg(test)]
fn create_test_material_bind_group_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-test-material-property-uniform-layout"),
        entries: &[wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Buffer {
                ty: wgpu::BufferBindingType::Uniform,
                has_dynamic_offset: false,
                min_binding_size: None,
            },
            count: None,
        }],
    })
}
