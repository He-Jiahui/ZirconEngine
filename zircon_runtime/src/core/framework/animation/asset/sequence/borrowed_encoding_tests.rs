use std::hint::black_box;
use std::time::Instant;

use super::super::channel::{AnimationChannelKeyAsset, AnimationChannelValueAsset};
use super::*;

const TARGET_P95_PERCENT: u128 = 70;

#[test]
fn optimization_batch_20260828hr_runtime_preserves_sequence_wire_bytes_and_round_trip() {
    let sequence = benchmark_sequence(4, 8, 32);

    let optimized = sequence.to_bytes().expect("borrowed sequence encoding");
    let legacy = legacy_to_bytes(&sequence).expect("owned sequence encoding");

    assert_eq!(optimized, legacy);
    assert_eq!(
        AnimationSequenceAsset::from_bytes(&optimized).unwrap(),
        sequence
    );
}

#[test]
fn optimization_batch_20260828hr_runtime_to_bytes_borrows_sequence_bindings() {
    let source = include_str!("../sequence.rs");
    let to_bytes = source
        .split("pub fn to_bytes")
        .nth(1)
        .and_then(|body| body.split("pub fn track_paths").next())
        .expect("sequence to_bytes implementation");

    assert!(source.contains("struct AnimationSequenceAssetRef<'a>"));
    assert!(source.contains("bindings: &'a [AnimationSequenceBindingAsset]"));
    assert!(to_bytes.contains("AnimationSequenceAssetRef::from(self)"));
    assert!(!to_bytes.contains("AnimationBinaryAssetKind::Sequence, self"));
}

#[test]
#[ignore = "release performance contract; run through the managed validation coordinator"]
fn optimization_batch_20260828hr_runtime_borrowed_sequence_encoding_benchmark() {
    const SAMPLES: usize = 11;
    const ITERATIONS: usize = 8;
    let sequence = benchmark_sequence(512, 16, 2_048);

    black_box(sequence.to_bytes().unwrap());
    black_box(legacy_to_bytes(&sequence).unwrap());

    let mut legacy_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);
    for sample_index in 0..SAMPLES {
        let measure_legacy = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(legacy_to_bytes(black_box(&sequence)).unwrap());
            }
            started.elapsed().as_nanos()
        };
        let measure_optimized = || {
            let started = Instant::now();
            for _ in 0..ITERATIONS {
                black_box(black_box(&sequence).to_bytes().unwrap());
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
        "RUNTIME264_BORROWED_SEQUENCE_BINARY_ENCODING_BENCH_V1 legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100)
            <= legacy_p95_ns.saturating_mul(TARGET_P95_PERCENT),
        "optimized P95 {optimized_p95_ns}ns must be at most {TARGET_P95_PERCENT}% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_to_bytes(sequence: &AnimationSequenceAsset) -> AnimationAssetResult<Vec<u8>> {
    encode_binary_asset(AnimationBinaryAssetKind::Sequence, sequence)
}

fn benchmark_sequence(
    binding_count: usize,
    key_count: usize,
    target_bytes: usize,
) -> AnimationSequenceAsset {
    let target_suffix = "x".repeat(target_bytes);
    AnimationSequenceAsset {
        name: Some("benchmark-sequence".to_string()),
        duration_seconds: 1.0,
        frames_per_second: 60.0,
        bindings: (0..binding_count)
            .map(|binding_index| AnimationSequenceBindingAsset {
                entity_path: EntityPath::parse(&format!("Root/Node{binding_index}")).unwrap(),
                target_id: Some(format!("Root/Node{binding_index}/{target_suffix}")),
                tracks: vec![AnimationSequenceTrackAsset {
                    property_path: ComponentPropertyPath::parse("Transform.translation").unwrap(),
                    channel: AnimationChannelAsset {
                        interpolation: super::super::channel::AnimationInterpolationAsset::Linear,
                        keys: (0..key_count)
                            .map(|key_index| AnimationChannelKeyAsset {
                                time_seconds: key_index as Real / key_count.max(1) as Real,
                                value: AnimationChannelValueAsset::Vec3([
                                    binding_index as Real,
                                    key_index as Real,
                                    0.0,
                                ]),
                                in_tangent: None,
                                out_tangent: None,
                            })
                            .collect(),
                    },
                }],
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
