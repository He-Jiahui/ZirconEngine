use super::super::geometry_pipeline::create_geometry_pipeline;
use super::super::lighting_bind_group_layout::create_lighting_bind_group_layout;
use super::super::lighting_pipeline::create_lighting_pipeline;
use super::DeferredSceneResources;
use crate::graphics::scene::scene_renderer::shadow::slot::{GpuShadowGlobals, GpuShadowSlot};
use wgpu::util::DeviceExt;

impl DeferredSceneResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
        target_format: wgpu::TextureFormat,
    ) -> Self {
        let geometry_pipeline =
            create_geometry_pipeline(device, scene_layout, material_layout, gpu_scene_layout);
        let lighting_bind_group_layout = create_lighting_bind_group_layout(device);
        let lighting_pipeline = create_lighting_pipeline(
            device,
            scene_layout,
            &lighting_bind_group_layout,
            gpu_scene_layout,
            target_format,
        );
        let shadow_compare_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("zircon-deferred-shadow-compare-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Nearest,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_atlas_fallback_view = create_shadow_atlas_fallback_view(device);
        let shadow_atlas_fallback_slot_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-deferred-shadow-atlas-slots-fallback"),
                contents: bytemuck::bytes_of(&GpuShadowSlot::disabled()),
                usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            });
        let shadow_atlas_fallback_globals_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("zircon-deferred-shadow-atlas-globals-fallback"),
                contents: bytemuck::bytes_of(&GpuShadowGlobals::disabled(1, 1)),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        Self {
            geometry_pipeline,
            lighting_bind_group_layout,
            lighting_pipeline,
            shadow_compare_sampler,
            shadow_atlas_fallback_view,
            shadow_atlas_fallback_slot_buffer,
            shadow_atlas_fallback_globals_buffer,
        }
    }
}

fn create_shadow_atlas_fallback_view(device: &wgpu::Device) -> wgpu::TextureView {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-deferred-shadow-atlas-fallback-texture"),
        size: wgpu::Extent3d {
            width: 1,
            height: 1,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: super::super::super::core::DEPTH_FORMAT,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    texture.create_view(&wgpu::TextureViewDescriptor::default())
}
