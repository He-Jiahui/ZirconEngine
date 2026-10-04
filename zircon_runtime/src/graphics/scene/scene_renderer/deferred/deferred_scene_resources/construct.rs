use super::super::lighting_bind_group_layout::create_lighting_bind_group_layout;
use super::super::lighting_pipeline::DeferredLightingPipelineCache;
use super::DeferredSceneResources;
use crate::asset::ProjectAssetManager;
use crate::core::framework::render::ShadingModelDescriptor;
use crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources;
use crate::graphics::scene::scene_renderer::environment::{
    LightmapGpuBindings, ReflectionProbeGpuBindings,
};
use crate::graphics::scene::scene_renderer::shadow::slot::{GpuShadowGlobals, GpuShadowSlot};
use crate::graphics::scene::scene_renderer::SceneRendererDeferredLightingProfile;
use crate::graphics::types::GraphicsError;
use std::time::{Duration, Instant};
use wgpu::util::DeviceExt;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct DeferredSceneResourcesStartupReport {
    lighting_pipelines: Duration,
    lighting_shader_source_assembly: Duration,
    lighting_pipeline_foundation: Duration,
    lighting_standard_pipeline: Duration,
    fallback_resources: Duration,
}

impl DeferredSceneResourcesStartupReport {
    pub(crate) const fn lighting_pipelines(self) -> Duration {
        self.lighting_pipelines
    }

    pub(crate) const fn lighting_shader_source_assembly(self) -> Duration {
        self.lighting_shader_source_assembly
    }

    pub(crate) const fn lighting_pipeline_foundation(self) -> Duration {
        self.lighting_pipeline_foundation
    }

    pub(crate) const fn lighting_standard_pipeline(self) -> Duration {
        self.lighting_standard_pipeline
    }

    pub(crate) const fn fallback_resources(self) -> Duration {
        self.fallback_resources
    }
}

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
        deferred_lighting_profile: SceneRendererDeferredLightingProfile,
    ) -> Result<(Self, DeferredSceneResourcesStartupReport), GraphicsError> {
        let lighting_pipelines_started = Instant::now();
        let lighting_bind_group_layout =
            create_lighting_bind_group_layout(device, deferred_lighting_profile);
        let (lighting_pipelines, lighting_pipeline_startup) = DeferredLightingPipelineCache::new(
            device,
            asset_manager,
            scene_layout,
            &lighting_bind_group_layout,
            gpu_scene_layout,
            target_format,
            plugin_shading_models,
            volumetric_enabled,
            deferred_lighting_profile,
        )?;
        let lighting_pipelines_elapsed = lighting_pipelines_started.elapsed();

        // 完整光照所需的 shadow/volumetric 槽位保留占位资源；真实场景输入只在执行时按 Option 借用，
        // 不替换这些由 owner 保留的 fallback 句柄。
        let fallback_resources_started = Instant::now();
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
        let fallback_resources_elapsed = fallback_resources_started.elapsed();

        Ok((
            Self {
                deferred_lighting_profile,
                lighting_bind_group_layout,
                lighting_pipelines,
                shadow_compare_sampler,
                shadow_atlas_fallback_view,
                shadow_atlas_fallback_slot_buffer,
                shadow_atlas_fallback_globals_buffer,
                reflection_probe_bindings,
                lightmap_bindings,
                volumetric_apply,
            },
            DeferredSceneResourcesStartupReport {
                lighting_pipelines: lighting_pipelines_elapsed,
                lighting_shader_source_assembly: lighting_pipeline_startup.shader_source_assembly(),
                lighting_pipeline_foundation: lighting_pipeline_startup.pipeline_foundation(),
                lighting_standard_pipeline: lighting_pipeline_startup.standard_pipeline(),
                fallback_resources: fallback_resources_elapsed,
            },
        ))
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

#[cfg(test)]
#[path = "tests/construct.rs"]
mod tests;
