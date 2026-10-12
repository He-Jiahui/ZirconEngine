use std::hint::black_box;
use std::time::Instant;

use super::*;

const SAMPLE_PAIRS: usize = 17;
const SUMMARIES_PER_SAMPLE: usize = 131_072;

#[test]
fn optimization_batch_fb_editor390_preserves_invalidation_stats_summary() {
    for counts in [
        [0; 9],
        [1, 2, 3, 4, 5, 6, 7, 8, 9],
        [
            u64::MAX,
            10,
            100,
            1_000,
            10_000,
            100_000,
            1_000_000,
            10_000_000,
            u64::MAX,
        ],
    ] {
        assert_eq!(
            invalidation_stats_summary(counts),
            legacy_invalidation_stats_summary(counts)
        );
    }
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_fb_editor390_direct_invalidation_stats_benchmark() {
    let counts = [17, 23, 31, 47, 59, 71, 89, 101, 127];
    for _ in 0..4 {
        black_box(measure(legacy_invalidation_stats_summary, counts));
        black_box(measure(invalidation_stats_summary, counts));
    }
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair_index in 0..SAMPLE_PAIRS {
        if pair_index % 2 == 0 {
            legacy_samples.push(measure(legacy_invalidation_stats_summary, counts));
            optimized_samples.push(measure(invalidation_stats_summary, counts));
        } else {
            optimized_samples.push(measure(invalidation_stats_summary, counts));
            legacy_samples.push(measure(legacy_invalidation_stats_summary, counts));
        }
    }

    report_performance(&legacy_samples, &optimized_samples);
}

fn legacy_invalidation_stats_summary(counts: [u64; 9]) -> String {
    format!(
        "requests={} layout={} presentation={} render={} paint_only={} hit_test={} window_metrics={} slow_path={} render_path={}",
        counts[0],
        counts[1],
        counts[2],
        counts[3],
        counts[4],
        counts[5],
        counts[6],
        counts[7],
        counts[8]
    )
}

fn measure(mut build: impl FnMut([u64; 9]) -> String, counts: [u64; 9]) -> u128 {
    let started = Instant::now();
    let mut checksum = 0_usize;
    for _ in 0..SUMMARIES_PER_SAMPLE {
        let value = black_box(build(black_box(counts)));
        checksum = checksum.wrapping_add(value.len());
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn report_performance(legacy_samples: &[u128], optimized_samples: &[u128]) {
    let legacy_p95 = nearest_rank_p95(legacy_samples);
    let optimized_p95 = nearest_rank_p95(optimized_samples);
    let improvement_percent =
        legacy_p95.saturating_sub(optimized_p95).saturating_mul(100) / legacy_p95.max(1);
    println!(
        "EDITOR390_DIRECT_INVALIDATION_STATS_BENCH_V1 sample_pairs={SAMPLE_PAIRS} summaries_per_sample={SUMMARIES_PER_SAMPLE} legacy_ns={} optimized_ns={} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} improvement_percent={improvement_percent} threshold_percent=25",
        csv(legacy_samples),
        csv(optimized_samples),
    );
    assert!(
        optimized_p95 <= legacy_p95.saturating_mul(75) / 100,
        "direct invalidation summaries must reduce P95 by at least 25%"
    );
}

fn nearest_rank_p95(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * 95).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
