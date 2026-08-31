use std::hint::black_box;
use std::time::Instant;

use super::super::binary::{encode_binary_asset, AnimationBinaryAssetKind};
use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hs_runtime_preserves_skeleton_wire_bytes_and_round_trip() {
    let skeleton = benchmark_skeleton(64, 128);

    let optimized = skeleton.to_bytes().expect("borrowed skeleton encoding");
    let legacy = legacy_to_bytes(&skeleton).expect("owned skeleton encoding");

    assert_eq!(optimized, legacy);
    assert_eq!(
        AnimationSkeletonAsset::from_bytes(&optimized).unwrap(),
        skeleton
    );
}

#[test]
fn optimization_batch_20260828hs_runtime_to_bytes_borrows_skeleton_bones() {
    let source = include_str!("../skeleton.rs");
    let to_bytes = source
        .split("pub fn to_bytes")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("skeleton to_bytes implementation");

    assert!(source.contains("struct AnimationSkeletonAssetRef<'a>"));
    assert!(source.contains("bones: &'a [AnimationSkeletonBoneAsset]"));
    assert!(to_bytes.contains("AnimationSkeletonAssetRef::from(self)"));
    assert!(!to_bytes.contains("AnimationBinaryAssetKind::Skeleton, self"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hs_runtime_borrowed_skeleton_encoding_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let skeleton = benchmark_skeleton(512, 4 * 1024);

    black_box(skeleton.to_bytes().unwrap());
    black_box(legacy_to_bytes(&skeleton).unwrap());

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_to_bytes(black_box(&skeleton)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(black_box(&skeleton).to_bytes().unwrap());
            }
            started.elapsed().as_nanos()
        };
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy());
            optimized_samples.push(measure_optimized());
        } else {
            optimized_samples.push(measure_optimized());
            legacy_samples.push(measure_legacy());
        }
    }

    let legacy_p95_ns = nearest_rank_percentile(&legacy_samples, 95);
    let optimized_p95_ns = nearest_rank_percentile(&optimized_samples, 95);
    println!(
        "RUNTIME265_BORROWED_SKELETON_BINARY_ENCODING_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_to_bytes(skeleton: &AnimationSkeletonAsset) -> AnimationAssetResult<Vec<u8>> {
    encode_binary_asset(AnimationBinaryAssetKind::Skeleton, skeleton)
}

fn benchmark_skeleton(bone_count: usize, name_bytes: usize) -> AnimationSkeletonAsset {
    let name_suffix = "x".repeat(name_bytes);
    AnimationSkeletonAsset {
        name: Some("benchmark-skeleton".to_string()),
        bones: (0..bone_count)
            .map(|bone_index| AnimationSkeletonBoneAsset {
                name: format!("bone-{bone_index}-{name_suffix}"),
                parent_index: bone_index.checked_sub(1).map(|index| index as u32),
                local_translation: [bone_index as Real, 0.0, 0.0],
                local_rotation: [0.0, 0.0, 0.0, 1.0],
                local_scale: [1.0, 1.0, 1.0],
            })
            .collect(),
    }
}

fn nearest_rank_percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    ordered[(ordered.len() * percentile).div_ceil(100) - 1]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
