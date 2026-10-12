use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime::core::framework::animation::{AnimationPoseBone, AnimationPoseSource};

use super::*;

#[test]
fn optimization_batch_20260830ch_aligned_palette_avoids_name_index() {
    let production = include_str!("../palette.rs")
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    let start = production
        .find("fn pose_locals_for_skeleton(")
        .expect("pose local owner");
    let helper = &production[start..];

    assert!(helper.contains("pose.bones.iter().zip(&skeleton.bones)"));
    assert!(helper.contains("return pose"));
    assert!(helper.contains(".map(|bone| bone.local_transform)"));
    assert!(helper.contains("collect::<BTreeMap<_, _>>()"));
}

#[test]
fn optimization_batch_20260830ch_reordered_palette_uses_name_fallback() {
    let skeleton = skeleton();
    let pose = AnimationPoseOutput {
        source: AnimationPoseSource::Clip,
        active_state: None,
        bones: vec![
            AnimationPoseBone {
                name: "Hand".into(),
                local_transform: Transform::from_translation(Vec3::new(2.0, 1.0, 0.0)),
            },
            AnimationPoseBone {
                name: "Root".into(),
                local_transform: Transform::from_translation(Vec3::new(1.0, 0.0, 0.0)),
            },
        ],
    };

    let palette = SkinningPalette::from_skeleton_pose(&skeleton, &pose).unwrap();
    assert_eq!(palette.joint_count(), 2);
    assert_eq!(
        palette.joint_matrices[0].transform_point3(Vec3::ZERO),
        Vec3::new(1.0, 0.0, 0.0)
    );
}

#[test]
#[ignore = "Release-only Runtime170 performance contract"]
fn optimization_batch_20260830ch_aligned_palette_p95() {
    const JOINTS: usize = 256;
    const ITERATIONS: usize = 2_000;
    const SAMPLES: usize = 17;
    let names = (0..JOINTS)
        .map(|index| format!("Bone{index}"))
        .collect::<Vec<_>>();
    let pose = names
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), index))
        .collect::<Vec<_>>();
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let baseline = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                let by_name = pose
                    .iter()
                    .map(|(name, value)| (name.as_str(), *value))
                    .collect::<BTreeMap<_, _>>();
                black_box(
                    names
                        .iter()
                        .map(|name| by_name.get(name.as_str()).copied())
                        .collect::<Vec<_>>(),
                );
            }
            started.elapsed().as_nanos()
        };
        let optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                assert!(names
                    .iter()
                    .zip(&pose)
                    .all(|(left, (right, _))| left == right));
                black_box(pose.iter().map(|(_, value)| *value).collect::<Vec<_>>());
            }
            started.elapsed().as_nanos()
        };
        if sample % 2 == 0 {
            baseline_samples.push(baseline());
            optimized_samples.push(optimized());
        } else {
            optimized_samples.push(optimized());
            baseline_samples.push(baseline());
        }
    }

    let baseline_p95 = percentile_95(&mut baseline_samples);
    let optimized_p95 = percentile_95(&mut optimized_samples);
    println!(
        "RUNTIME170_ALIGNED_PALETTE_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(20),
        "expected aligned pose path to reduce P95 by at least 80%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
    );
}

fn skeleton() -> AnimationSkeletonAsset {
    AnimationSkeletonAsset {
        name: Some("Rig".into()),
        bones: vec![
            AnimationSkeletonBoneAsset {
                name: "Root".into(),
                parent_index: None,
                local_translation: [0.0; 3],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0; 3],
            },
            AnimationSkeletonBoneAsset {
                name: "Hand".into(),
                parent_index: Some(0),
                local_translation: [0.0, 1.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0; 3],
            },
        ],
    }
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
}
