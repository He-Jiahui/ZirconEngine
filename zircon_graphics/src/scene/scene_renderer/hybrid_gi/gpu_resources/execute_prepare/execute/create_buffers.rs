use super::super::super::buffer_helpers::{
    buffer_size_for_words, create_pod_storage_buffer, create_readback_buffer,
    create_u32_storage_buffer,
};
use super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareExecutionBuffers;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;

pub(super) fn create_buffers(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    inputs: &HybridGiPrepareExecutionInputs,
) -> HybridGiPrepareExecutionBuffers {
    let cache_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-cache-buffer",
        bytemuck::cast_slice(&inputs.cache_entries),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let cache_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-cache-readback",
        inputs.cache_word_count,
    );
    encoder.copy_buffer_to_buffer(
        &cache_buffer,
        0,
        &cache_readback,
        0,
        buffer_size_for_words(inputs.cache_word_count),
    );

    let resident_probe_buffer = create_pod_storage_buffer(
        device,
        "zircon-hybrid-gi-resident-probes",
        &inputs.resident_probe_inputs,
        wgpu::BufferUsages::STORAGE,
    );
    let pending_probe_buffer = create_pod_storage_buffer(
        device,
        "zircon-hybrid-gi-pending-probes",
        &inputs.pending_probe_inputs,
        wgpu::BufferUsages::STORAGE,
    );
    let trace_region_buffer = create_pod_storage_buffer(
        device,
        "zircon-hybrid-gi-trace-regions",
        &inputs.trace_region_inputs,
        wgpu::BufferUsages::STORAGE,
    );
    let completed_probe_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-completed-probes",
        &vec![0_u32; inputs.completed_probe_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let completed_trace_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-completed-traces",
        &vec![0_u32; inputs.completed_trace_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let completed_probe_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-completed-probe-readback",
        inputs.completed_probe_word_count.max(1),
    );
    let completed_trace_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-completed-trace-readback",
        inputs.completed_trace_word_count.max(1),
    );
    let irradiance_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-irradiance-buffer",
        &vec![0_u32; inputs.irradiance_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let irradiance_readback = create_readback_buffer(
        device,
        "zircon-hybrid-gi-irradiance-readback",
        inputs.irradiance_word_count.max(1),
    );

    HybridGiPrepareExecutionBuffers {
        cache_readback,
        resident_probe_buffer,
        pending_probe_buffer,
        trace_region_buffer,
        completed_probe_buffer,
        completed_trace_buffer,
        completed_probe_readback,
        completed_trace_readback,
        irradiance_buffer,
        irradiance_readback,
    }
}
