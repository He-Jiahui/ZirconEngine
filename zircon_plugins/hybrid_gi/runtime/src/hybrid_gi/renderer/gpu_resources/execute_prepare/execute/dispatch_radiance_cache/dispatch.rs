use bytemuck::{Pod, Zeroable};
use zircon_runtime::core::framework::render::{
    RenderHybridGiRadianceCacheGpuStage, RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT,
};
use zircon_runtime::graphics::RenderPassBufferUploadSink;

use crate::hybrid_gi::{
    HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT,
    HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT,
};

use super::super::super::super::buffer_helpers::{
    create_pod_storage_buffer, create_u32_storage_buffer,
};
use super::super::super::super::gpu_radiance_cache_storage_entry::{
    GpuRadianceCacheStorageEntry, GPU_RADIANCE_CACHE_PROBE_ATLAS_WORD_COUNT,
    GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_OFFSET, GPU_RADIANCE_CACHE_PROBE_TILE_EXTENT,
};
use super::super::super::super::hybrid_gi_gpu_resources::HybridGiGpuResources;
use super::super::super::super::radiance_cache_gpu_state::RadianceCacheGpuState;
use super::super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareExecutionBuffers;
use super::super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

const RADIANCE_CACHE_PARAM_MARK: usize = 0;
const RADIANCE_CACHE_PARAM_ALLOCATE: usize = 1;
const RADIANCE_CACHE_PARAM_TRACE: usize = 2;
const RADIANCE_CACHE_PARAM_FILTER: usize = 3;
const RADIANCE_CACHE_PARAM_BORDER_MIP: usize = 4;
const RADIANCE_CACHE_PARAM_CONSUME: usize = 5;
const RADIANCE_CACHE_STAGE_MARK: u32 = RenderHybridGiRadianceCacheGpuStage::Mark.index() as u32;
const RADIANCE_CACHE_STAGE_ALLOCATE: u32 =
    RenderHybridGiRadianceCacheGpuStage::Allocate.index() as u32;
const RADIANCE_CACHE_STAGE_TRACE: u32 = RenderHybridGiRadianceCacheGpuStage::Trace.index() as u32;
const RADIANCE_CACHE_STAGE_FILTER: u32 = RenderHybridGiRadianceCacheGpuStage::Filter.index() as u32;
const RADIANCE_CACHE_STAGE_BORDER_MIP: u32 =
    RenderHybridGiRadianceCacheGpuStage::BorderMip.index() as u32;
const RADIANCE_CACHE_WORKGROUP_SIZE: u32 = 64;
pub(in crate::hybrid_gi::renderer) const RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT: usize =
    RENDER_HYBRID_GI_RADIANCE_CACHE_GPU_STAGE_COUNT;
pub(in crate::hybrid_gi::renderer) const RADIANCE_CACHE_DISPATCH_COUNTER_WORD_OFFSET: usize =
    HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT;
pub(in crate::hybrid_gi::renderer) const RADIANCE_CACHE_MARK_WORD_COUNT: usize =
    RADIANCE_CACHE_DISPATCH_COUNTER_WORD_OFFSET + RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT;
const RADIANCE_CACHE_SHADER_SOURCE: &str =
    include_str!("../../../../shaders/radiance_cache_update.wgsl");

#[derive(Clone, Copy)]
struct RadianceCacheUpdateStage {
    parameter_index: usize,
    stage: u32,
    label: &'static str,
}

const RADIANCE_CACHE_UPDATE_STAGES: [RadianceCacheUpdateStage; 5] = [
    RadianceCacheUpdateStage {
        parameter_index: RADIANCE_CACHE_PARAM_MARK,
        stage: RADIANCE_CACHE_STAGE_MARK,
        label: "HybridGiRadianceCacheMarkPass",
    },
    RadianceCacheUpdateStage {
        parameter_index: RADIANCE_CACHE_PARAM_ALLOCATE,
        stage: RADIANCE_CACHE_STAGE_ALLOCATE,
        label: "HybridGiRadianceCacheAllocateTraceTilesPass",
    },
    RadianceCacheUpdateStage {
        parameter_index: RADIANCE_CACHE_PARAM_TRACE,
        stage: RADIANCE_CACHE_STAGE_TRACE,
        label: "HybridGiRadianceCacheTracePass",
    },
    RadianceCacheUpdateStage {
        parameter_index: RADIANCE_CACHE_PARAM_FILTER,
        stage: RADIANCE_CACHE_STAGE_FILTER,
        label: "HybridGiRadianceCacheFilterPass",
    },
    RadianceCacheUpdateStage {
        parameter_index: RADIANCE_CACHE_PARAM_BORDER_MIP,
        stage: RADIANCE_CACHE_STAGE_BORDER_MIP,
        label: "HybridGiRadianceCacheBorderFixupMipPass",
    },
];

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct RadianceCacheDispatchParams {
    update_count: u32,
    consume_count: u32,
    resident_probe_count: u32,
    stage: u32,
}

pub(in crate::hybrid_gi::renderer::gpu_resources::execute_prepare::execute) fn dispatch_radiance_cache(
    device: &wgpu::Device,
    resources: &HybridGiGpuResources,
    state: &RadianceCacheGpuState,
    buffer_uploads: &mut dyn RenderPassBufferUploadSink,
    encoder: &mut wgpu::CommandEncoder,
    buffers: &HybridGiPrepareExecutionBuffers,
    inputs: &HybridGiPrepareExecutionInputs,
) {
    buffer_uploads.write_buffer(
        &state.mark_buffer,
        (RADIANCE_CACHE_DISPATCH_COUNTER_WORD_OFFSET * std::mem::size_of::<u32>()) as u64,
        bytemuck::cast_slice(&[0_u32; RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT]),
    );
    let update_count = inputs
        .radiance_cache_update_inputs
        .len()
        .min(u32::MAX as usize) as u32;
    let consume_count = inputs
        .radiance_cache_consume_inputs
        .len()
        .min(u32::MAX as usize) as u32;
    if update_count == 0 && consume_count == 0 {
        return;
    }

    let resident_probe_count = inputs.resident_probe_inputs.len().min(u32::MAX as usize) as u32;
    if update_count > 0 {
        for update_stage in RADIANCE_CACHE_UPDATE_STAGES {
            buffer_uploads.write_buffer(
                &state.params_buffers[update_stage.parameter_index],
                0,
                bytemuck::bytes_of(&RadianceCacheDispatchParams {
                    update_count,
                    consume_count,
                    resident_probe_count,
                    stage: update_stage.stage,
                }),
            );
            let bind_group = radiance_cache_bind_group(
                device,
                resources,
                state,
                &state.params_buffers[update_stage.parameter_index],
                buffers,
            );
            encode_radiance_cache_pass(
                encoder,
                update_stage.label,
                &resources.radiance_cache_update_pipeline,
                &bind_group,
                update_count,
            );
        }
    }

    if consume_count > 0 {
        buffer_uploads.write_buffer(
            &state.params_buffers[RADIANCE_CACHE_PARAM_CONSUME],
            0,
            bytemuck::bytes_of(&RadianceCacheDispatchParams {
                update_count,
                consume_count,
                resident_probe_count,
                stage: 0,
            }),
        );
        let bind_group = radiance_cache_bind_group(
            device,
            resources,
            state,
            &state.params_buffers[RADIANCE_CACHE_PARAM_CONSUME],
            buffers,
        );
        encode_radiance_cache_pass(
            encoder,
            "HybridGiRadianceCacheScreenProbeConsumePass",
            &resources.radiance_cache_consume_pipeline,
            &bind_group,
            consume_count,
        );
    }
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_radiance_cache_bind_group_layout(
    device: &wgpu::Device,
) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-bind-group-layout"),
        entries: &[
            uniform_layout_entry(0),
            storage_layout_entry(1, true),
            storage_layout_entry(2, true),
            storage_layout_entry(3, false),
            storage_layout_entry(4, false),
            storage_layout_entry(5, false),
            storage_layout_entry(6, false),
            storage_layout_entry(7, false),
            storage_layout_entry(8, false),
        ],
    })
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_radiance_cache_update_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    radiance_cache_pipeline(
        device,
        bind_group_layout,
        "zircon-hybrid-gi-radiance-cache-update-pipeline",
        "cs_update",
    )
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_radiance_cache_consume_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
) -> wgpu::ComputePipeline {
    radiance_cache_pipeline(
        device,
        bind_group_layout,
        "zircon-hybrid-gi-radiance-cache-consume-pipeline",
        "cs_consume",
    )
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_radiance_cache_params_buffers(
    device: &wgpu::Device,
) -> [wgpu::Buffer; 6] {
    std::array::from_fn(|_| {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("zircon-hybrid-gi-radiance-cache-params"),
            size: std::mem::size_of::<RadianceCacheDispatchParams>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    })
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_radiance_cache_storage_buffer(
    device: &wgpu::Device,
    label: &'static str,
) -> wgpu::Buffer {
    create_pod_storage_buffer(
        device,
        label,
        &[GpuRadianceCacheStorageEntry::zeroed();
            HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT],
        wgpu::BufferUsages::STORAGE,
    )
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_radiance_cache_atlas_buffer(
    device: &wgpu::Device,
    label: &'static str,
) -> wgpu::Buffer {
    create_u32_storage_buffer(
        device,
        label,
        &[0; HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT
            * GPU_RADIANCE_CACHE_PROBE_ATLAS_WORD_COUNT],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    )
}

pub(in crate::hybrid_gi::renderer::gpu_resources) fn create_radiance_cache_mark_buffer(
    device: &wgpu::Device,
) -> wgpu::Buffer {
    create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-radiance-cache-marks",
        &[0; RADIANCE_CACHE_MARK_WORD_COUNT],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
    )
}

fn radiance_cache_bind_group(
    device: &wgpu::Device,
    resources: &HybridGiGpuResources,
    state: &RadianceCacheGpuState,
    params_buffer: &wgpu::Buffer,
    buffers: &HybridGiPrepareExecutionBuffers,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-bind-group"),
        layout: &resources.radiance_cache_bind_group_layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: params_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: buffers.radiance_cache_update_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: buffers.radiance_cache_consume_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: state.storage_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: state.mark_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 5,
                resource: state.trace_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 6,
                resource: state.filtered_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 7,
                resource: state.final_atlas_buffer.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 8,
                resource: buffers.resident_probe_buffer.as_entire_binding(),
            },
        ],
    })
}

fn encode_radiance_cache_pass(
    encoder: &mut wgpu::CommandEncoder,
    label: &'static str,
    pipeline: &wgpu::ComputePipeline,
    bind_group: &wgpu::BindGroup,
    item_count: u32,
) {
    let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
        label: Some(label),
        timestamp_writes: None,
    });
    pass.set_pipeline(pipeline);
    pass.set_bind_group(0, bind_group, &[]);
    pass.dispatch_workgroups(radiance_cache_workgroup_count(item_count), 1, 1);
}

fn radiance_cache_workgroup_count(item_count: u32) -> u32 {
    item_count.div_ceil(RADIANCE_CACHE_WORKGROUP_SIZE)
}

fn radiance_cache_pipeline(
    device: &wgpu::Device,
    bind_group_layout: &wgpu::BindGroupLayout,
    label: &'static str,
    entry_point: &'static str,
) -> wgpu::ComputePipeline {
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-shader"),
        source: wgpu::ShaderSource::Wgsl(RADIANCE_CACHE_SHADER_SOURCE.into()),
    });
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-pipeline-layout"),
        bind_group_layouts: &[Some(bind_group_layout)],
        immediate_size: 0,
    });
    device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: Some(label),
        layout: Some(&pipeline_layout),
        module: &shader,
        entry_point: Some(entry_point),
        compilation_options: wgpu::PipelineCompilationOptions::default(),
        cache: None,
    })
}

fn uniform_layout_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::COMPUTE,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: false,
            min_binding_size: None,
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
