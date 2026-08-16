use std::sync::mpsc;

use wgpu::util::DeviceExt;

use super::super::super::super::super::{
    gpu_pending_probe_input::GpuPendingProbeInput,
    gpu_radiance_cache_consume_input::GpuRadianceCacheConsumeInput,
    gpu_radiance_cache_update_input::GpuRadianceCacheUpdateInput,
    gpu_resident_probe_input::{
        GpuResidentProbeInput, GPU_RESIDENT_PROBE_INPUT_WORD_COUNT,
        GPU_RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET,
    },
    gpu_trace_region_input::GpuTraceRegionInput,
};
use super::*;

const HYBRID_GI_STORAGE_BUFFER_BINDING_COUNT: u32 = 9;

#[test]
fn radiance_cache_workgroup_count_covers_each_nonempty_input_range() {
    assert_eq!(radiance_cache_workgroup_count(0), 0);
    assert_eq!(radiance_cache_workgroup_count(1), 1);
    assert_eq!(
        radiance_cache_workgroup_count(RADIANCE_CACHE_WORKGROUP_SIZE),
        1
    );
    assert_eq!(
        radiance_cache_workgroup_count(RADIANCE_CACHE_WORKGROUP_SIZE + 1),
        2
    );
}

#[test]
fn radiance_cache_update_stages_preserve_the_visibility_order() {
    assert_eq!(
        RADIANCE_CACHE_UPDATE_STAGES.map(|stage| stage.stage),
        [
            RADIANCE_CACHE_STAGE_MARK,
            RADIANCE_CACHE_STAGE_ALLOCATE,
            RADIANCE_CACHE_STAGE_TRACE,
            RADIANCE_CACHE_STAGE_FILTER,
            RADIANCE_CACHE_STAGE_BORDER_MIP,
        ]
    );
    assert_eq!(
        RADIANCE_CACHE_UPDATE_STAGES.map(|stage| stage.label),
        [
            "HybridGiRadianceCacheMarkPass",
            "HybridGiRadianceCacheAllocateTraceTilesPass",
            "HybridGiRadianceCacheTracePass",
            "HybridGiRadianceCacheFilterPass",
            "HybridGiRadianceCacheBorderFixupMipPass",
        ]
    );
}

#[test]
fn radiance_cache_shader_abi_constants_match_the_rust_storage_contract() {
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains(&format!(
        "const RADIANCE_CACHE_SLOT_CAPACITY: u32 = {}u;",
        HYBRID_GI_RADIANCE_CACHE_MAX_RESIDENT_PROBE_COUNT
    )));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains(&format!(
        "const RESIDENT_PROBE_WORD_COUNT: u32 = {}u;",
        super::super::super::super::super::gpu_resident_probe_input::GPU_RESIDENT_PROBE_INPUT_WORD_COUNT
    )));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains(&format!(
            "const RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET: u32 = {}u;",
            super::super::super::super::super::gpu_resident_probe_input::GPU_RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET
        )));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains(&format!(
        "array<u32, {}>",
        HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT
    )));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains(&format!(
        "const RADIANCE_CACHE_PROBE_TILE_EXTENT: u32 = {}u;",
        GPU_RADIANCE_CACHE_PROBE_TILE_EXTENT
    )));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains(&format!(
            "const RADIANCE_CACHE_PROBE_MIP1_WORD_COUNT: u32 = {}u;",
            super::super::super::super::super::gpu_radiance_cache_storage_entry::GPU_RADIANCE_CACHE_PROBE_MIP1_WORD_COUNT
        )));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains(&format!(
            "const RADIANCE_CACHE_PROBE_MIP2_WORD_COUNT: u32 = {}u;",
            super::super::super::super::super::gpu_radiance_cache_storage_entry::GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_COUNT
        )));
    assert!(RADIANCE_CACHE_SHADER_SOURCE
        .contains("final_atlas[entry.atlas_base + RADIANCE_CACHE_PROBE_MIP2_OFFSET]"));
    assert!(RADIANCE_CACHE_SHADER_SOURCE
        .contains("RADIANCE_CACHE_PROBE_MIP2_OFFSET + RADIANCE_CACHE_PROBE_MIP2_WORD_COUNT"));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains("fn average_cross_rgba8("));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains("RADIANCE_CACHE_PROBE_MIP2_OFFSET"));
    assert!(
        RADIANCE_CACHE_SHADER_SOURCE.contains("if (update_input.reuse_committed_radiance != 0u)")
    );
    assert!(RADIANCE_CACHE_SHADER_SOURCE
        .contains("let resident_base = consume.resident_probe_index * RESIDENT_PROBE_WORD_COUNT;"));
    assert!(RADIANCE_CACHE_SHADER_SOURCE
        .contains("var<storage, read_write> marked_slots: array<atomic<u32>>;"));
    assert!(RADIANCE_CACHE_SHADER_SOURCE.contains("atomicAdd("));
    assert!(!RADIANCE_CACHE_SHADER_SOURCE.contains("var resident_index = 0u;"));
}

#[test]
fn radiance_cache_shader_commits_final_mip_before_screen_probe_consume() {
    let Some((device, queue)) = test_device() else {
        eprintln!("skipping radiance-cache Wgpu test because no adapter is available");
        return;
    };

    let resources = HybridGiGpuResources::new(&device);
    let state = RadianceCacheGpuState::new(&device);
    let update = GpuRadianceCacheUpdateInput {
        slot: 3,
        generation_low: 11,
        generation_high: 0,
        radiance_confidence: u32::from_le_bytes([40, 50, 60, 200]),
        reuse_committed_radiance: 0,
    };
    let consume = GpuRadianceCacheConsumeInput {
        probe_id: 55,
        generation_low: 11,
        generation_high: 0,
        resident_probe_index: 0,
        slots: [3; HYBRID_GI_RADIANCE_CACHE_INTERPOLATION_CORNER_COUNT],
        weights_q16: [u32::from(u16::MAX), 0, 0, 0, 0, 0, 0, 0],
    };
    let mut resident_probe = GpuResidentProbeInput::zeroed();
    resident_probe.probe_id = 55;
    let inputs = HybridGiPrepareExecutionInputs {
        resident_probe_inputs: vec![resident_probe],
        radiance_cache_update_inputs: vec![update],
        radiance_cache_consume_inputs: vec![consume],
        ..Default::default()
    };
    let buffers = test_execution_buffers(&device, &inputs);
    let resident_readback = create_readback_buffer(
        &device,
        GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize,
        "zircon-hybrid-gi-radiance-cache-test-resident-readback",
    );
    let final_mip_readback = create_readback_buffer(
        &device,
        1,
        "zircon-hybrid-gi-radiance-cache-test-final-mip-readback",
    );
    let dispatch_counter_readback = create_readback_buffer(
        &device,
        RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT,
        "zircon-hybrid-gi-radiance-cache-test-dispatch-counter-readback",
    );

    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-test-encoder"),
    });
    dispatch_radiance_cache(
        &device,
        &resources,
        &state,
        &queue,
        &mut encoder,
        &buffers,
        &inputs,
    );
    encoder.copy_buffer_to_buffer(
        &state.final_atlas_buffer,
        ((3 * GPU_RADIANCE_CACHE_PROBE_ATLAS_WORD_COUNT
            + GPU_RADIANCE_CACHE_PROBE_MIP2_WORD_OFFSET)
            * std::mem::size_of::<u32>()) as u64,
        &final_mip_readback,
        0,
        std::mem::size_of::<u32>() as u64,
    );
    encoder.copy_buffer_to_buffer(
        &buffers.resident_probe_buffer,
        0,
        &resident_readback,
        0,
        (GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize * std::mem::size_of::<u32>()) as u64,
    );
    encoder.copy_buffer_to_buffer(
        &state.mark_buffer,
        (RADIANCE_CACHE_DISPATCH_COUNTER_WORD_OFFSET * std::mem::size_of::<u32>()) as u64,
        &dispatch_counter_readback,
        0,
        (RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(encoder.finish()));

    assert_eq!(
        readback_u32s(
            &device,
            &dispatch_counter_readback,
            RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT,
        ),
        vec![1, 1, 1, 1, 1, 1],
        "the shader must author five update counts and one committed consume count"
    );
    assert_eq!(
        readback_u32s(&device, &final_mip_readback, 1),
        vec![u32::from_le_bytes([40, 50, 60, 200])]
    );
    let resident_words = readback_u32s(
        &device,
        &resident_readback,
        GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize,
    );
    assert_eq!(
        resident_words[GPU_RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET as usize],
        u32::from_le_bytes([40, 50, 60, 0])
    );

    let stable_inputs = HybridGiPrepareExecutionInputs {
        resident_probe_inputs: vec![test_resident_probe(55, 0)],
        radiance_cache_consume_inputs: vec![consume],
        ..Default::default()
    };
    let stable_buffers = test_execution_buffers(&device, &stable_inputs);
    let stable_readback = create_readback_buffer(
        &device,
        GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize,
        "zircon-hybrid-gi-radiance-cache-test-stable-readback",
    );
    let stable_dispatch_counter_readback = create_readback_buffer(
        &device,
        RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT,
        "zircon-hybrid-gi-radiance-cache-test-stable-dispatch-counter-readback",
    );
    let mut stable_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-test-stable-encoder"),
    });
    dispatch_radiance_cache(
        &device,
        &resources,
        &state,
        &queue,
        &mut stable_encoder,
        &stable_buffers,
        &stable_inputs,
    );
    stable_encoder.copy_buffer_to_buffer(
        &stable_buffers.resident_probe_buffer,
        0,
        &stable_readback,
        0,
        (GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize * std::mem::size_of::<u32>()) as u64,
    );
    stable_encoder.copy_buffer_to_buffer(
        &state.mark_buffer,
        (RADIANCE_CACHE_DISPATCH_COUNTER_WORD_OFFSET * std::mem::size_of::<u32>()) as u64,
        &stable_dispatch_counter_readback,
        0,
        (RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(stable_encoder.finish()));
    assert_eq!(
        readback_u32s(
            &device,
            &stable_dispatch_counter_readback,
            RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT,
        ),
        vec![0, 0, 0, 0, 0, 1],
        "stable frames must skip RC updates while committing final-atlas consumption"
    );
    assert_eq!(
        readback_u32s(
            &device,
            &stable_readback,
            GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize,
        )[GPU_RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET as usize],
        u32::from_le_bytes([40, 50, 60, 0]),
        "stable frames must consume the persistent final atlas without a new update upload"
    );

    let propagated_consume = GpuRadianceCacheConsumeInput {
        generation_low: 12,
        ..consume
    };
    let propagated_inputs = HybridGiPrepareExecutionInputs {
        resident_probe_inputs: vec![test_resident_probe(55, 0)],
        radiance_cache_update_inputs: vec![GpuRadianceCacheUpdateInput {
            generation_low: 12,
            radiance_confidence: u32::from_le_bytes([90, 100, 110, 220]),
            reuse_committed_radiance: 1,
            ..update
        }],
        radiance_cache_consume_inputs: vec![propagated_consume],
        ..Default::default()
    };
    let propagated_buffers = test_execution_buffers(&device, &propagated_inputs);
    let propagated_readback = create_readback_buffer(
        &device,
        GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize,
        "zircon-hybrid-gi-radiance-cache-test-propagated-readback",
    );
    let mut propagated_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-test-propagated-encoder"),
    });
    dispatch_radiance_cache(
        &device,
        &resources,
        &state,
        &queue,
        &mut propagated_encoder,
        &propagated_buffers,
        &propagated_inputs,
    );
    propagated_encoder.copy_buffer_to_buffer(
        &propagated_buffers.resident_probe_buffer,
        0,
        &propagated_readback,
        0,
        (GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(propagated_encoder.finish()));
    assert_eq!(
        readback_u32s(
            &device,
            &propagated_readback,
            GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize,
        )[GPU_RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET as usize],
        u32::from_le_bytes([40, 50, 60, 0]),
        "scroll propagation must advance metadata while retaining the committed atlas sample"
    );

    let stale_consume = GpuRadianceCacheConsumeInput {
        generation_low: 13,
        ..consume
    };
    let stale_inputs = HybridGiPrepareExecutionInputs {
        resident_probe_inputs: vec![test_resident_probe(55, u32::from_le_bytes([5, 6, 7, 0]))],
        radiance_cache_consume_inputs: vec![stale_consume],
        ..Default::default()
    };
    let stale_buffers = test_execution_buffers(&device, &stale_inputs);
    let stale_readback = create_readback_buffer(
        &device,
        GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize,
        "zircon-hybrid-gi-radiance-cache-test-stale-readback",
    );
    let stale_dispatch_counter_readback = create_readback_buffer(
        &device,
        RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT,
        "zircon-hybrid-gi-radiance-cache-test-stale-dispatch-counter-readback",
    );
    let mut stale_encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-test-stale-encoder"),
    });
    dispatch_radiance_cache(
        &device,
        &resources,
        &state,
        &queue,
        &mut stale_encoder,
        &stale_buffers,
        &stale_inputs,
    );
    stale_encoder.copy_buffer_to_buffer(
        &stale_buffers.resident_probe_buffer,
        0,
        &stale_readback,
        0,
        (GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize * std::mem::size_of::<u32>()) as u64,
    );
    stale_encoder.copy_buffer_to_buffer(
        &state.mark_buffer,
        (RADIANCE_CACHE_DISPATCH_COUNTER_WORD_OFFSET * std::mem::size_of::<u32>()) as u64,
        &stale_dispatch_counter_readback,
        0,
        (RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT * std::mem::size_of::<u32>()) as u64,
    );
    queue.submit(std::iter::once(stale_encoder.finish()));
    assert_eq!(
        readback_u32s(
            &device,
            &stale_dispatch_counter_readback,
            RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT,
        ),
        vec![0; RADIANCE_CACHE_DISPATCH_COUNTER_WORD_COUNT],
        "consume evidence must count committed atlas reuse, not dispatched invalid work"
    );
    assert_eq!(
        readback_u32s(
            &device,
            &stale_readback,
            GPU_RESIDENT_PROBE_INPUT_WORD_COUNT as usize,
        )[GPU_RESIDENT_PROBE_PREVIOUS_IRRADIANCE_WORD_OFFSET as usize],
        u32::from_le_bytes([5, 6, 7, 0]),
        "a generation mismatch must preserve the deterministic resident fallback"
    );
}

fn test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let mut descriptor = wgpu::InstanceDescriptor::new_without_display_handle();
    descriptor.backends = wgpu::Backends::PRIMARY;
    let instance = wgpu::Instance::new(descriptor);
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    if adapter.limits().max_storage_buffers_per_shader_stage
        < HYBRID_GI_STORAGE_BUFFER_BINDING_COUNT
    {
        return None;
    }
    let required_limits = wgpu::Limits {
        max_storage_buffers_per_shader_stage: HYBRID_GI_STORAGE_BUFFER_BINDING_COUNT,
        ..wgpu::Limits::default()
    };
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-hybrid-gi-radiance-cache-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits,
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}

fn test_execution_buffers(
    device: &wgpu::Device,
    inputs: &HybridGiPrepareExecutionInputs,
) -> HybridGiPrepareExecutionBuffers {
    HybridGiPrepareExecutionBuffers {
        cache_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-cache",
            &[0_u32],
        ),
        resident_probe_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-resident",
            &inputs.resident_probe_inputs,
        ),
        pending_probe_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-pending",
            &[GpuPendingProbeInput::zeroed()],
        ),
        radiance_cache_update_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-updates",
            &inputs.radiance_cache_update_inputs,
        ),
        radiance_cache_consume_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-consumes",
            &inputs.radiance_cache_consume_inputs,
        ),
        trace_region_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-trace-regions",
            &[GpuTraceRegionInput::zeroed()],
        ),
        scene_prepare_descriptor_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-scene-prepare-descriptors",
            &[[0_u32; 12]],
        ),
        scene_prepare_descriptor_count: 0,
        voxel_cell_descriptor_offset: 0,
        voxel_cell_descriptor_count: 0,
        voxel_cell_lookup_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-empty-voxel-cell-lookup",
            &[u32::MAX],
        ),
        voxel_cell_lookup_complete: false,
        voxel_cell_lookup_clipmap_count: 0,
        completed_probe_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-completed-probes",
            &[0_u32],
        ),
        completed_trace_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-completed-traces",
            &[0_u32],
        ),
        irradiance_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-irradiance",
            &[0_u32],
        ),
        trace_lighting_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-trace-lighting",
            &[0_u32],
        ),
        trace_diagnostic_buffer: create_storage_buffer(
            device,
            "zircon-hybrid-gi-radiance-cache-test-trace-diagnostics",
            &[0_u32],
        ),
        scene_prepare_resources: None,
    }
}

fn test_resident_probe(probe_id: u32, previous_irradiance_rgb: u32) -> GpuResidentProbeInput {
    let mut probe = GpuResidentProbeInput::zeroed();
    probe.probe_id = probe_id;
    probe.previous_irradiance_rgb = previous_irradiance_rgb;
    probe
}

fn create_storage_buffer<T: Pod + Zeroable>(
    device: &wgpu::Device,
    label: &'static str,
    contents: &[T],
) -> wgpu::Buffer {
    if contents.is_empty() {
        return device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some(label),
            contents: bytemuck::bytes_of(&T::zeroed()),
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
        });
    }

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some(label),
        contents: bytemuck::cast_slice(contents),
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_SRC,
    })
}

fn create_readback_buffer(
    device: &wgpu::Device,
    word_count: usize,
    label: &'static str,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: (word_count * std::mem::size_of::<u32>()) as u64,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    })
}

fn readback_u32s(device: &wgpu::Device, buffer: &wgpu::Buffer, word_count: usize) -> Vec<u32> {
    let slice = buffer.slice(..(word_count * std::mem::size_of::<u32>()) as u64);
    let (sender, receiver) = mpsc::channel();
    slice.map_async(wgpu::MapMode::Read, move |result| {
        sender.send(result).ok();
    });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let mapped = slice.get_mapped_range();
    let data = bytemuck::cast_slice(&mapped[..]).to_vec();
    drop(mapped);
    buffer.unmap();
    data
}
