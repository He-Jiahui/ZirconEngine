use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::hybrid_gi::types::HybridGiPrepareFrame;

use super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareExecutionBuffers;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

const PROBE_TRACE_TILE_TRACE_WORKGROUP_SIZE: u32 = 64;
const SURFACE_CACHE_TRACE_ATLAS_COLUMNS: u32 = 8;
const SURFACE_CACHE_TRACE_TILE_EXTENT: u32 = 64;
const FALLBACK_SURFACE_CACHE_TEXTURE_EXTENT: u32 = 1;

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
    scene_prepare_descriptor_count: u32,
    _pad0: u32,
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
    device: &wgpu::Device,
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

    let surface_cache_params = probe_trace_tile_surface_cache_params(scene_prepare_resources);
    let fallback_surface_cache = if scene_prepare_resources.atlas_view.is_some()
        && scene_prepare_resources.surface_cache_depth_view.is_some()
    {
        None
    } else {
        Some(create_probe_trace_tile_fallback_surface_cache_textures(
            device,
        ))
    };
    let fallback_surface_cache = fallback_surface_cache.as_ref();
    let surface_cache_atlas_view = scene_prepare_resources
        .atlas_view
        .as_ref()
        .or_else(|| fallback_surface_cache.map(|fallback| &fallback.atlas_view))
        .expect("probe trace tile fallback atlas view must exist");
    let surface_cache_depth_view = scene_prepare_resources
        .surface_cache_depth_view
        .as_ref()
        .or_else(|| fallback_surface_cache.map(|fallback| &fallback.depth_view))
        .expect("probe trace tile fallback depth view must exist");
    let params_buffer = create_probe_trace_tile_dispatch_params_buffer(
        device,
        inputs.resident_probe_inputs.len() as u32,
        completed_probe_count,
        scene_prepare_resources.probe_trace_tile_record_count as u32,
        surface_cache_params,
        buffers.scene_prepare_descriptor_count as u32,
    );
    let bind_group_layout = create_probe_trace_tile_dispatch_bind_group_layout(device);
    let pipeline = create_probe_trace_tile_dispatch_pipeline(device, &bind_group_layout);
    let bind_group = create_probe_trace_tile_dispatch_bind_group(
        device,
        &bind_group_layout,
        &params_buffer,
        &buffers.resident_probe_buffer,
        &buffers.pending_probe_buffer,
        probe_trace_tile_buffer,
        &buffers.trace_lighting_buffer,
        surface_cache_atlas_view,
        surface_cache_depth_view,
        &buffers.scene_prepare_descriptor_buffer,
    );
    encode_probe_trace_tile_dispatch(encoder, &pipeline, &bind_group, entry_count);
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
            scene_prepare_descriptor_count,
            _pad0: 0,
        }),
        usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
    })
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

fn create_probe_trace_tile_dispatch_bind_group_layout(
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

fn create_probe_trace_tile_dispatch_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-tile-dispatch-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../../shaders/trace_probe_tiles.wgsl").into(),
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

fn create_probe_trace_tile_dispatch_bind_group(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    params_buffer: &wgpu::Buffer,
    resident_probe_buffer: &wgpu::Buffer,
    pending_probe_buffer: &wgpu::Buffer,
    probe_trace_tile_buffer: &wgpu::Buffer,
    trace_lighting_buffer: &wgpu::Buffer,
    surface_cache_atlas_view: &wgpu::TextureView,
    surface_cache_depth_view: &wgpu::TextureView,
    scene_prepare_descriptor_buffer: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-probe-trace-tile-dispatch-bind-group"),
        layout: bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: resident_probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: pending_probe_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: probe_trace_tile_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: trace_lighting_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: wgpu::BindingResource::TextureView(surface_cache_atlas_view),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: wgpu::BindingResource::TextureView(surface_cache_depth_view),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: scene_prepare_descriptor_buffer.as_entire_binding(),
            },
        ],
    })
}

#[cfg(test)]
mod tests;
