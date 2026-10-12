use std::sync::Arc;

use crate::asset::{MeshVertex, ModelPrimitiveAsset};
use crate::core::math::{Mat4, Vec2, Vec3};
use crate::graphics::scene::gpu_scene::{
    GpuSceneSkinnedGpuSourceState, GpuSceneSkinnedJointPaletteState,
};
use crate::graphics::scene::resources::GpuMeshResource;
use crate::graphics::scene::scene_renderer::mesh::skinning::SkinnedMeshJointPaletteStorage;

use super::previous_skinned_gpu_state_for_states;

#[test]
fn previous_skinned_joint_palette_requires_matching_signature_and_joint_count() {
    let current = test_palette_state(7, &[1.0]);
    let previous = test_palette_state(7, &[2.0]);
    let mismatched_signature = test_palette_state(8, &[2.0]);
    let mismatched_joint_count = test_palette_state(7, &[2.0, 3.0]);

    assert_eq!(
        previous_skinned_gpu_state_for_states(false, Some(current), Some(previous), None)
            .joint_palette,
        Some(previous.storage)
    );
    assert_eq!(
        previous_skinned_gpu_state_for_states(
            false,
            Some(current),
            Some(mismatched_signature),
            None,
        )
        .joint_palette,
        None
    );
    assert_eq!(
        previous_skinned_gpu_state_for_states(
            false,
            Some(current),
            Some(mismatched_joint_count),
            None,
        )
        .joint_palette,
        None
    );
}

#[test]
fn previous_skinned_joint_palette_requires_matching_cpu_morphed_shape_or_source() {
    let current = test_palette_state(7, &[1.0]);
    let previous = test_palette_state(7, &[2.0]);
    let current_shape = test_palette_state_with_morph_shape(7, 11, &[1.0]);
    let previous_same_shape = test_palette_state_with_morph_shape(7, 11, &[2.0]);
    let previous_different_shape = test_palette_state_with_morph_shape(7, 12, &[2.0]);

    assert_eq!(
        previous_skinned_gpu_state_for_states(true, Some(current), Some(previous), None)
            .joint_palette,
        None
    );
    assert_eq!(
        previous_skinned_gpu_state_for_states(
            true,
            Some(current_shape),
            Some(previous_same_shape),
            None,
        )
        .joint_palette,
        Some(previous_same_shape.storage)
    );
    assert_eq!(
        previous_skinned_gpu_state_for_states(
            true,
            Some(current_shape),
            Some(previous_different_shape),
            None,
        )
        .joint_palette,
        None
    );
}

#[test]
fn previous_skinned_joint_palette_accepts_changed_cpu_morphed_shape_with_previous_source() {
    let Some(backend) = test_backend() else {
        return;
    };
    let current_shape = test_palette_state_with_morph_shape(7, 12, &[1.0]);
    let previous_shape = test_palette_state_with_morph_shape(7, 11, &[2.0]);
    let previous_source = GpuSceneSkinnedGpuSourceState {
        morph_shape_signature: 11,
        mesh: test_source(&backend.device, 1.0),
    };

    let selected = previous_skinned_gpu_state_for_states(
        true,
        Some(current_shape),
        Some(previous_shape),
        Some(previous_source.clone()),
    );

    assert_eq!(selected.joint_palette, Some(previous_shape.storage));
    assert!(Arc::ptr_eq(
        &selected.source.expect("previous source"),
        &previous_source.mesh
    ));
}

#[test]
fn previous_skinned_joint_palette_rejects_changed_cpu_morphed_shape_without_matching_source() {
    let Some(backend) = test_backend() else {
        return;
    };
    let current_shape = test_palette_state_with_morph_shape(7, 12, &[1.0]);
    let previous_shape = test_palette_state_with_morph_shape(7, 11, &[2.0]);
    let wrong_source = GpuSceneSkinnedGpuSourceState {
        morph_shape_signature: 10,
        mesh: test_source(&backend.device, 1.0),
    };

    assert_eq!(
        previous_skinned_gpu_state_for_states(
            true,
            Some(current_shape),
            Some(previous_shape),
            Some(wrong_source),
        )
        .joint_palette,
        None
    );
    assert_eq!(
        previous_skinned_gpu_state_for_states(
            true,
            Some(current_shape),
            Some(previous_shape),
            None,
        )
        .joint_palette,
        None
    );
}

fn test_palette_state(
    signature: u64,
    translate_x_values: &[f32],
) -> GpuSceneSkinnedJointPaletteState {
    test_palette_state_with_optional_morph_shape(signature, None, translate_x_values)
}

fn test_palette_state_with_morph_shape(
    signature: u64,
    morph_shape_signature: u64,
    translate_x_values: &[f32],
) -> GpuSceneSkinnedJointPaletteState {
    test_palette_state_with_optional_morph_shape(
        signature,
        Some(morph_shape_signature),
        translate_x_values,
    )
}

fn test_palette_state_with_optional_morph_shape(
    signature: u64,
    morph_shape_signature: Option<u64>,
    translate_x_values: &[f32],
) -> GpuSceneSkinnedJointPaletteState {
    let matrices = translate_x_values
        .iter()
        .copied()
        .map(|translate_x| Mat4::from_translation(Vec3::new(translate_x, 0.0, 0.0)))
        .collect::<Vec<_>>();
    GpuSceneSkinnedJointPaletteState {
        signature,
        morph_shape_signature,
        storage: SkinnedMeshJointPaletteStorage::from_matrices(&matrices)
            .expect("test palette fits fixed storage ABI"),
    }
}

fn test_backend() -> Option<crate::graphics::backend::RenderBackend> {
    crate::graphics::backend::RenderBackend::new_offscreen()
        .inspect_err(|error| eprintln!("skipping previous skinned source policy test: {error:?}"))
        .ok()
}

fn test_source(device: &wgpu::Device, x: f32) -> Arc<GpuMeshResource> {
    Arc::new(GpuMeshResource::from_asset(
        device,
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
        },
    ))
}
