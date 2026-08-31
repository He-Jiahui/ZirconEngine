use std::collections::{HashMap, HashSet};

use zr_rhi::RenderAdapterFacts;

use crate::core::framework::render::builtin_geometry_source_descriptors;
use crate::graphics::backend::SystemTextureGenerationLease;
use crate::graphics::pipeline::{PipelineAsyncCompiler, RuntimePipelineCache};
use crate::graphics::scene::gpu_scene::{
    GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING, GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
    gpu_scene_bind_group_layout_entries,
};
use crate::graphics::scene::resources::GpuMeshVertex;
use crate::graphics::scene::scene_renderer::advanced_lighting::froxel::VolumetricApplyFallbackResources;
use crate::graphics::scene::scene_renderer::advanced_lighting::irradiance_volume::IrradianceVolumeResources;
use crate::graphics::scene::scene_renderer::advanced_lighting::light_cookie::LightCookieAtlasResources;
use crate::graphics::scene::scene_renderer::advanced_lighting::transmission::TransmissionSceneColorFallbackResources;
use crate::graphics::scene::scene_renderer::core::material_texture_bind_group_layout_entries;
use crate::graphics::scene::scene_renderer::environment::{
    SceneLightmapResources, SceneReflectionProbeResources, scene_bind_group_layout_entries,
};
use crate::graphics::shader::ShaderVariantCacheDisk;

use super::forward_shadow_receiver::{
    create_fallback_shadow_atlas_view, create_forward_light_grid_empty_tile_masks_buffer,
    create_forward_light_grid_empty_zbins_buffer, create_forward_light_grid_params_buffer,
    create_forward_shadow_atlas_fallback_globals_buffer,
    create_forward_shadow_atlas_fallback_slot_buffer, create_forward_shadow_compare_sampler,
    create_forward_shadow_receiver_layout, forward_shadow_receiver_layout_entries,
};
use super::mesh_shader_fragment_contract_wgpu::MeshShaderFragmentOutputContracts;
use super::mesh_shader_resource_contract::MeshShaderPipelineLayoutContract;
use super::mesh_shader_vertex_contract::MeshShaderVertexLayoutContract;
use super::{
    MAX_ASYNC_BASE_PIPELINES_IN_FLIGHT, MAX_ASYNC_SHADER_SOURCE_VALIDATIONS_IN_FLIGHT,
    MeshPipelineCache, MeshPipelineVariantRegistry,
};

impl MeshPipelineCache {
    #[cfg(test)]
    pub(crate) fn new(
        device: &wgpu::Device,
        _queue: &wgpu::Queue,
        system_textures: &SystemTextureGenerationLease,
        target_format: wgpu::TextureFormat,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
    ) -> Self {
        Self::construct(
            device,
            system_textures,
            target_format,
            None,
            &std::env::temp_dir().join("zircon-mesh-pipeline-cache-tests"),
            scene_layout,
            material_layout,
            gpu_scene_layout,
            false,
        )
    }

    pub(crate) fn new_with_adapter_facts(
        device: &wgpu::Device,
        system_textures: &SystemTextureGenerationLease,
        target_format: wgpu::TextureFormat,
        adapter_facts: &RenderAdapterFacts,
        project_root: &std::path::Path,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
        defer_local_reflection_provider_resources: bool,
    ) -> Self {
        Self::construct(
            device,
            system_textures,
            target_format,
            Some(adapter_facts),
            project_root,
            scene_layout,
            material_layout,
            gpu_scene_layout,
            defer_local_reflection_provider_resources,
        )
    }

    fn construct(
        device: &wgpu::Device,
        system_textures: &SystemTextureGenerationLease,
        target_format: wgpu::TextureFormat,
        adapter_facts: Option<&RenderAdapterFacts>,
        project_root: &std::path::Path,
        scene_layout: &wgpu::BindGroupLayout,
        material_layout: &wgpu::BindGroupLayout,
        gpu_scene_layout: &wgpu::BindGroupLayout,
        defer_local_reflection_provider_resources: bool,
    ) -> Self {
        let scene_layout_entries = scene_bind_group_layout_entries();
        let forward_layout_entries = forward_shadow_receiver_layout_entries();
        let material_layout_entries = material_texture_bind_group_layout_entries();
        let gpu_scene_layout_entries = gpu_scene_shader_contract_layout_entries();
        let oit_fragment_store_entries = oit_fragment_store_layout_entries();
        let mesh_shader_resource_contract =
            MeshShaderPipelineLayoutContract::from_wgpu_bind_group_layouts([
                (0, scene_layout_entries.as_slice()),
                (1, forward_layout_entries.as_slice()),
                (2, material_layout_entries.as_slice()),
                (3, gpu_scene_layout_entries.as_slice()),
            ])
            .expect("engine-owned Mesh bind group layout entries must be unique");
        let environment_only_mesh_shader_resource_contract =
            MeshShaderPipelineLayoutContract::from_wgpu_bind_group_layouts([
                (0, scene_layout_entries.as_slice()),
                (2, material_layout_entries.as_slice()),
                (3, gpu_scene_layout_entries.as_slice()),
            ])
            .expect("engine-owned environment-only Mesh layout entries must be unique");
        let oit_mesh_shader_resource_contract =
            MeshShaderPipelineLayoutContract::from_wgpu_bind_group_layouts([
                (0, scene_layout_entries.as_slice()),
                (1, forward_layout_entries.as_slice()),
                (2, material_layout_entries.as_slice()),
                (3, gpu_scene_layout_entries.as_slice()),
                (4, oit_fragment_store_entries.as_slice()),
            ])
            .expect("engine-owned OIT Mesh layout entries must be unique");
        let mesh_shader_vertex_contract =
            MeshShaderVertexLayoutContract::from_wgpu_vertex_buffer_layouts([
                GpuMeshVertex::layout(),
            ])
            .expect("engine-owned Mesh vertex attributes must have unique shader locations");
        let velocity_mesh_shader_vertex_contract =
            MeshShaderVertexLayoutContract::from_wgpu_vertex_buffer_layouts([
                GpuMeshVertex::layout(),
                GpuMeshVertex::previous_position_layout(),
            ])
            .expect("engine-owned Velocity vertex attributes must have unique shader locations");
        let mesh_shader_fragment_contracts =
            MeshShaderFragmentOutputContracts::from_wgpu_pipeline_targets(target_format)
                .expect("engine-owned Mesh color targets must have numeric fragment types");
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
                entries: &oit_fragment_store_entries,
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
        let transmission_scene_color =
            TransmissionSceneColorFallbackResources::new(device, system_textures);
        let light_cookies = LightCookieAtlasResources::new(device);
        let irradiance_volume = IrradianceVolumeResources::new(device, system_textures);
        let reflection_probes = if defer_local_reflection_provider_resources {
            SceneReflectionProbeResources::new_environment_only_preview(device)
        } else {
            SceneReflectionProbeResources::new(device)
        };
        let lightmaps = SceneLightmapResources::new(device, system_textures);
        Self {
            target_format,
            mesh_pipeline_layout,
            environment_only_mesh_pipeline_layout,
            oit_fragment_store_layout,
            oit_mesh_pipeline_layout,
            mesh_shader_resource_contract,
            environment_only_mesh_shader_resource_contract,
            oit_mesh_shader_resource_contract,
            mesh_shader_vertex_contract,
            velocity_mesh_shader_vertex_contract,
            mesh_shader_fragment_contracts,
            forward_shadow_receiver_layout,
            standard_forward_receiver_bind_group_create_count: 0,
            full_forward_receiver_bind_group_create_count: 0,
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
            pipeline_shader_module_references: Default::default(),
            mesh_variant_pipelines: HashMap::new(),
            background_base_pipeline_variants: HashSet::new(),
            pipeline_failures: HashMap::new(),
            pipeline_unavailable_states: HashMap::new(),
            pbr_ior_forward_base_pipeline_variant: None,
            oit_mesh_variant_pipelines: HashMap::new(),
            gbuffer_mesh_pipelines: HashMap::new(),
            depth_prepass_mesh_pipelines: HashMap::new(),
            hit_proxy_mesh_pipelines: HashMap::new(),
            velocity_mesh_pipelines: HashMap::new(),
            shadow_mesh_pipelines: HashMap::new(),
            taa_reactive_mask_mesh_pipelines: HashMap::new(),
            taa_reactive_material_mask_mesh_pipelines: HashMap::new(),
            pipeline_submission_usage: Default::default(),
            material_pipeline_generation_admissions: Default::default(),
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
            shader_source_validation_states: Default::default(),
            runtime_pipeline_cache: adapter_facts
                .map_or_else(RuntimePipelineCache::disabled, |facts| {
                    RuntimePipelineCache::new(device, facts, project_root)
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

fn gpu_scene_shader_contract_layout_entries() -> [wgpu::BindGroupLayoutEntry; 12] {
    let mut entries = gpu_scene_bind_group_layout_entries(
        wgpu::BufferSize::new(1).expect("Mesh ABI placeholder size is non-zero"),
    );
    // The live layout owns palette capacity; this projection only owns shader-visible ABI.
    for binding in [
        GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
        GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
    ] {
        let entry = entries
            .iter_mut()
            .find(|entry| entry.binding == binding)
            .expect("GPU Scene palette binding must exist in the owner layout");
        match &mut entry.ty {
            wgpu::BindingType::Buffer {
                min_binding_size, ..
            } => *min_binding_size = None,
            _ => unreachable!("GPU Scene palette binding must remain a buffer"),
        }
    }
    entries
}

fn oit_fragment_store_layout_entries() -> [wgpu::BindGroupLayoutEntry; 3] {
    [
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
    ]
}

fn default_runtime_shader_cache(project_root: &std::path::Path) -> ShaderVariantCacheDisk {
    ShaderVariantCacheDisk::with_fallback_roots(
        ShaderVariantCacheDisk::default_project_root(project_root),
        [ShaderVariantCacheDisk::default_staged_project_root(
            project_root,
        )],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gpu_scene_shader_contract_keeps_dynamic_palette_minimums_late_bound() {
        let entries = gpu_scene_shader_contract_layout_entries();

        for binding in [
            GPU_SCENE_SKINNED_JOINT_PALETTE_BINDING,
            GPU_SCENE_PREVIOUS_SKINNED_JOINT_PALETTE_BINDING,
        ] {
            let entry = entries
                .iter()
                .find(|entry| entry.binding == binding)
                .expect("GPU Scene palette binding must exist");
            assert!(matches!(
                &entry.ty,
                wgpu::BindingType::Buffer {
                    min_binding_size: None,
                    ..
                }
            ));
        }
    }
}
