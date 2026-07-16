use super::super::lighting_bind_group_layout::create_lighting_bind_group_layout;
use super::super::lighting_pipeline::create_lighting_pipeline;
use super::DeferredSceneResources;
use crate::asset::ProjectAssetManager;
use crate::core::framework::render::ShadingModelDescriptor;
use crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources;
use crate::graphics::scene::scene_renderer::environment::{
    LightmapGpuBindings, ReflectionProbeGpuBindings,
};
use crate::graphics::scene::scene_renderer::shadow::slot::{GpuShadowGlobals, GpuShadowSlot};
use crate::graphics::types::GraphicsError;
use wgpu::util::DeviceExt;

impl DeferredSceneResources {
    pub(crate) fn new(
        device: &wgpu::Device,
        asset_manager: &ProjectAssetManager,
        scene_layout: &wgpu::BindGroupLayout,
        _material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
        reflection_probe_bindings: ReflectionProbeGpuBindings,
        lightmap_bindings: LightmapGpuBindings,
        target_format: wgpu::TextureFormat,
        plugin_shading_models: &[ShadingModelDescriptor],
        volumetric_enabled: bool,
    ) -> Result<Self, GraphicsError> {
        let lighting_bind_group_layout = create_lighting_bind_group_layout(device);
        let lighting_pipeline = create_lighting_pipeline(
            device,
            asset_manager,
            scene_layout,
            &lighting_bind_group_layout,
            gpu_scene_layout,
            target_format,
            plugin_shading_models,
            false,
            volumetric_enabled,
        )?;
        let lighting_subsurface_mrt_pipeline = create_lighting_pipeline(
            device,
            asset_manager,
            scene_layout,
            &lighting_bind_group_layout,
            gpu_scene_layout,
            target_format,
            plugin_shading_models,
            true,
            volumetric_enabled,
        )?;
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
        let volumetric_apply = VolumetricApplyFallbackResources::new(device, "zircon-deferred");

        Ok(Self {
            lighting_bind_group_layout,
            lighting_pipeline,
            lighting_subsurface_mrt_pipeline,
            shadow_compare_sampler,
            shadow_atlas_fallback_view,
            shadow_atlas_fallback_slot_buffer,
            shadow_atlas_fallback_globals_buffer,
            reflection_probe_bindings,
            lightmap_bindings,
            volumetric_apply,
        })
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
