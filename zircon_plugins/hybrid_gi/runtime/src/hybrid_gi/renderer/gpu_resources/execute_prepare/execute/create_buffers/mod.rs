mod scene_prepare_depth_samples;
mod scene_prepare_descriptors;
mod scene_prepare_resources;
mod scene_prepare_textures;
mod scene_prepare_trace_tiles;
mod scene_prepare_voxel_samples;
mod surface_cache_depth_hierarchy;

use super::super::super::buffer_helpers::{create_pod_storage_buffer, create_u32_storage_buffer};
use super::hybrid_gi_prepare_execution_buffers::HybridGiPrepareExecutionBuffers;
use super::hybrid_gi_prepare_execution_inputs::HybridGiPrepareExecutionInputs;
use scene_prepare_descriptors::{
    gpu_scene_card_capture_seed_rgb, gpu_scene_persisted_page_card_capture_seed_rgb,
    gpu_scene_prepare_descriptors,
};
use scene_prepare_resources::scene_prepare_resources;

use super::material_capture_source::HybridGiMaterialCaptureSource;

pub(super) fn create_buffers(
    device: &wgpu::Device,
    encoder: &mut wgpu::CommandEncoder,
    streamer: &impl HybridGiMaterialCaptureSource,
    inputs: &HybridGiPrepareExecutionInputs,
    tracing_budget: Option<u32>,
) -> HybridGiPrepareExecutionBuffers {
    let scene_card_capture_seed_rgb =
        gpu_scene_card_capture_seed_rgb(&inputs.scene_card_capture_requests, streamer, inputs);
    let persisted_page_seed_rgb = gpu_scene_persisted_page_card_capture_seed_rgb(
        &inputs.scene_card_capture_requests,
        &inputs.scene_surface_cache_page_contents,
    );
    let scene_prepare_descriptors = gpu_scene_prepare_descriptors(
        &inputs.scene_card_capture_requests,
        &inputs.scene_surface_cache_page_contents,
        &scene_card_capture_seed_rgb,
        &persisted_page_seed_rgb,
        &inputs.scene_voxel_clipmaps,
        &inputs.scene_voxel_cells,
    );
    let scene_prepare_resources =
        scene_prepare_resources(device, encoder, streamer, inputs, tracing_budget);
    let cache_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-cache-buffer",
        bytemuck::cast_slice(&inputs.cache_entries),
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
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
    let scene_prepare_descriptor_buffer = create_pod_storage_buffer(
        device,
        "zircon-hybrid-gi-scene-prepare-descriptors",
        &scene_prepare_descriptors,
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
    let irradiance_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-irradiance-buffer",
        &vec![0_u32; inputs.irradiance_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    let trace_lighting_buffer = create_u32_storage_buffer(
        device,
        "zircon-hybrid-gi-trace-lighting-buffer",
        &vec![0_u32; inputs.trace_lighting_word_count.max(1)],
        wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    );
    HybridGiPrepareExecutionBuffers {
        cache_buffer,
        resident_probe_buffer,
        pending_probe_buffer,
        trace_region_buffer,
        scene_prepare_descriptor_buffer,
        scene_prepare_descriptor_count: scene_prepare_descriptors.len(),
        completed_probe_buffer,
        completed_trace_buffer,
        irradiance_buffer,
        trace_lighting_buffer,
        scene_prepare_resources,
    }
}
