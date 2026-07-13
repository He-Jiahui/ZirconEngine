use zircon_plugin_animation_runtime::{
    AnimationGpuSkinningDecision, SkinningPalette, SkinningPaletteDoubleBuffer, MAX_SKIN_JOINTS,
};
use zircon_runtime::core::framework::animation::{
    AnimationGpuSkinningReadiness, AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource,
    AnimationSkinningBackend,
};
use zircon_runtime::core::framework::animation::{
    AnimationSkeletonAsset, AnimationSkeletonBoneAsset,
};
use zircon_runtime::core::math::{Transform, Vec3};

#[test]
fn palette_double_buffer_swaps_per_frame() {
    let skeleton = skeleton(2);
    let first = SkinningPalette::from_skeleton_pose(&skeleton, &pose(1.0)).unwrap();
    let second = SkinningPalette::from_skeleton_pose(&skeleton, &pose(2.0)).unwrap();
    let mut buffers = SkinningPaletteDoubleBuffer::default();

    buffers.upload(&first);
    assert_eq!(buffers.current().joint_count(), 2);
    assert_eq!(buffers.previous().joint_count(), 0);
    buffers.upload(&second);
    assert_eq!(
        buffers.current().joint_matrices[0],
        second.joint_matrices[0]
    );
    assert_eq!(
        buffers.previous().joint_matrices[0],
        first.joint_matrices[0]
    );
}

#[test]
fn not_ready_falls_back_to_cpu_with_diagnostic() {
    let readiness = AnimationGpuSkinningReadiness::default()
        .with_missing_gpu_resource("gpu_scene.skinning_palettes");
    let decision = AnimationGpuSkinningDecision::select(&readiness, 32);
    assert!(decision.is_cpu_fallback());
    assert!(decision.diagnostic().unwrap().contains("not ready"));
}

#[test]
fn over_256_joints_falls_back() {
    let readiness = AnimationGpuSkinningReadiness {
        enabled: true,
        backend: AnimationSkinningBackend::Gpu,
        ..AnimationGpuSkinningReadiness::default()
    };
    let decision = AnimationGpuSkinningDecision::select(&readiness, MAX_SKIN_JOINTS + 1);
    assert!(decision.is_cpu_fallback());
    assert!(decision.diagnostic().unwrap().contains("256"));
}

#[test]
fn gpu_cpu_skinning_parity_within_tolerance() {
    let skeleton = skeleton(2);
    let palette = SkinningPalette::from_skeleton_pose(&skeleton, &pose(1.0)).unwrap();
    let transformed = palette.joint_matrices[1].transform_point3(Vec3::new(0.0, 1.0, 0.0));
    let expected = Vec3::new(1.0, 1.0, 0.0);
    assert!((transformed - expected).length() <= 1.0e-6);
}

fn skeleton(count: usize) -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("Rig".into()),
        bones: (0..count)
            .map(|index| AnimationSkeletonBoneAsset {
                name: format!("Bone{index}"),
                parent_index: index.checked_sub(1).map(|parent| parent as u32),
                local_translation: [0.0, if index == 0 { 0.0 } else { 1.0 }, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0; 3],
            })
            .collect(),
    }
}

fn pose(root_x: f32) -> AnimationPoseOutput {
    AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones: vec![AnimationPoseBone {
            name: "Bone0".into(),
            local_transform: Transform::from_translation(Vec3::new(root_x, 0.0, 0.0)),
        }],
    }
}
