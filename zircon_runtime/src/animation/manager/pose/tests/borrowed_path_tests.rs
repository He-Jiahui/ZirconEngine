use std::hint::black_box;
use std::time::{Duration, Instant};

use super::skeleton_bone_path;
use crate::core::framework::animation::{AnimationSkeletonAsset, AnimationSkeletonBoneAsset};

const SAMPLE_PAIRS: usize = 101;
const PATHS_PER_SAMPLE: usize = 4_096;
const BONE_DEPTH: usize = 32;

#[test]
fn runtime877_skeleton_path_borrowed_segments_preserves_exact_paths() {
    let skeleton = AnimationSkeletonAsset {
        name: Some("动画骨骼".to_string()),
        bones: vec![
            bone("Root", None),
            bone("", Some(0)),
            bone("骨🙂", Some(1)),
            bone("duplicate", Some(2)),
            bone("duplicate", Some(3)),
            bone("orphan", Some(200)),
        ],
    };
    for index in 0..=skeleton.bones.len() {
        assert_eq!(
            skeleton_bone_path(&skeleton, index),
            legacy_path(&skeleton, index)
        );
    }
}

#[test]
#[ignore = "release percentile evidence; run through the managed Windows validation lane"]
fn runtime877_skeleton_path_borrowed_segments_release_percentiles() {
    let skeleton = AnimationSkeletonAsset {
        name: None,
        bones: (0..BONE_DEPTH)
            .map(|index| {
                bone(
                    &format!("bone_{index:02}"),
                    index.checked_sub(1).map(|parent| parent as u32),
                )
            })
            .collect(),
    };
    let mut legacy = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy.push(measure(&skeleton, legacy_path));
            optimized.push(measure(&skeleton, skeleton_bone_path));
        } else {
            optimized.push(measure(&skeleton, skeleton_bone_path));
            legacy.push(measure(&skeleton, legacy_path));
        }
    }
    let legacy_p50_ns = percentile(&mut legacy.clone(), 50);
    let legacy_p95_ns = percentile(&mut legacy.clone(), 95);
    let legacy_p99_ns = percentile(&mut legacy, 99);
    let optimized_p50_ns = percentile(&mut optimized.clone(), 50);
    let optimized_p95_ns = percentile(&mut optimized.clone(), 95);
    let optimized_p99_ns = percentile(&mut optimized, 99);
    println!(
        "RUNTIME877_SKELETON_PATH_BORROWED_SEGMENTS_BENCH_V1 legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} legacy_p99_ns={legacy_p99_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} optimized_p99_ns={optimized_p99_ns}"
    );
    assert!(optimized_p95_ns <= legacy_p95_ns.saturating_mul(110).div_ceil(100));
}

fn bone(name: &str, parent_index: Option<u32>) -> AnimationSkeletonBoneAsset {
    AnimationSkeletonBoneAsset {
        name: name.to_string(),
        parent_index,
        local_translation: [0.0; 3],
        local_rotation: [0.0, 0.0, 0.0, 1.0],
        local_scale: [1.0; 3],
    }
}

fn legacy_path(skeleton: &AnimationSkeletonAsset, index: usize) -> Option<String> {
    let bone = skeleton.bones.get(index)?;
    let mut segments = vec![bone.name.clone()];
    let mut parent = bone.parent_index;
    while let Some(parent_index) = parent {
        let parent_bone = skeleton.bones.get(parent_index as usize)?;
        segments.push(parent_bone.name.clone());
        parent = parent_bone.parent_index;
    }
    segments.reverse();
    Some(segments.join("/"))
}

fn measure(
    skeleton: &AnimationSkeletonAsset,
    render: fn(&AnimationSkeletonAsset, usize) -> Option<String>,
) -> Duration {
    let started = Instant::now();
    let checksum = (0..PATHS_PER_SAMPLE)
        .map(|_| black_box(render(black_box(skeleton), BONE_DEPTH - 1).unwrap()).len())
        .sum::<usize>();
    black_box(checksum);
    started.elapsed()
}

fn percentile(samples: &mut [Duration], percent: usize) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * percent).div_ceil(100).saturating_sub(1)].as_nanos()
}
