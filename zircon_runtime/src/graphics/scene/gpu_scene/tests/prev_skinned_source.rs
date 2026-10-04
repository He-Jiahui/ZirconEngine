use std::sync::Arc;

use crate::asset::{MeshVertex, ModelPrimitiveAsset};
use crate::core::math::{Vec2, Vec3};
use crate::graphics::scene::gpu_scene::GpuScene;
use crate::graphics::scene::resources::GpuMeshResource;

use super::*;

const TEST_STABLE_INSTANCE_KEY: u64 = 0x7100_0001;
const TEST_OTHER_STABLE_INSTANCE_KEY: u64 = 0x7100_0002;
const TEST_SKINNED_JOINT_MATRIX_COUNT: u64 = 256;
const TEST_SKINNED_JOINT_MATRIX_BYTES: u64 = 64;
const TEST_SKINNED_JOINT_PARAMS_BYTES: u64 = 16;

#[test]
fn render_gpu_scene_rolls_current_skinned_gpu_source_after_success() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);
    let first = test_source(&backend.device, 1.0);
    let second = test_source(&backend.device, 2.0);

    scene.stage_current_skinned_gpu_source(TEST_STABLE_INSTANCE_KEY, Some(first.clone()));
    let first_report = scene.roll_prev_skinned_gpu_sources_after_success();

    assert_eq!(first_report.current_source_count, 1);
    assert_eq!(first_report.previous_source_count, 1);
    assert_eq!(first_report.removed_previous_source_count, 0);
    assert!(Arc::ptr_eq(
        &scene
            .previous_skinned_gpu_source_state(TEST_STABLE_INSTANCE_KEY)
            .expect("first previous source")
            .mesh,
        &first.mesh
    ));

    scene.stage_current_skinned_gpu_source(TEST_STABLE_INSTANCE_KEY, Some(second.clone()));

    assert!(Arc::ptr_eq(
        &scene
            .previous_skinned_gpu_source_state(TEST_STABLE_INSTANCE_KEY)
            .expect("rolled source should stay first before success")
            .mesh,
        &first.mesh
    ));

    let second_report = scene.roll_prev_skinned_gpu_sources_after_success();

    assert_eq!(second_report.current_source_count, 1);
    assert_eq!(second_report.previous_source_count, 1);
    assert_eq!(second_report.removed_previous_source_count, 0);
    assert!(Arc::ptr_eq(
        &scene
            .previous_skinned_gpu_source_state(TEST_STABLE_INSTANCE_KEY)
            .expect("second previous source")
            .mesh,
        &second.mesh
    ));
}

#[test]
fn render_gpu_scene_drops_previous_skinned_gpu_source_when_current_is_missing() {
    let Some(backend) = test_backend() else {
        return;
    };
    let mut scene = test_gpu_scene(&backend.device);

    scene.stage_current_skinned_gpu_source(
        TEST_STABLE_INSTANCE_KEY,
        Some(test_source(&backend.device, 1.0)),
    );
    scene.stage_current_skinned_gpu_source(
        TEST_OTHER_STABLE_INSTANCE_KEY,
        Some(test_source(&backend.device, 2.0)),
    );
    let _ = scene.roll_prev_skinned_gpu_sources_after_success();

    scene.stage_current_skinned_gpu_source(TEST_STABLE_INSTANCE_KEY, None);
    let report = scene.roll_prev_skinned_gpu_sources_after_success();

    assert_eq!(report.current_source_count, 1);
    assert_eq!(report.previous_source_count, 1);
    assert_eq!(report.removed_previous_source_count, 1);
    assert!(scene
        .previous_skinned_gpu_source_state(TEST_STABLE_INSTANCE_KEY)
        .is_none());
    assert!(scene
        .previous_skinned_gpu_source_state(TEST_OTHER_STABLE_INSTANCE_KEY)
        .is_some());
}

fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
    crate::graphics::backend::RenderBackend::new_offscreen()
        .inspect_err(|error| eprintln!("skipping gpu scene prev source test: {error:?}"))
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
        label: Some("zircon-test-prev-skinned-source-palette-buffer"),
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

fn test_source(device: &wgpu::Device, x: f32) -> GpuSceneSkinnedGpuSourceState {
    GpuSceneSkinnedGpuSourceState {
        morph_shape_signature: x.to_bits() as u64,
        mesh: Arc::new(GpuMeshResource::from_asset(device, test_primitive(x))),
    }
}

fn test_primitive(x: f32) -> ModelPrimitiveAsset {
    ModelPrimitiveAsset {
        vertices: vec![
            MeshVertex::new(Vec3::new(x, 0.0, 0.0), Vec3::Z, Vec2::ZERO),
            MeshVertex::new(Vec3::new(x + 1.0, 0.0, 0.0), Vec3::Z, Vec2::ZERO),
            MeshVertex::new(Vec3::new(x, 1.0, 0.0), Vec3::Z, Vec2::ZERO),
        ],
        indices: vec![0, 1, 2],
        mesh: None,
        mesh_sdf: None,
        virtual_geometry: None,
    }
}
