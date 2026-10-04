use std::sync::Arc;

use crate::graphics::scene::gpu_scene::{
    GpuMorphDelta, GpuMorphPayload, GpuMorphWeight, GpuScene, GpuSceneMorphUploadReport,
    GPU_MORPH_DELTA_STRIDE,
};

const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

#[test]
fn render_gpu_scene_uploads_morph_storage_buffers() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let payloads = [GpuMorphPayload::new(0, 0, 2, 1)];
    let deltas = [
        GpuMorphDelta::position_xyz(1.0, 2.0, 3.0),
        GpuMorphDelta::position_xyz(-1.0, 0.5, 0.25),
    ];
    let weights = [GpuMorphWeight::new(0.25), GpuMorphWeight::new(0.75)];

    let first_report = submit_morph_upload(&mut scene, &backend, &payloads, &deltas, &weights);

    assert_eq!(first_report.payload_count, 1);
    assert_eq!(first_report.delta_count, 2);
    assert_eq!(first_report.weight_count, 2);
    assert!(first_report.uploaded_bytes > 0);
    assert!(first_report.rebuilt_bind_group);
    assert_eq!(scene.debug_morph_payloads_shadow(), &payloads);
    assert_eq!(scene.debug_morph_deltas_shadow(), &deltas);
    assert_eq!(scene.debug_morph_weights_shadow(), &weights);

    let second_report = submit_morph_upload(&mut scene, &backend, &payloads, &deltas, &weights);

    assert_eq!(second_report.payload_count, 1);
    assert_eq!(second_report.delta_count, 2);
    assert_eq!(second_report.weight_count, 2);
    assert_eq!(second_report.uploaded_bytes, 0);
    assert!(!second_report.rebuilt_bind_group);
}

#[test]
fn render_gpu_scene_reuses_morph_buffers_when_active_rows_shrink() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let payloads = [
        GpuMorphPayload::new(0, 0, 1, 1),
        GpuMorphPayload::new(1, 2, 1, 1),
    ];
    let deltas = [
        GpuMorphDelta::position_xyz(1.0, 2.0, 3.0),
        GpuMorphDelta::position_xyz(4.0, 5.0, 6.0),
    ];
    let weights = [GpuMorphWeight::new(0.25), GpuMorphWeight::new(0.75)];

    let first_report = submit_morph_upload(&mut scene, &backend, &payloads, &deltas, &weights);
    assert!(first_report.rebuilt_bind_group);

    let shrunk_payloads = [GpuMorphPayload::new(0, 0, 1, 1)];
    let shrunk_deltas = [GpuMorphDelta::position_xyz(7.0, 8.0, 9.0)];
    let shrunk_weights = [GpuMorphWeight::new(0.5)];
    let shrunk_report = submit_morph_upload(
        &mut scene,
        &backend,
        &shrunk_payloads,
        &shrunk_deltas,
        &shrunk_weights,
    );

    assert!(shrunk_report.uploaded_bytes > 0);
    assert!(
        !shrunk_report.rebuilt_bind_group,
        "smaller morph payloads must reuse the existing storage buffers and scene bind group"
    );
    assert_eq!(scene.debug_morph_payloads_shadow(), &shrunk_payloads);
    assert_eq!(scene.debug_morph_deltas_shadow(), &shrunk_deltas);
    assert_eq!(scene.debug_morph_weights_shadow(), &shrunk_weights);
}

#[test]
fn render_gpu_scene_uploads_only_changed_morph_delta_rows() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let payloads = [GpuMorphPayload::new(0, 0, 1, 1)];
    let initial_deltas = [
        GpuMorphDelta::position_xyz(1.0, 2.0, 3.0),
        GpuMorphDelta::position_xyz(4.0, 5.0, 6.0),
    ];
    let weights = [GpuMorphWeight::new(0.25)];
    let _ = submit_morph_upload(&mut scene, &backend, &payloads, &initial_deltas, &weights);

    let changed_deltas = [
        initial_deltas[0],
        GpuMorphDelta::position_xyz(7.0, 8.0, 9.0),
    ];
    let report = submit_morph_upload(&mut scene, &backend, &payloads, &changed_deltas, &weights);

    assert_eq!(
        report.uploaded_bytes, GPU_MORPH_DELTA_STRIDE as u64,
        "one changed morph delta must upload exactly one storage row"
    );
    assert!(!report.rebuilt_bind_group);
}

#[test]
fn render_gpu_scene_dropped_morph_preparation_keeps_committed_shadow_for_retry() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let payloads = [GpuMorphPayload::new(0, 0, 1, 1)];
    let deltas = [GpuMorphDelta::position_xyz(1.0, 2.0, 3.0)];
    let weights = [GpuMorphWeight::new(0.5)];
    submit_morph_upload(&mut scene, &backend, &payloads, &deltas, &weights);
    let grown_payloads =
        vec![GpuMorphPayload::new(0, 0, 1, 1); scene.morph_payloads_capacity as usize + 1];
    let dropped = scene.prepare_morph_buffers(
        &backend.device,
        grown_payloads,
        deltas.to_vec(),
        weights.to_vec(),
    );
    assert!(dropped.report().uploaded_bytes > 0);
    assert!(dropped.report().rebuilt_bind_group);
    let mut dropped_frame = scene.prepare_direct_updates();
    dropped_frame.append_morph_upload(dropped);
    drop(dropped_frame);
    assert_eq!(scene.debug_morph_payloads_shadow(), &payloads);

    let retry = scene.prepare_morph_buffers(
        &backend.device,
        payloads.to_vec(),
        deltas.to_vec(),
        weights.to_vec(),
    );
    assert!(retry.report().uploaded_bytes > 0);
}

#[test]
fn render_gpu_scene_rejects_overlapping_morph_preparations() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let first = scene.prepare_morph_buffers(
        &backend.device,
        vec![GpuMorphPayload::new(0, 0, 1, 1)],
        vec![GpuMorphDelta::position_xyz(1.0, 2.0, 3.0)],
        vec![GpuMorphWeight::new(0.5)],
    );

    let overlapping = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        scene.prepare_morph_buffers(&backend.device, Vec::new(), Vec::new(), Vec::new())
    }));
    assert!(overlapping.is_err());

    drop(first);
    let retry = scene.prepare_morph_buffers(&backend.device, Vec::new(), Vec::new(), Vec::new());
    drop(retry);
}

#[test]
fn render_gpu_scene_rejects_foreign_morph_preparation_attachment() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut source_scene = test_gpu_scene(&backend.device);
    let mut target_scene = test_gpu_scene(&backend.device);
    let prepared = source_scene.prepare_morph_buffers(
        &backend.device,
        vec![GpuMorphPayload::new(0, 0, 1, 1)],
        vec![GpuMorphDelta::position_xyz(1.0, 2.0, 3.0)],
        vec![GpuMorphWeight::new(0.5)],
    );
    let mut target_frame = target_scene.prepare_direct_updates();

    let attachment = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        target_frame.append_morph_upload(prepared);
    }));

    assert!(attachment.is_err());
    drop(target_frame);
    let retry =
        source_scene.prepare_morph_buffers(&backend.device, Vec::new(), Vec::new(), Vec::new());
    drop(retry);
}

fn submit_morph_upload(
    scene: &mut GpuScene,
    backend: &crate::graphics::backend::RenderBackend,
    payloads: &[GpuMorphPayload],
    deltas: &[GpuMorphDelta],
    weights: &[GpuMorphWeight],
) -> GpuSceneMorphUploadReport {
    let prepared = scene.prepare_morph_buffers(
        &backend.device,
        payloads.to_vec(),
        deltas.to_vec(),
        weights.to_vec(),
    );
    let report = prepared.report();
    let mut frame = scene.prepare_direct_updates();
    frame.append_morph_upload(prepared);
    scene
        .submit_prepared_upload(backend, frame)
        .expect("morph upload batch must be accepted by the test backend");
    report
}

fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
    crate::graphics::backend::RenderBackend::new_offscreen()
        .inspect_err(|error| eprintln!("skipping gpu scene morph upload test: {error:?}"))
        .ok()
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
