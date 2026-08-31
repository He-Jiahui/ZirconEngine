use std::hint::black_box;
use std::time::Instant;

use crate::core::math::Real;
use crate::core::resource::{AssetReference, ResourceLocator};

use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hq_runtime_preserves_clip_wire_bytes_and_round_trip() {
    let clip = benchmark_clip(4, 32);

    let optimized = clip.to_bytes().expect("borrowed clip encoding");
    let legacy = legacy_to_bytes(&clip).expect("owned clip encoding");

    assert_eq!(optimized, legacy);
    assert_eq!(AnimationClipAsset::from_bytes(&optimized).unwrap(), clip);
}

#[test]
fn optimization_batch_20260828hq_runtime_to_bytes_borrows_large_clip_collections() {
    let source = include_str!("../clip.rs");
    let to_bytes = source
        .split("pub fn to_bytes")
        .nth(1)
        .and_then(|body| body.split("pub fn direct_references").next())
        .expect("clip to_bytes implementation");

    assert!(source.contains("struct AnimationClipBinaryAssetRef<'a>"));
    assert!(source.contains("tracks: &'a [AnimationClipBoneTrackAsset]"));
    assert!(source.contains("event_tracks: &'a [AnimationEventTrackAsset]"));
    assert!(to_bytes.contains("AnimationClipBinaryAssetRef::from(self)"));
    assert!(!to_bytes.contains("AnimationClipBinaryAsset::from(self)"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hq_runtime_borrowed_clip_encoding_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let clip = benchmark_clip(1_024, 1_024);

    black_box(clip.to_bytes().unwrap());
    black_box(legacy_to_bytes(&clip).unwrap());

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_to_bytes(black_box(&clip)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(black_box(&clip).to_bytes().unwrap());
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
        "RUNTIME263_BORROWED_CLIP_BINARY_ENCODING_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_to_bytes(clip: &AnimationClipAsset) -> AnimationAssetResult<Vec<u8>> {
    encode_binary_asset(
        AnimationBinaryAssetKind::Clip,
        &AnimationClipBinaryAsset::from(clip),
    )
}

fn benchmark_clip(event_count: usize, payload_bytes: usize) -> AnimationClipAsset {
    let payload = "p".repeat(payload_bytes);
    AnimationClipAsset {
        name: Some("benchmark-clip".to_string()),
        skeleton: AssetReference::from_locator(
            ResourceLocator::parse("res://animation/benchmark.skeleton.zranim").unwrap(),
        ),
        duration_seconds: 1.0,
        tracks: Vec::new(),
        event_tracks: (0..event_count)
            .map(|index| AnimationEventTrackAsset {
                target_id: Some(format!("Root/Bone{index}")),
                event: format!("event-{index}"),
                time_seconds: index as Real / event_count.max(1) as Real,
                payload: Some(payload.clone()),
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
