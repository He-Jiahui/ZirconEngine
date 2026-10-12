use std::hint::black_box;
use std::time::Instant;

use crate::core::framework::animation::{
    AnimationPoseBone, AnimationPoseOutput, AnimationPoseSource, AnimationSkeletonAsset,
    AnimationSkeletonBoneAsset,
};
use crate::core::math::{Quat, Vec3};

use super::*;

const SAMPLE_PAIRS: usize = 17;
const BUILDS_PER_SAMPLE: usize = 96;
const BONE_COUNT: usize = 256;

#[test]
fn optimization_batch_de_runtime412_fused_joint_palette_matches_legacy_staged_build() {
    let (skeleton, pose) = benchmark_skeleton_and_pose();

    assert_eq!(
        build_joint_matrices(&skeleton, &pose).unwrap(),
        legacy_joint_matrices(&skeleton, &pose).unwrap()
    );
}

#[test]
fn optimization_batch_de_runtime412_fused_joint_palette_preserves_missing_parent_errors() {
    let (mut skeleton, pose) = benchmark_skeleton_and_pose();
    skeleton.bones[1].parent_index = Some(BONE_COUNT as u32);

    let error = build_joint_matrices(&skeleton, &pose).unwrap_err();

    assert!(error.contains("at index 1 references missing parent 256"));
}

#[test]
fn optimization_batch_de_runtime412_fused_joint_palette_uses_two_bone_sized_world_buffers() {
    const SOURCE: &str = include_str!("../skinning.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();
    let build = production
        .split("fn build_joint_matrices")
        .nth(1)
        .unwrap()
        .split("fn compose_world_matrix")
        .next()
        .unwrap();

    assert_eq!(
        build
            .matches("Vec::with_capacity(skeleton.bones.len())")
            .count(),
        2
    );
    assert!(!build.contains("collect::<Vec<_>>()"));
    assert!(build.contains("posed_worlds.iter_mut().zip(bind_worlds)"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_de_runtime412_skinning_palette_fused_build_p95() {
    let (skeleton, pose) = benchmark_skeleton_and_pose();
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&skeleton, &pose, false));
            optimized.push(measure(&skeleton, &pose, true));
        } else {
            optimized.push(measure(&skeleton, &pose, true));
            legacy.push(measure(&skeleton, &pose, false));
        }
    }
    let legacy_p95_ns = percentile(&legacy, 95);
    let optimized_p95_ns = percentile(&optimized, 95);
    println!(
        "RUNTIME412_SKINNING_PALETTE_FUSED_BUILD_BENCH_V1 sample_pairs={SAMPLE_PAIRS} builds_per_sample={BUILDS_PER_SAMPLE} bones_per_build={BONE_COUNT} legacy_bone_vectors_per_build=5 optimized_bone_vectors_per_build=2 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        csv(&legacy),
        csv(&optimized)
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "fused palette construction must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn benchmark_skeleton_and_pose() -> (AnimationSkeletonAsset, AnimationPoseOutput) {
    let bones = (0..BONE_COUNT)
        .map(|index| AnimationSkeletonBoneAsset {
            name: format!("joint-{index}"),
            // BUG: [CR-SCENE-MESH-0001] 首骨骼 index=0 时 then_some 仍先计算 index-1；开启溢出检查时两个 runtime412 功能测试在夹具构造阶段 panic。
            parent_index: (index > 0).then_some((index - 1) as u32),
            local_translation: [0.01, 0.02, 0.03],
            local_rotation: Quat::IDENTITY.to_array(),
            local_scale: Vec3::ONE.to_array(),
        })
        .collect::<Vec<_>>();
    let pose_bones = (0..BONE_COUNT)
        .step_by(2)
        .map(|index| AnimationPoseBone {
            name: format!("joint-{index}"),
            local_transform: Transform::from_translation(Vec3::new(0.02, 0.01, 0.03)),
        })
        .collect();
    (
        AnimationSkeletonAsset {
            name: Some("optimization-runtime412".to_string()),
            bones,
        },
        AnimationPoseOutput {
            source: AnimationPoseSource::Clip,
            active_state: None,
            bones: pose_bones,
        },
    )
}

fn legacy_joint_matrices(
    skeleton: &AnimationSkeletonAsset,
    pose: &AnimationPoseOutput,
) -> Result<Vec<Mat4>, String> {
    let bind_locals = skeleton
        .bones
        .iter()
        .map(bind_transform)
        .collect::<Vec<_>>();
    let pose_by_name = pose
        .bones
        .iter()
        .map(|bone| (bone.name.as_str(), bone.local_transform))
        .collect::<HashMap<_, _>>();
    let pose_locals = skeleton
        .bones
        .iter()
        .map(|bone| {
            pose_by_name
                .get(bone.name.as_str())
                .copied()
                .unwrap_or_else(|| bind_transform(bone))
        })
        .collect::<Vec<_>>();
    let bind_worlds = legacy_world_matrices(skeleton, &bind_locals)?;
    let posed_worlds = legacy_world_matrices(skeleton, &pose_locals)?;
    Ok(bind_worlds
        .into_iter()
        .zip(posed_worlds)
        .map(|(bind_world, posed_world)| posed_world * bind_world.inverse())
        .collect())
}

fn legacy_world_matrices(
    skeleton: &AnimationSkeletonAsset,
    locals: &[Transform],
) -> Result<Vec<Mat4>, String> {
    let mut worlds = Vec::with_capacity(locals.len());
    for (index, (bone, local)) in skeleton.bones.iter().zip(locals).enumerate() {
        let world = compose_world_matrix(&worlds, bone, index, local.matrix())?;
        worlds.push(world);
    }
    Ok(worlds)
}

fn measure(skeleton: &AnimationSkeletonAsset, pose: &AnimationPoseOutput, optimized: bool) -> u128 {
    let started = Instant::now();
    for _ in 0..BUILDS_PER_SAMPLE {
        let matrices = if optimized {
            build_joint_matrices(black_box(skeleton), black_box(pose))
        } else {
            legacy_joint_matrices(black_box(skeleton), black_box(pose))
        }
        .unwrap();
        black_box(matrices);
    }
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[(sorted.len() * percentile).div_ceil(100).saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
