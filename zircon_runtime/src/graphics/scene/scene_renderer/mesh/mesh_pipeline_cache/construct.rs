use std::collections::{HashMap, HashSet};

use crate::core::framework::render::builtin_geometry_source_descriptors;
use crate::graphics::pipeline::{PipelineAsyncCompiler, RuntimePipelineCache};
use crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources;
use crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::IrradianceVolumeResources;
use crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::LightCookieAtlasResources;
use crate::graphics::scene::scene_renderer::advanced_lighting::transmission::TransmissionSceneColorFallbackResources;
use crate::graphics::scene::scene_renderer::environment::{
    SceneLightmapResources, SceneReflectionProbeResources,
};
use crate::graphics::shader::ShaderVariantCacheDisk;

use super::forward_shadow_receiver::{
    create_fallback_shadow_atlas_view, create_forward_light_grid_empty_tile_masks_buffer,
    create_forward_light_grid_empty_zbins_buffer, create_forward_light_grid_params_buffer,
    create_forward_shadow_atlas_fallback_globals_buffer,
    create_forward_shadow_atlas_fallback_slot_buffer, create_forward_shadow_compare_sampler,
    create_forward_shadow_receiver_layout,
};
use super::{
    MeshPipelineCache, MeshPipelineVariantRegistry, MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT,
    MAX_ASYNC_SHADER_SOURCE_VALIDATIONS_IN_FLIGHT,
};

impl MeshPipelineCache {
    #[cfg(test)]
    pub(crate) fn new(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self::construct(
            device,
            queue,
            target_format,
            None,
            &std::env::temp_dir().join("zircon-mesh-pipeline-cache-tests"),
            scene_layout,
            material_layout,
            gpu_scene_layout,
            false,
        )
    }

    pub(crate) fn new_with_adapter_info(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        adapter_info: &wgpu::AdapterInfo,
        project_root: &std::path::Path,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
        defer_local_reflection_provider_resources: bool,
    ) -> Self {
        Self::construct(
            device,
            queue,
            target_format,
            Some(adapter_info),
            project_root,
            scene_layout,
            material_layout,
            gpu_scene_layout,
            defer_local_reflection_provider_resources,
        )
    }

    fn construct(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        target_format: wgpu::TextureFormat,
        adapter_info: Option<&wgpu::AdapterInfo>,
        project_root: &std::path::Path,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
        defer_local_reflection_provider_resources: bool,
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
        let environment_only_mesh_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("zircon-environment-only-mesh-layout"),
                bind_group_layouts: &[
                    Some(scene_layout),
                    None,
                    Some(material_layout),
                    Some(gpu_scene_layout),
                ],
                immediate_size: 0,
            });
        let oit_fragment_store_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("zircon-oit-fragment-store-layout"),
                entries: &[
                    oit_storage_entry(0),
                    oit_storage_entry(1),
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });
        let oit_mesh_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("zircon-oit-mesh-layout"),
                bind_group_layouts: &[
                    Some(scene_layout),
                    Some(&forward_shadow_receiver_layout),
                    Some(material_layout),
                    Some(gpu_scene_layout),
                    Some(&oit_fragment_store_layout),
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
        let forward_volumetric_apply =
            VolumetricApplyFallbackResources::new(device, "zircon-forward");
        let forward_volumetric_disabled_params_buffer = forward_volumetric_apply
            .create_disabled_params_buffer(device, "zircon-forward-volumetric-disabled-params");
        let transmission_scene_color = TransmissionSceneColorFallbackResources::new(device, queue);
        let light_cookies = LightCookieAtlasResources::new(device, queue);
        let irradiance_volume = IrradianceVolumeResources::new(device, queue);
        let reflection_probes = if defer_local_reflection_provider_resources {
            SceneReflectionProbeResources::new_environment_only_preview(device)
        } else {
            SceneReflectionProbeResources::new(device)
        };
        let lightmaps = SceneLightmapResources::new(device, queue);
        Self {
            target_format,
            mesh_pipeline_layout,
            environment_only_mesh_pipeline_layout,
            oit_fragment_store_layout,
            oit_mesh_pipeline_layout,
            forward_shadow_receiver_layout,
            forward_shadow_compare_sampler,
            forward_light_grid_params_buffer,
            forward_light_grid_empty_zbins_buffer,
            forward_light_grid_empty_tile_masks_buffer,
            forward_shadow_atlas_fallback_slot_buffer,
            forward_shadow_atlas_fallback_globals_buffer,
            fallback_shadow_atlas_view,
            forward_volumetric_apply,
            forward_volumetric_disabled_params_buffer,
            transmission_scene_color,
            light_cookies,
            irradiance_volume,
            reflection_probes,
            lightmaps,
            shader_modules: HashMap::new(),
            mesh_variant_pipelines: HashMap::new(),
            background_base_pipeline_variants: HashSet::new(),
            background_base_pipeline_failures: HashMap::new(),
            oit_mesh_variant_pipelines: HashMap::new(),
            gbuffer_mesh_pipelines: HashMap::new(),
            depth_prepass_mesh_pipelines: HashMap::new(),
            velocity_mesh_pipelines: HashMap::new(),
            shadow_mesh_pipelines: HashMap::new(),
            taa_reactive_mask_mesh_pipelines: HashMap::new(),
            taa_reactive_material_mask_mesh_pipelines: HashMap::new(),
            pipeline_variant_registry: MeshPipelineVariantRegistry::default(),
            geometry_source_descriptors: builtin_geometry_source_descriptors()
                .into_iter()
                .map(|descriptor| (descriptor.id, descriptor))
                .collect(),
            shader_variant_disk_cache: default_runtime_shader_cache(project_root),
            pending_pipeline_creation_diagnostics: Vec::new(),
            shader_source_validation_compiler: PipelineAsyncCompiler::new(
                "mesh-shader-validate",
                MAX_ASYNC_SHADER_SOURCE_VALIDATIONS_IN_FLIGHT,
            )
            .ok(),
            runtime_pipeline_cache: adapter_info
                .map_or_else(RuntimePipelineCache::disabled, |info| {
                    RuntimePipelineCache::new(device, info, project_root)
                }),
            async_base_pipeline_compiler: PipelineAsyncCompiler::new(
                "mesh-pipeline-compile",
                MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT,
            )
            .ok(),
            allow_async_pipeline_compile: false,
            force_synchronous_base_pipeline_compile: false,
            async_variant_first_frame_miss_count: 0,
            pipeline_creation_metrics: Default::default(),
        }
    }
}

fn oit_storage_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only: false },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

fn default_runtime_shader_cache(project_root: &std::path::Path) -> ShaderVariantCacheDisk {
    ShaderVariantCacheDisk::with_fallback_roots(
        ShaderVariantCacheDisk::default_project_root(project_root),
        [ShaderVariantCacheDisk::default_staged_project_root(
            project_root,
        )],
    )
}
