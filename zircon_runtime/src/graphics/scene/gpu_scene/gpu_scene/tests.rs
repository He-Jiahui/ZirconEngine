use std::sync::Arc;

use super::*;
use crate::graphics::backend::{BufferByteReadback, read_buffer_bytes};
use crate::graphics::scene::gpu_scene::{
    GPU_PRIMITIVE_FLAG_VISIBLE, GPU_SCENE_INVALID_PAYLOAD_SLOT,
};

const TEST_STABLE_INSTANCE_KEY: u64 = 0x1000_0001;
const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;
const TEST_STAGING_ENTRY_COUNT: u64 = 2_048;

#[test]
fn render_gpu_scene_static_scene_second_frame_uploads_zero_bytes() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);

    sync_test_entry(&backend.device, &mut scene, 1, 0.0);
    let first_report = scene
        .flush_updates(&backend)
        .expect("first GPU scene upload should be accepted");
    assert!(first_report.uploaded_bytes > 0);

    sync_test_entry(&backend.device, &mut scene, 1, 0.0);
    let second_report = scene
        .flush_updates(&backend)
        .expect("empty GPU scene upload should be accepted");

    assert_eq!(
        second_report.upload_path,
        GpuSceneUploadPath::DirectQueueWrite
    );
    assert_eq!(second_report.upload_path.label(), "direct_queue_write");
    assert_eq!(second_report.uploaded_bytes, 0);
    assert_eq!(second_report.primitive_upload_range_count, 0);
    assert_eq!(second_report.instance_upload_range_count, 0);
    assert_eq!(scene.stats().dirty_entry_count, 0);
}

#[test]
fn render_gpu_scene_single_moving_entity_uploads_only_its_entry() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);

    sync_test_entry(&backend.device, &mut scene, 1, 0.0);
    scene
        .flush_updates(&backend)
        .expect("initial GPU scene upload should be accepted");

    sync_test_entry(&backend.device, &mut scene, 2, 5.0);
    let moving_report = scene
        .flush_updates(&backend)
        .expect("moving GPU scene upload should be accepted");

    assert_eq!(
        moving_report.upload_path,
        GpuSceneUploadPath::DirectQueueWrite
    );
    assert_eq!(
        moving_report.uploaded_bytes,
        GPU_INSTANCE_DATA_STRIDE as u64
    );
    assert_eq!(moving_report.primitive_upload_range_count, 0);
    assert_eq!(moving_report.instance_upload_range_count, 1);
}

#[test]
fn render_gpu_scene_buffered_write_is_visible_after_the_frame_submission() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);

    sync_test_entry(&backend.device, &mut scene, 1, 7.0);
    let report = scene
        .flush_updates(&backend)
        .expect("GPU scene buffer batch should be accepted");
    assert_eq!(report.upload_path, GpuSceneUploadPath::DirectQueueWrite);
    submit_empty_graphics_frame(&backend);

    let uploaded_bytes = read_buffer_bytes(
        &backend.device,
        &backend.queue,
        scene.instance_buffer(),
        BufferByteReadback {
            source_offset: 0,
            byte_len: GPU_INSTANCE_DATA_STRIDE as u64,
            label: "zircon-test-gpu-scene-buffered-write-readback",
        },
    )
    .expect("buffered GPU scene instance write should be readable after submission");
    let uploaded_instances = bytemuck::cast_slice::<u8, GpuInstanceData>(&uploaded_bytes);

    assert_eq!(uploaded_instances.len(), 1);
    assert_eq!(uploaded_instances[0].world_from_local[3][0], 7.0);
}

#[test]
fn render_gpu_scene_light_buffer_grows_and_skips_unchanged_uploads() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let lights = vec![test_light_data(1), test_light_data(2), test_light_data(3)];

    scene.write_lights(&backend.device, &lights);
    let first_report = scene
        .flush_updates(&backend)
        .expect("initial light upload should be accepted");

    assert_eq!(scene.stats().light_count, 3);
    assert!(scene.stats().light_capacity >= 4);
    assert_eq!(first_report.light_upload_range_count, 1);
    assert_eq!(
        first_report.uploaded_bytes,
        (lights.len() * GpuLightData::STRIDE) as u64
    );

    scene.write_lights(&backend.device, &lights);
    let unchanged_report = scene
        .flush_updates(&backend)
        .expect("unchanged light upload should be accepted");

    assert_eq!(unchanged_report.uploaded_bytes, 0);
    assert_eq!(unchanged_report.light_upload_range_count, 0);
}

#[test]
fn render_gpu_scene_uploads_only_changed_light_rows() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let lights = vec![test_light_data(1), test_light_data(2)];
    scene.write_lights(&backend.device, &lights);
    scene
        .flush_updates(&backend)
        .expect("initial light upload should be accepted");

    let changed_lights = vec![lights[0], test_light_data(3)];
    scene.write_lights(&backend.device, &changed_lights);
    let report = scene
        .flush_updates(&backend)
        .expect("changed light upload should be accepted");

    assert_eq!(report.light_upload_range_count, 1);
    assert_eq!(report.uploaded_bytes, GpuLightData::STRIDE as u64);
}

#[test]
fn render_gpu_scene_large_frame_upload_uses_persistent_staging_ring() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let mut encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-test-gpu-scene-staging-upload"),
        });

    for stable_instance_key in 0..TEST_STAGING_ENTRY_COUNT {
        let entry = scene.register(&backend.device, stable_instance_key, 1);
        scene.write_primitive(entry, test_primitive_data());
        scene.write_instances(entry, &[test_instance_data(stable_instance_key as f32)]);
    }

    let report = scene
        .flush_updates_with_staging(&backend, &mut encoder)
        .expect("staged GPU scene upload should be accepted");

    assert_eq!(report.upload_path, GpuSceneUploadPath::StagingCopy);
    assert!(report.uploaded_bytes >= 256 * 1024);
    backend
        .submit_graphics_command_buffers(vec![encoder.finish()])
        .expect("staged GPU scene test submission should succeed");
}

#[test]
fn render_gpu_scene_staging_ring_rotates_and_preserves_last_instance_upload() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let entries = (0..TEST_STAGING_ENTRY_COUNT)
        .map(|stable_instance_key| scene.register(&backend.device, stable_instance_key, 1))
        .collect::<Vec<_>>();
    let final_frame = 3_u32;

    for frame in 0..=final_frame {
        let mut encoder = backend
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("zircon-test-gpu-scene-staging-ring-rotation"),
            });
        for (index, entry) in entries.iter().copied().enumerate() {
            scene.write_instances(entry, &[test_instance_data(index as f32 + frame as f32)]);
        }

        let report = scene
            .flush_updates_with_staging(&backend, &mut encoder)
            .expect("staged GPU scene upload should be accepted");

        assert_eq!(report.upload_path, GpuSceneUploadPath::StagingCopy);
        backend
            .submit_graphics_command_buffers(vec![encoder.finish()])
            .expect("staged GPU scene test submission should succeed");
    }

    let byte_len = TEST_STAGING_ENTRY_COUNT * GPU_INSTANCE_DATA_STRIDE as u64;
    let uploaded_bytes = read_buffer_bytes(
        &backend.device,
        &backend.queue,
        scene.instance_buffer(),
        BufferByteReadback {
            source_offset: 0,
            byte_len,
            label: "zircon-test-gpu-scene-staging-ring-readback",
        },
    )
    .expect("staging ring test should read back the instance buffer");
    let uploaded_instances = bytemuck::cast_slice::<u8, GpuInstanceData>(&uploaded_bytes);

    assert_eq!(uploaded_instances.len(), TEST_STAGING_ENTRY_COUNT as usize);
    for (index, instance) in uploaded_instances.iter().enumerate() {
        assert_eq!(
            instance.world_from_local[3][0],
            index as f32 + final_frame as f32
        );
    }
}

#[test]
fn render_gpu_scene_core_uploads_use_the_backend_submission_boundary() {
    let direct_upload_source = include_str!("../direct_upload.rs");
    let staged_upload_source = include_str!("../staged_upload.rs");
    let staging_ring_source = include_str!("../staging_ring.rs");

    assert!(direct_upload_source.contains("backend.enqueue_copy_buffer_upload_batch(batch)?"));
    assert!(direct_upload_source.contains("Ok(prepared.commit(self))"));
    assert!(direct_upload_source.contains("prepare_direct_updates"));
    assert!(staged_upload_source.contains("self.staging_ring.encode_upload("));
    assert!(staging_ring_source.contains("WgpuBufferUpload::new("));
    for source in [
        direct_upload_source,
        staged_upload_source,
        staging_ring_source,
    ] {
        assert!(
            !source.contains("queue.write_buffer"),
            "migrated GPU scene upload owner must not write the native queue directly"
        );
        assert!(
            !source.contains("queue.submit"),
            "migrated GPU scene upload owner must not submit the native queue directly"
        );
    }
}

fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
    crate::graphics::backend::RenderBackend::new_offscreen()
        .inspect_err(|error| eprintln!("skipping gpu scene upload test: {error:?}"))
        .ok()
}

fn submit_empty_graphics_frame(backend: &crate::graphics::backend::RenderBackend) {
    let encoder = backend
        .device
        .create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("zircon-test-gpu-scene-buffered-write-flush"),
        });
    backend
        .submit_graphics_command_buffers(vec![encoder.finish()])
        .expect("frame submission should flush accepted GPU scene writes");
}

fn test_gpu_scene(device: &wgpu::Device) -> GpuScene {
    GpuScene::new(
        device,
        test_skinned_joint_palette_buffer(device),
        test_skinned_joint_palette_min_binding_size(),
    )
}

fn test_skinned_joint_palette_buffer(device: &wgpu::Device) -> Arc<wgpu::Buffer> {
    Arc::new(device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("zircon-test-empty-skinned-joint-palette-buffer"),
        size: test_skinned_joint_palette_min_binding_size().get(),
        usage: wgpu::BufferUsages::STORAGE,
        mapped_at_creation: false,
    }))
}

fn test_skinned_joint_palette_min_binding_size() -> wgpu::BufferSize {
    wgpu::BufferSize::new(
        TEST_SKINNED_JOINT_MATRIX_COUNT * TEST_SKINNED_JOINT_MATRIX_BYTES
            + TEST_SKINNED_JOINT_PARAMS_BYTES,
    )
    .expect("test skinned joint palette storage size is non-zero")
}

fn sync_test_entry(
    device: &wgpu::Device,
    scene: &mut GpuScene,
    transform_revision: u64,
    translate_x: f32,
) {
    let entry = scene.register(device, TEST_STABLE_INSTANCE_KEY, 1);
    scene.write_primitive(entry, test_primitive_data());
    scene.write_instances(entry, &[test_instance_data(translate_x)]);
    scene.set_transform_revision(TEST_STABLE_INSTANCE_KEY, transform_revision);
}

fn test_primitive_data() -> GpuPrimitiveData {
    GpuPrimitiveData {
        local_bounds_center: [0.0, 0.0, 0.0],
        local_bounds_radius: 1.0,
        tint: [1.0, 1.0, 1.0, 1.0],
        shadow_params: [0.0, 0.5, 1.0, 0.0],
        motion_params: [0.0, 0.0, 0.0, 0.0],
        flags: GPU_PRIMITIVE_FLAG_VISIBLE,
        first_instance_index: u32::MAX,
        instance_count: u32::MAX,
        payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        material_payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        hit_proxy_token: 0,
        material_payload_padding: [0; 2],
    }
}

fn test_instance_data(translate_x: f32) -> GpuInstanceData {
    let mut world_from_local = test_identity_matrix();
    world_from_local[3][0] = translate_x;
    GpuInstanceData {
        world_from_local,
        prev_world_from_local: test_identity_matrix(),
        primitive_index: u32::MAX,
        flags: 0,
        payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        morph_payload_slot: GPU_SCENE_INVALID_PAYLOAD_SLOT,
        lightmap_uv_rect: [0.0; 4],
        lightmap_params: [0; 4],
        skinning_palette_params: [0; 4],
    }
}

fn test_light_data(light_id: u32) -> GpuLightData {
    GpuLightData {
        color_intensity: [1.0, 0.5, 0.25, 2.0],
        shadow_slot_layer: [u32::MAX, 1, light_id, 0],
        ..GpuLightData::default()
    }
}

fn test_identity_matrix() -> [[f32; 4]; 4] {
    [
        [1.0, 0.0, 0.0, 0.0],
        [0.0, 1.0, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [0.0, 0.0, 0.0, 1.0],
    ]
}
