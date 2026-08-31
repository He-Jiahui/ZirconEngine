use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::hybrid_gi::renderer::gpu_resources::{
    GlobalSdfGpuState, GlobalSdfGpuTraceBindings, GlobalSdfGpuTraceClipmap,
};
use crate::hybrid_gi::renderer::HybridGiGpuResources;
use crate::hybrid_gi::scene_representation::{
    HybridGiGlobalSdfSceneState, HybridGiIntersectionBackend, HybridGiLightingSource,
    HybridGiTraceCapabilities, HybridGiTraceCapabilityGraph, HybridGiTraceDomain,
    HybridGiTraceFallbackReason, HybridGiTraceRequest, GLOBAL_SDF_CLIPMAP_COUNT,
};
use crate::hybrid_gi::types::HybridGiPrepareFrame;
use zircon_runtime::graphics::{
    RenderPassBufferUploadSink, RuntimePrepareFrameTransactionRecorder,
};

use super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareExecutionBuffers;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

const PROBE_TRACE_TILE_TRACE_WORKGROUP_SIZE: u32 = 64;
const SURFACE_CACHE_TRACE_ATLAS_COLUMNS: u32 = 8;
const SURFACE_CACHE_TRACE_TILE_EXTENT: u32 = 64;
const FALLBACK_SURFACE_CACHE_TEXTURE_EXTENT: u32 = 1;
const TRACE_BACKEND_SURFACE_CACHE: u32 = 1 << 0;
const TRACE_BACKEND_GLOBAL_SDF: u32 = 1 << 1;
const TRACE_BACKEND_VOXEL_CLIPMAP: u32 = 1 << 2;
const TRACE_BACKEND_HARDWARE_RAY_TRACING: u32 = 1 << 3;
const FALLBACK_TRACE_DIAGNOSTIC_WORD_COUNT: usize = 256;

mod bind_group;

#[cfg(test)]
use bind_group::create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics;
use bind_group::{
    create_probe_trace_tile_dispatch_bind_group,
    create_probe_trace_tile_dispatch_bind_group_from_buffers_with_diagnostics_and_voxel_lookup,
    create_probe_trace_tile_dispatch_bind_group_with_global_sdf,
    create_probe_trace_tile_dispatch_bind_group_with_voxel_lookup,
};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ProbeTraceTileDispatchParams {
    resident_probe_count: u32,
    completed_probe_count: u32,
    tile_count: u32,
    surface_cache_texture_available: u32,
    surface_cache_atlas_width: u32,
    surface_cache_atlas_height: u32,
    surface_cache_atlas_columns: u32,
    surface_cache_tile_extent: u32,
    voxel_cell_descriptor_offset: u32,
    global_sdf_page_count: u32,
    intersection_backend_mask: u32,
    global_sdf_lighting_source: u32,
    fallback_reason: u32,
    voxel_cell_descriptor_count: u32,
    voxel_cell_lookup_clipmap_count: u32,
    _pad2: u32,
    global_sdf_clipmaps: [GlobalSdfGpuTraceClipmap; GLOBAL_SDF_CLIPMAP_COUNT],
}

#[derive(Clone, Copy)]
struct ProbeTraceTileSurfaceCacheParams {
    texture_available: u32,
    atlas_width: u32,
    atlas_height: u32,
    atlas_columns: u32,
    tile_extent: u32,
}

impl ProbeTraceTileSurfaceCacheParams {
    fn unavailable() -> Self {
        Self {
            texture_available: 0,
            atlas_width: FALLBACK_SURFACE_CACHE_TEXTURE_EXTENT,
            atlas_height: FALLBACK_SURFACE_CACHE_TEXTURE_EXTENT,
            atlas_columns: FALLBACK_SURFACE_CACHE_TEXTURE_EXTENT,
            tile_extent: FALLBACK_SURFACE_CACHE_TEXTURE_EXTENT,
        }
    }
}

struct ProbeTraceTileFallbackSurfaceCacheTextures {
    _atlas_texture: wgpu::Texture,
    atlas_view: wgpu::TextureView,
    _depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,
}

pub(super) fn dispatch_probe_trace_tiles(
    resources: &HybridGiGpuResources,
    global_sdf_state: &GlobalSdfGpuState,
    global_sdf_scene_state: &HybridGiGlobalSdfSceneState,
    device: &wgpu::Device,
    buffer_uploads: &mut dyn RenderPassBufferUploadSink,
    frame_transactions: &mut RuntimePrepareFrameTransactionRecorder<'_>,
    encoder: &mut wgpu::CommandEncoder,
    buffers: &HybridGiPrepareExecutionBuffers,
    inputs: &HybridGiPrepareExecutionInputs,
    prepare: &HybridGiPrepareFrame,
    probe_budget: Option<u32>,
) {
    let Some(scene_prepare_resources) = buffers.scene_prepare_resources.as_ref() else {
        return;
    };
    let Some(probe_trace_tile_buffer) = scene_prepare_resources.probe_trace_tile_buffer.as_ref()
    else {
        return;
    };
    if scene_prepare_resources.probe_trace_tile_record_count == 0 {
        return;
    }
    let completed_probe_count = completed_probe_count(inputs, prepare, probe_budget);
    let entry_count = inputs
        .resident_probe_inputs
        .len()
        .saturating_add(completed_probe_count as usize);
    if entry_count == 0 {
        return;
    }

    let available_surface_cache_params =
        probe_trace_tile_surface_cache_params(scene_prepare_resources);
    let global_sdf_bindings = global_sdf_state.create_trace_bindings(
        buffer_uploads,
        frame_transactions,
        global_sdf_scene_state,
    );
    let trace_route = HybridGiTraceCapabilityGraph.select(
        HybridGiTraceRequest {
            domain: HybridGiTraceDomain::Screen,
            prefer_hardware_ray_tracing: false,
        },
        HybridGiTraceCapabilities {
            surface_cache_hzb: available_surface_cache_params.texture_available != 0,
            global_sdf: global_sdf_bindings.page_count != 0,
            voxel_clipmap: buffers.voxel_cell_descriptor_count != 0
                && buffers.voxel_cell_lookup_complete,
            hardware_ray_tracing: false,
            probe_lineage_lighting: true,
        },
    );
    let surface_cache_params = if trace_route.allows(HybridGiIntersectionBackend::SurfaceCacheHzb) {
        available_surface_cache_params
    } else {
        ProbeTraceTileSurfaceCacheParams::unavailable()
    };
    let global_sdf_page_count = if trace_route.allows(HybridGiIntersectionBackend::GlobalSdf) {
        global_sdf_bindings.page_count
    } else {
        0
    };
    let voxel_cell_descriptor_count =
        if trace_route.allows(HybridGiIntersectionBackend::VoxelClipmap) {
            buffers.voxel_cell_descriptor_count as u32
        } else {
            0
        };
    let voxel_cell_lookup_clipmap_count = if voxel_cell_descriptor_count != 0 {
        buffers.voxel_cell_lookup_clipmap_count as u32
    } else {
        0
    };
    let voxel_cell_descriptor_offset = if voxel_cell_descriptor_count != 0 {
        buffers.voxel_cell_descriptor_offset as u32
    } else {
        0
    };
    let fallback_surface_cache = if scene_prepare_resources.atlas_view.is_some()
        && scene_prepare_resources.surface_cache_depth_view.is_some()
    {
        None
    } else {
        Some(create_probe_trace_tile_fallback_surface_cache_textures(
            device,
        ))
    };
    let (surface_cache_atlas_view, surface_cache_depth_view) = match (
        scene_prepare_resources.atlas_view.as_ref(),
        scene_prepare_resources.surface_cache_depth_view.as_ref(),
    ) {
        (Some(atlas), Some(depth)) => (atlas, depth),
        _ => {
            let Some(fallback) = fallback_surface_cache.as_ref() else {
                return;
            };
            (&fallback.atlas_view, &fallback.depth_view)
        }
    };
    let params_buffer =
        create_probe_trace_tile_dispatch_params_buffer_with_route_and_global_sdf_clipmaps(
            device,
            inputs.resident_probe_inputs.len() as u32,
            completed_probe_count,
            scene_prepare_resources.probe_trace_tile_record_count as u32,
            surface_cache_params,
            voxel_cell_descriptor_offset,
            global_sdf_page_count,
            trace_backend_mask(trace_route),
            trace_lighting_source_code(
                trace_route.lighting_source_for(HybridGiIntersectionBackend::GlobalSdf),
            ),
            trace_fallback_reason_code(trace_route.fallback_reason()),
            voxel_cell_descriptor_count,
            voxel_cell_lookup_clipmap_count,
            global_sdf_bindings.clipmaps,
        );
    let bind_group = create_probe_trace_tile_dispatch_bind_group_with_global_sdf(
        device,
        &resources.probe_trace_tile_bind_group_layout,
        &params_buffer,
        &buffers.resident_probe_buffer,
        &buffers.pending_probe_buffer,
        probe_trace_tile_buffer,
        &buffers.trace_lighting_buffer,
        &buffers.trace_diagnostic_buffer,
        surface_cache_atlas_view,
        surface_cache_depth_view,
        &buffers.scene_prepare_descriptor_buffer,
        &buffers.voxel_cell_lookup_buffer,
        &global_sdf_bindings,
    );
    encode_probe_trace_tile_dispatch(
        encoder,
        &resources.probe_trace_tile_pipeline,
        &bind_group,
        entry_count,
    );
}

fn completed_probe_count(
    inputs: &HybridGiPrepareExecutionInputs,
    prepare: &HybridGiPrepareFrame,
    probe_budget: Option<u32>,
) -> u32 {
    let resident_probe_count = inputs.resident_probe_inputs.len() as u32;
    let pending_probe_count = inputs.pending_probe_inputs.len() as u32;
    let free_budget = probe_budget
        .unwrap_or_default()
        .max(resident_probe_count)
        .saturating_sub(resident_probe_count);
    pending_probe_count.min(free_budget.saturating_add(prepare.evictable_probe_ids.len() as u32))
}

fn probe_trace_tile_surface_cache_params(
    scene_prepare_resources: &super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareScenePrepareResources,
) -> ProbeTraceTileSurfaceCacheParams {
    let (atlas_width, atlas_height) = scene_prepare_resources.snapshot.atlas_texture_extent();
    if scene_prepare_resources.atlas_view.is_none()
        || scene_prepare_resources.surface_cache_depth_view.is_none()
        || atlas_width == 0
        || atlas_height == 0
    {
        return ProbeTraceTileSurfaceCacheParams::unavailable();
    }

    ProbeTraceTileSurfaceCacheParams {
        texture_available: 1,
        atlas_width,
        atlas_height,
        atlas_columns: SURFACE_CACHE_TRACE_ATLAS_COLUMNS,
        tile_extent: SURFACE_CACHE_TRACE_TILE_EXTENT,
    }
}

fn create_probe_trace_tile_fallback_surface_cache_textures(
    device: &wgpu::Device,
) -> ProbeTraceTileFallbackSurfaceCacheTextures {
    let (atlas_texture, atlas_view) = create_probe_trace_tile_fallback_texture(
        device,
        "zircon-hybrid-gi-probe-trace-tile-fallback-surface-cache-atlas",
        "zircon-hybrid-gi-probe-trace-tile-fallback-surface-cache-atlas-view",
    );
    let (depth_texture, depth_view) = create_probe_trace_tile_fallback_texture(
        device,
        "zircon-hybrid-gi-probe-trace-tile-fallback-surface-cache-depth",
        "zircon-hybrid-gi-probe-trace-tile-fallback-surface-cache-depth-view",
    );
    ProbeTraceTileFallbackSurfaceCacheTextures {
        _atlas_texture: atlas_texture,
        atlas_view,
        _depth_texture: depth_texture,
        depth_view,
    }
}

fn create_probe_trace_tile_fallback_texture(
    device: &wgpu::Device,
    texture_label: &'static str,
    view_label: &'static str,
) -> (wgpu::Texture, wgpu::TextureView) {
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(texture_label),
        size: wgpu::Extent3d {
            width: FALLBACK_SURFACE_CACHE_TEXTURE_EXTENT,
            height: FALLBACK_SURFACE_CACHE_TEXTURE_EXTENT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::TEXTURE_BINDING,
        view_formats: &[],
    });
    let view = texture.create_view(&wgpu::TextureViewDescriptor {
        label: Some(view_label),
        ..Default::default()
    });
    (texture, view)
}

fn create_probe_trace_tile_dispatch_params_buffer(
    device: &wgpu::Device,
    resident_probe_count: u32,
    completed_probe_count: u32,
    tile_count: u32,
    surface_cache_params: ProbeTraceTileSurfaceCacheParams,
    scene_prepare_descriptor_count: u32,
) -> wgpu::Buffer {
    create_probe_trace_tile_dispatch_params_buffer_with_global_sdf(
        device,
        resident_probe_count,
        completed_probe_count,
        tile_count,
        surface_cache_params,
        scene_prepare_descriptor_count,
        0,
    )
}

fn create_probe_trace_tile_dispatch_params_buffer_with_global_sdf(
    device: &wgpu::Device,
    resident_probe_count: u32,
    completed_probe_count: u32,
    tile_count: u32,
    surface_cache_params: ProbeTraceTileSurfaceCacheParams,
    scene_prepare_descriptor_count: u32,
    global_sdf_page_count: u32,
) -> wgpu::Buffer {
    let mut intersection_backend_mask = 0;
    if surface_cache_params.texture_available != 0 {
        intersection_backend_mask |= TRACE_BACKEND_SURFACE_CACHE;
    }
    if global_sdf_page_count != 0 {
        intersection_backend_mask |= TRACE_BACKEND_GLOBAL_SDF;
    }
    if scene_prepare_descriptor_count != 0 {
        intersection_backend_mask |= TRACE_BACKEND_VOXEL_CLIPMAP;
    }
    let global_sdf_lighting_source = if global_sdf_page_count != 0 {
        trace_lighting_source_code(HybridGiLightingSource::ProbeLineage)
    } else {
        trace_lighting_source_code(HybridGiLightingSource::NeutralAmbient)
    };
    let fallback_reason = (surface_cache_params.texture_available == 0)
        .then_some(HybridGiTraceFallbackReason::ScreenDataUnavailable);
    create_probe_trace_tile_dispatch_params_buffer_with_route(
        device,
        resident_probe_count,
        completed_probe_count,
        tile_count,
        surface_cache_params,
        0,
        global_sdf_page_count,
        intersection_backend_mask,
        global_sdf_lighting_source,
        trace_fallback_reason_code(fallback_reason),
        scene_prepare_descriptor_count,
        scene_prepare_descriptor_count.min(1),
    )
}

#[allow(clippy::too_many_arguments)]
fn create_probe_trace_tile_dispatch_params_buffer_with_route(
    device: &wgpu::Device,
    resident_probe_count: u32,
    completed_probe_count: u32,
    tile_count: u32,
    surface_cache_params: ProbeTraceTileSurfaceCacheParams,
    voxel_cell_descriptor_offset: u32,
    global_sdf_page_count: u32,
    intersection_backend_mask: u32,
    global_sdf_lighting_source: u32,
    fallback_reason: u32,
    voxel_cell_descriptor_count: u32,
    voxel_cell_lookup_clipmap_count: u32,
) -> wgpu::Buffer {
    create_probe_trace_tile_dispatch_params_buffer_with_route_and_global_sdf_clipmaps(
        device,
        resident_probe_count,
        completed_probe_count,
        tile_count,
        surface_cache_params,
        voxel_cell_descriptor_offset,
        global_sdf_page_count,
        intersection_backend_mask,
        global_sdf_lighting_source,
        fallback_reason,
        voxel_cell_descriptor_count,
        voxel_cell_lookup_clipmap_count,
        [GlobalSdfGpuTraceClipmap::zeroed(); GLOBAL_SDF_CLIPMAP_COUNT],
    )
}

#[allow(clippy::too_many_arguments)]
fn create_probe_trace_tile_dispatch_params_buffer_with_route_and_global_sdf_clipmaps(
    device: &wgpu::Device,
    resident_probe_count: u32,
    completed_probe_count: u32,
    tile_count: u32,
    surface_cache_params: ProbeTraceTileSurfaceCacheParams,
    voxel_cell_descriptor_offset: u32,
    global_sdf_page_count: u32,
    intersection_backend_mask: u32,
    global_sdf_lighting_source: u32,
    fallback_reason: u32,
    voxel_cell_descriptor_count: u32,
    voxel_cell_lookup_clipmap_count: u32,
    global_sdf_clipmaps: [GlobalSdfGpuTraceClipmap; GLOBAL_SDF_CLIPMAP_COUNT],
) -> wgpu::Buffer {
    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-tile-dispatch-params"),
        contents: bytemuck::bytes_of(&ProbeTraceTileDispatchParams {
            resident_probe_count,
            completed_probe_count,
            tile_count,
            surface_cache_texture_available: surface_cache_params.texture_available,
            surface_cache_atlas_width: surface_cache_params.atlas_width,
            surface_cache_atlas_height: surface_cache_params.atlas_height,
            surface_cache_atlas_columns: surface_cache_params.atlas_columns,
            surface_cache_tile_extent: surface_cache_params.tile_extent,
            voxel_cell_descriptor_offset,
            global_sdf_page_count,
            intersection_backend_mask,
            global_sdf_lighting_source,
            fallback_reason,
            voxel_cell_descriptor_count,
            voxel_cell_lookup_clipmap_count,
            _pad2: 0,
            global_sdf_clipmaps,
        }),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
}

fn trace_backend_mask(route: crate::hybrid_gi::scene_representation::HybridGiTraceRoute) -> u32 {
    let mut mask = 0;
    for (backend, bit) in [
        (
            HybridGiIntersectionBackend::SurfaceCacheHzb,
            TRACE_BACKEND_SURFACE_CACHE,
        ),
        (
            HybridGiIntersectionBackend::GlobalSdf,
            TRACE_BACKEND_GLOBAL_SDF,
        ),
        (
            HybridGiIntersectionBackend::VoxelClipmap,
            TRACE_BACKEND_VOXEL_CLIPMAP,
        ),
        (
            HybridGiIntersectionBackend::HardwareRayTracing,
            TRACE_BACKEND_HARDWARE_RAY_TRACING,
        ),
    ] {
        if route.allows(backend) {
            mask |= bit;
        }
    }
    mask
}

fn trace_lighting_source_code(source: HybridGiLightingSource) -> u32 {
    match source {
        HybridGiLightingSource::SurfaceCache => 1,
        HybridGiLightingSource::ProbeLineage => 2,
        HybridGiLightingSource::VoxelRadiance => 3,
        HybridGiLightingSource::NeutralAmbient => 0,
    }
}

fn trace_fallback_reason_code(reason: Option<HybridGiTraceFallbackReason>) -> u32 {
    match reason {
        None => 0,
        Some(HybridGiTraceFallbackReason::ScreenDataUnavailable) => 1,
        Some(HybridGiTraceFallbackReason::HardwareRayTracingUnavailable) => 2,
        Some(HybridGiTraceFallbackReason::GlobalSdfUnavailable) => 3,
        Some(HybridGiTraceFallbackReason::IntersectionMiss) => 4,
        Some(HybridGiTraceFallbackReason::LightingUnavailable) => 5,
    }
}

fn encode_probe_trace_tile_dispatch(
    encoder: &mut wgpu::CommandEncoder,
    pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
    entry_count: usize,
) {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some("HybridGiTraceProbeTilesPass"),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.dispatch_workgroups(
        (entry_count as u32).div_ceil(PROBE_TRACE_TILE_TRACE_WORKGROUP_SIZE),
        1,
        1,
    );
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_probe_trace_tile_dispatch_bind_group_layout(
    device: &wgpu::Device,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-tile-dispatch-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::COMPUTE,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            storage_layout_entry(1, true),
            storage_layout_entry(2, true),
            storage_layout_entry(3, true),
            storage_layout_entry(4, false),
            texture_layout_entry(5),
            texture_layout_entry(6),
            storage_layout_entry(7, true),
            storage_layout_entry(8, true),
            storage_layout_entry(9, true),
            storage_layout_entry(10, false),
            storage_layout_entry(11, true),
        ],
    })
}

fn texture_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

fn storage_layout_entry(binding: u32, read_only: bool) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Storage { read_only },
            has_dynamic_offset: false,
            min_binding_size: None,
        },
        count: None,
    }
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_probe_trace_tile_dispatch_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-tile-dispatch-shader"),
        source: wgpu::ShaderSource::Wgsl(
            format!(
                "{}\n{}\n{}\n{}\n{}",
                include_str!("../../../shaders/trace_probe_tiles.wgsl"),
                include_str!("../../../shaders/trace_probe_tiles_global_sdf.wgsl"),
                include_str!("../../../shaders/trace_probe_tiles_voxel.wgsl"),
                include_str!("../../../shaders/trace_probe_tiles_aggregate.wgsl"),
                include_str!("../../../shaders/trace_probe_tiles_output.wgsl"),
            )
            .into(),
        ),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-tile-dispatch-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-tile-dispatch-pipeline"),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some("cs_main"),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

#[cfg(test)]
mod tests;
