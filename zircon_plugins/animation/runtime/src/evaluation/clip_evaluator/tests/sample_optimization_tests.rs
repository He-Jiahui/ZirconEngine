use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use super::{record_pose_pool_miss_delta, AnimationClipEvaluatorStats};

#[test]
fn optimization_batch_20260830cd_pose_pool_stats_use_local_miss_delta() {
    let source = include_str!("../sample.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("production source");
    let sample_start = production
        .find("    pub fn sample_clip(")
        .expect("sample owner");
    let sample_end = production[sample_start..]
        .find("    fn ensure_skeleton(")
        .map(|offset| sample_start + offset)
        .expect("sample owner boundary");
    let sample = &production[sample_start..sample_end];

    assert!(sample.contains("pose_pool_miss_count_before"));
    assert!(sample.contains("record_pose_pool_miss_delta"));
    assert!(!sample.contains(".skeletons\n            .values()"));
    assert!(!sample.contains(".map(|cached| cached.pose_pool.miss_count())"));
}

#[test]
fn optimization_batch_20260830cd_pose_pool_miss_delta_is_cumulative_and_saturating() {
    let mut stats = AnimationClipEvaluatorStats {
        pose_pool_miss_count: 7,
        ..AnimationClipEvaluatorStats::default()
    };

    record_pose_pool_miss_delta(&mut stats, 11, 12);
    assert_eq!(stats.pose_pool_miss_count, 8);
    record_pose_pool_miss_delta(&mut stats, 12, 12);
    assert_eq!(stats.pose_pool_miss_count, 8);
    record_pose_pool_miss_delta(&mut stats, 12, 11);
    assert_eq!(stats.pose_pool_miss_count, 8);

    stats.pose_pool_miss_count = u64::MAX;
    record_pose_pool_miss_delta(&mut stats, 0, 1);
    assert_eq!(stats.pose_pool_miss_count, u64::MAX);
}

#[test]
#[ignore = "Release-only Runtime170 performance contract"]
fn optimization_batch_20260830cd_pose_pool_miss_delta_p95() {
    const CACHE_COUNT: u64 = 64;
    const ITERATIONS: usize = 100_000;
    const SAMPLES: usize = 17;
    let pools = (0..CACHE_COUNT)
        .map(|id| (id, id & 3))
        .collect::<BTreeMap<_, _>>();
    let mut baseline_samples = Vec::with_capacity(SAMPLES);
    let mut optimized_samples = Vec::with_capacity(SAMPLES);

    for sample in 0..SAMPLES {
        let baseline = || {
            let started = Instant::now();
            let mut stats = 0_u64;
            for _ in 0..ITERATIONS {
                stats = black_box(&pools).values().copied().sum();
                black_box(stats);
            }
            started.elapsed().as_nanos()
        };
        let optimized = || {
            let started = Instant::now();
            let mut stats = 0_u64;
            for _ in 0..ITERATIONS {
                stats = stats.saturating_add(black_box(11_u64).saturating_sub(black_box(11)));
                black_box(stats);
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
        "RUNTIME170_POSE_POOL_MISS_DELTA_BENCH_V1 baseline_p95_ns={baseline_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95.saturating_mul(100) <= baseline_p95.saturating_mul(20),
        "expected local miss-delta accounting to reduce P95 by at least 80%: baseline={baseline_p95}ns optimized={optimized_p95}ns"
    );
}

fn percentile_95(samples: &mut [u128]) -> u128 {
    samples.sort_unstable();
    samples[(samples.len() * 95 / 100).min(samples.len() - 1)]
}
