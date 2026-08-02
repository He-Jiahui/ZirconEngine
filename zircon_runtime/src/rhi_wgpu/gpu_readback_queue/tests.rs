use super::staging_ring::{
    align_readback_offset, StagingCapacityPolicy, LOW_UTILIZATION_FRAME_LIMIT,
    MIN_STAGING_CAPACITY, READBACK_FRAME_SLOTS, READBACK_OFFSET_ALIGNMENT,
};
use super::GpuReadbackQueue;
use std::sync::{Arc, Mutex};

#[test]
fn texture_readback_layout_unpads_rows_without_retaining_staging_padding() {
    let layout = super::queue::texture_rgba_readback_layout(65, 2).unwrap();
    assert_eq!(layout.unpadded_bytes_per_row, 260);
    assert_eq!(layout.padded_bytes_per_row, 512);
    assert_eq!(layout.staging_byte_len, 1024);

    let mut mapped = vec![0_u8; layout.staging_byte_len as usize];
    mapped[..260].fill(7);
    mapped[512..772].fill(9);

    let rgba = layout.unpack_rgba(&mapped).unwrap();
    assert_eq!(rgba.len(), 520);
    assert!(rgba[..260].iter().all(|byte| *byte == 7));
    assert!(rgba[260..].iter().all(|byte| *byte == 9));
}

#[test]
fn readback_ring_grows_to_fit_frame_requests() {
    let mut policy = StagingCapacityPolicy::default();
    assert_eq!(policy.capacity_for_frame(1), Some(MIN_STAGING_CAPACITY));
    assert_eq!(policy.capacity(), MIN_STAGING_CAPACITY);

    let required = MIN_STAGING_CAPACITY + 1;
    assert_eq!(
        policy.capacity_for_frame(required),
        Some(MIN_STAGING_CAPACITY * 2)
    );
    assert!(policy.capacity() >= required);
}

#[test]
fn readback_ring_shrinks_only_after_sustained_low_utilization() {
    let mut policy = StagingCapacityPolicy::default();
    policy.capacity_for_frame(MIN_STAGING_CAPACITY * 4);

    for _ in 1..LOW_UTILIZATION_FRAME_LIMIT {
        assert_eq!(policy.capacity_for_frame(READBACK_OFFSET_ALIGNMENT), None);
    }
    assert_eq!(
        policy.capacity_for_frame(READBACK_OFFSET_ALIGNMENT),
        Some(MIN_STAGING_CAPACITY * 2)
    );
}

#[test]
fn readback_ring_shrinks_after_sustained_empty_frames() {
    let mut policy = StagingCapacityPolicy::default();
    policy.capacity_for_frame(MIN_STAGING_CAPACITY * 4);

    for _ in 1..LOW_UTILIZATION_FRAME_LIMIT {
        assert_eq!(policy.capacity_for_frame(0), None);
    }
    assert_eq!(policy.capacity_for_frame(0), Some(MIN_STAGING_CAPACITY * 2));
}

#[test]
fn readback_ring_shrink_delay_counts_global_frames_across_slot_reuse() {
    let mut policy = StagingCapacityPolicy::default();
    policy.capacity_for_frame(MIN_STAGING_CAPACITY * 4);

    for _ in 1..(LOW_UTILIZATION_FRAME_LIMIT / READBACK_FRAME_SLOTS as u16) {
        assert_eq!(
            policy.capacity_for_elapsed_frames(READBACK_OFFSET_ALIGNMENT, 3),
            None
        );
    }
    assert_eq!(
        policy.capacity_for_elapsed_frames(READBACK_OFFSET_ALIGNMENT, 3),
        Some(MIN_STAGING_CAPACITY * 2)
    );
}

#[test]
fn readback_requests_use_256_byte_offsets_and_three_frame_slots() {
    assert_eq!(align_readback_offset(0), Some(0));
    assert_eq!(align_readback_offset(1), Some(READBACK_OFFSET_ALIGNMENT));
    assert_eq!(align_readback_offset(257), Some(512));
    assert_eq!(READBACK_FRAME_SLOTS, 3);
}

#[test]
fn readback_empty_queue_keeps_zero_staging_capacity() {
    let mut policy = StagingCapacityPolicy::default();
    assert_eq!(policy.capacity_for_frame(0), None);
    assert_eq!(policy.capacity(), 0);
}

#[test]
fn readback_callback_fires_after_n_frame_delay() {
    let Some((device, submission_queue)) = offscreen_test_device() else {
        return;
    };
    let source = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback-queue-source"),
        size: 8,
        usage: wgpu::BufferUsages::COPY_SRC | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    });
    let expected = 0x0102_0304_0506_0708_u64.to_le_bytes();
    submission_queue.write_buffer(&source, 0, &expected);

    let delivered = Arc::new(Mutex::new(None));
    let callback_delivered = Arc::clone(&delivered);
    let mut readback_queue = GpuReadbackQueue::new(&device);
    readback_queue.prepare_frame(&device, 0).unwrap();
    readback_queue
        .request_readback_external(
            "test-source",
            &source,
            0..8,
            Box::new(move |result| {
                *callback_delivered.lock().unwrap() = Some(result.unwrap().to_vec());
            }),
        )
        .unwrap();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-queue-test-encoder"),
    });
    readback_queue.encode_copies(&mut encoder, 0).unwrap();
    submission_queue.submit([encoder.finish()]);
    readback_queue.begin_map(0).unwrap();

    assert_eq!(*delivered.lock().unwrap(), None);
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    let stats = readback_queue.poll_completed(&device);
    assert_eq!(*delivered.lock().unwrap(), Some(expected.to_vec()));
    assert_eq!(stats.completed_request_count, 1);
    assert_eq!(stats.completed_bytes, 8);
    assert_eq!(stats.in_flight_count, 0);
    assert_eq!(readback_queue.stats(), stats);
}

#[test]
fn texture_readback_callback_delivers_rgba_after_async_map_completion() {
    let Some((device, submission_queue)) = offscreen_test_device() else {
        return;
    };
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("zircon-readback-queue-texture-source"),
        size: wgpu::Extent3d {
            width: 64,
            height: 2,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8Unorm,
        usage: wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let expected = (0..(64 * 2 * 4))
        .map(|value| value as u8)
        .collect::<Vec<_>>();
    submission_queue.write_texture(
        texture.as_image_copy(),
        &expected,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(64 * 4),
            rows_per_image: Some(2),
        },
        wgpu::Extent3d {
            width: 64,
            height: 2,
            depth_or_array_layers: 1,
        },
    );

    let delivered = Arc::new(Mutex::new(None));
    let callback_delivered = Arc::clone(&delivered);
    let mut readback_queue = GpuReadbackQueue::new(&device);
    readback_queue.prepare_frame(&device, 1).unwrap();
    readback_queue
        .request_texture_rgba(
            "test-texture",
            &texture,
            64,
            2,
            Box::new(move |result| {
                *callback_delivered.lock().unwrap() = Some(result.unwrap());
            }),
        )
        .unwrap();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-queue-texture-encoder"),
    });
    readback_queue.encode_copies(&mut encoder, 1).unwrap();
    submission_queue.submit([encoder.finish()]);
    readback_queue.begin_map(1).unwrap();

    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    readback_queue.poll_completed(&device);

    assert_eq!(*delivered.lock().unwrap(), Some(expected));
}

#[test]
fn readback_no_private_map_async_source_scan() {
    let timer = include_str!("../gpu_pass_timer.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();
    let queue = include_str!("queue.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(!timer.contains("map_async"));
    assert!(!timer.contains("std::sync::mpsc"));
    assert!(!timer.contains("readback_queue: GpuReadbackQueue"));
    assert!(timer.contains("readback_queue: &mut GpuReadbackQueue"));
    assert_eq!(queue.matches(".map_async(").count(), 1);

    let ordinary_consumers = [
        include_str!(
            "../../graphics/scene/scene_renderer/hzb/hzb_occlusion_culler.rs"
        ),
        include_str!(
            "../../graphics/scene/scene_renderer/mesh/mesh_pass/indirect_draw_execution.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_readback/decode/read_buffer_u32s.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_readback/pending_readback/hybrid_gi_gpu_readback_future.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/decode/read_buffer_u32s.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_readback/pending_readback/collect.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/particles/runtime/src/render/runtime_prepare.rs"
        ),
    ];
    for source in ordinary_consumers {
        let production_source = source.split("\n#[cfg(test)]").next().unwrap_or_default();
        assert!(!production_source.contains("map_async"));
        assert!(!production_source.contains("wait_indefinitely"));
    }

    let particle_backend =
        include_str!("../../../../zircon_plugins/particles/runtime/src/render/gpu/backend.rs");
    assert!(!particle_backend.contains("map_async"));

    let direct_storage_readback_sources = [
        include_str!(
            "../../../../zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/execute.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/create_buffers/mod.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/hybrid_gi/runtime/src/hybrid_gi/renderer/gpu_resources/execute_prepare/execute/create_buffers/scene_prepare_trace_tiles.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/execute_prepare/execute/execute.rs"
        ),
        include_str!(
            "../../../../zircon_plugins/virtual_geometry/runtime/src/virtual_geometry/renderer/gpu_resources/execute_prepare/execute/create_buffers.rs"
        ),
    ];
    for source in direct_storage_readback_sources {
        assert!(!source.contains("copy_readbacks"));
        assert!(!source.contains("create_readback_buffer"));
    }
}

#[test]
fn readback_queue_production_paths_are_panic_free() {
    let queue = include_str!("queue.rs")
        .split("\n#[cfg(test)]")
        .next()
        .unwrap_or_default();

    assert!(!queue.contains(".expect("));
    assert!(!queue.contains(".unwrap("));
    assert!(!queue.contains("panic!("));
    assert!(!queue.contains("#[allow(dead_code)]"));
}

#[test]
fn readback_request_requires_a_prepared_frame_and_aligned_source() {
    let Some((device, _)) = offscreen_test_device() else {
        return;
    };
    let source = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback-queue-validation-source"),
        size: 16,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let mut queue = GpuReadbackQueue::new(&device);
    assert!(queue
        .request_readback_external("inactive", &source, 0..4, Box::new(|_| {}))
        .is_err());

    queue.prepare_frame(&device, 7).unwrap();
    assert!(queue
        .request_readback_external("unaligned", &source, 2..6, Box::new(|_| {}))
        .is_err());
}

#[test]
fn readback_slot_reuse_is_refused_without_waiting_for_map_completion() {
    let Some((device, _)) = offscreen_test_device() else {
        return;
    };
    let mut queue = GpuReadbackQueue::new(&device);
    let _completion_sender = queue.inject_in_flight_slot_for_tests(0);

    let error = queue.prepare_frame(&device, 3).unwrap_err();

    assert!(matches!(
        error,
        super::ReadbackError::SlotReuseIncomplete { slot_index: 0 }
    ));
    assert_eq!(queue.stats().slot_reuse_rejection_count, 1);
}

#[test]
fn readback_invalid_map_or_abort_keeps_the_active_frame_owned() {
    let Some((device, _)) = offscreen_test_device() else {
        return;
    };
    let mut queue = GpuReadbackQueue::new(&device);
    queue.prepare_frame(&device, 9).unwrap();

    assert!(queue.begin_map(9).is_err());
    assert!(queue.begin_map(10).is_err());
    queue.abort_frame(10);
    assert!(matches!(
        queue.prepare_frame(&device, 11),
        Err(super::ReadbackError::FrameAlreadyActive {
            active: 9,
            requested: 11
        })
    ));

    queue.abort_frame(9);
    assert!(queue.prepare_frame(&device, 12).is_ok());
}

#[test]
fn readback_abort_completes_pending_callbacks_with_an_error() {
    let Some((device, _)) = offscreen_test_device() else {
        return;
    };
    let source = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback-queue-abort-source"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let delivered = Arc::new(Mutex::new(None));
    let callback_delivered = Arc::clone(&delivered);
    let mut queue = GpuReadbackQueue::new(&device);
    queue.prepare_frame(&device, 17).unwrap();
    queue
        .request_readback_external(
            "test-abort",
            &source,
            0..4,
            Box::new(move |result| {
                *callback_delivered.lock().unwrap() = Some(matches!(
                    result,
                    Err(super::ReadbackError::FrameAborted { frame_index: 17 })
                ));
            }),
        )
        .unwrap();

    queue.abort_frame(17);

    assert_eq!(*delivered.lock().unwrap(), Some(true));
}

#[test]
fn readback_layout_failure_preserves_callbacks_for_abort() {
    let Some((device, _)) = offscreen_test_device() else {
        return;
    };
    let source = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-readback-queue-overflow-source"),
        size: 4,
        usage: wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false,
    });
    let delivered = Arc::new(Mutex::new(None));
    let callback_delivered = Arc::clone(&delivered);
    let mut queue = GpuReadbackQueue::new(&device);
    queue.prepare_frame(&device, 23).unwrap();
    queue
        .request_readback_external(
            "test-layout-overflow",
            &source,
            0..u64::MAX - 3,
            Box::new(move |result| {
                *callback_delivered.lock().unwrap() = Some(matches!(
                    result,
                    Err(super::ReadbackError::FrameAborted { frame_index: 23 })
                ));
            }),
        )
        .unwrap();
    let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
        label: Some("zircon-readback-queue-overflow-encoder"),
    });

    assert!(matches!(
        queue.encode_copies(&mut encoder, 23),
        Err(super::ReadbackError::CapacityOverflow)
    ));
    assert_eq!(*delivered.lock().unwrap(), None);

    queue.abort_frame(23);

    assert_eq!(*delivered.lock().unwrap(), Some(true));
}

fn offscreen_test_device() -> Option<(wgpu::Device, wgpu::Queue)> {
    let instance = wgpu::Instance::new(wgpu::InstanceDescriptor::new_without_display_handle());
    let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
        power_preference: wgpu::PowerPreference::LowPower,
        compatible_surface: None,
        force_fallback_adapter: false,
    }))
    .ok()?;
    pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
        label: Some("zircon-readback-queue-test-device"),
        required_features: wgpu::Features::empty(),
        required_limits: wgpu::Limits::default(),
        memory_hints: wgpu::MemoryHints::Performance,
        trace: wgpu::Trace::Off,
        experimental_features: wgpu::ExperimentalFeatures::disabled(),
    }))
    .ok()
}
