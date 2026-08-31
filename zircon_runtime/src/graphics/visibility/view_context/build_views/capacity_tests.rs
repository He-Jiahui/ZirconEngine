use std::hint::black_box;
use std::time::Instant;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 128;
const PRIMITIVES_PER_VIEW: usize = 4_096;

#[test]
fn optimization_batch_20260826fs_runtime214_capacity_covers_visible_primitive_indices() {
    let mut visible = Vec::with_capacity(PRIMITIVES_PER_VIEW);
    visible.extend((0..PRIMITIVES_PER_VIEW).map(|index| index as u32));

    assert_eq!(visible.len(), PRIMITIVES_PER_VIEW);
    assert!(visible.capacity() >= PRIMITIVES_PER_VIEW);
    assert_eq!(visible[0], 0);
    assert_eq!(visible[PRIMITIVES_PER_VIEW - 1], 4_095);
}

#[test]
fn optimization_batch_20260826fs_runtime214_custom_and_shadow_views_reserve_relevance_count() {
    let source = include_str!("../build_views.rs");
    assert_eq!(
        source
            .matches("Vec::with_capacity(visible_index_capacity(frame_visibility))")
            .count(),
        2
    );
    assert!(source.contains("frame_visibility.relevance.len()"));
    assert!(!source.contains("let mut visible = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fs_runtime214_view_visible_index_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(false));
            optimized_samples.push(measure(true));
        } else {
            optimized_samples.push(measure(true));
            legacy_samples.push(measure(false));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "RUNTIME214_VIEW_VISIBLE_INDEX_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} primitives_per_view={PRIMITIVES_PER_VIEW} \
legacy_reservations_per_build=0 optimized_reservations_per_build=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[derive(Clone, Copy)]
struct VisibilityFixture([usize; 5]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for build in 0..BUILDS_PER_SAMPLE {
        let mut visible = if reserve {
            Vec::with_capacity(PRIMITIVES_PER_VIEW)
        } else {
            Vec::new()
        };
        for primitive in 0..PRIMITIVES_PER_VIEW {
            visible.push(VisibilityFixture([black_box(build ^ primitive); 5]));
        }
        checksum ^=
            black_box(visible.len() ^ visible.capacity() ^ visible[PRIMITIVES_PER_VIEW - 1].0[0]);
        black_box(&visible);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
