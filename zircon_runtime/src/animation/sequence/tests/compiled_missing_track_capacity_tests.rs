use std::hint::black_box;
use std::time::Instant;

const PERF_MARKER: &str = "RUNTIME839_ANIMATION_MISSING_TRACK_CAPACITY_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const TRACKS_PER_SAMPLE: usize = 4_096;
const PASSES_PER_SAMPLE: usize = 128;

#[test]
fn optimization_batch_20260919_runtime839_animation_missing_track_capacity_preserves_lazy_success_path(
) {
    let successful = collect_missing(TRACKS_PER_SAMPLE, 0);
    assert!(successful.is_empty());
    assert_eq!(successful.capacity(), 0);

    let missing = collect_missing(TRACKS_PER_SAMPLE, 4);
    assert_eq!(missing, vec![0, 1, 2, 3]);
    assert_eq!(missing.capacity(), TRACKS_PER_SAMPLE);
}

#[test]
fn optimization_batch_20260919_runtime839_animation_missing_track_capacity_source_contract() {
    let source = include_str!("../compiled.rs");

    assert!(source.contains("let mut missing_tracks = Vec::new();"));
    assert_eq!(
        source
            .matches("missing_tracks.reserve(source_track_capacity)")
            .count(),
        2
    );
    assert!(!source.contains("let mut missing_tracks = Vec::with_capacity"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260919_runtime839_animation_missing_track_capacity_bench() {
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
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let legacy_p99_ns = percentile(&legacy_samples, 99);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    let optimized_p99_ns = percentile(&optimized_samples, 99);
    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} tracks_per_sample={TRACKS_PER_SAMPLE} \
passes_per_sample={PASSES_PER_SAMPLE} legacy_growth_events={} optimized_growth_events=0 \
legacy_success_initial_capacity=0 optimized_success_initial_capacity=0 \
legacy_missing_initial_capacity=0 optimized_missing_initial_capacity={TRACKS_PER_SAMPLE} \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_p99_ns={legacy_p99_ns} optimized_p99_ns={optimized_p99_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        growth_events(TRACKS_PER_SAMPLE),
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(growth_events(TRACKS_PER_SAMPLE) > 0);
    assert!(legacy_p50_ns > 0 && optimized_p50_ns > 0);
    assert!(legacy_p95_ns > 0 && optimized_p95_ns > 0);
    assert!(legacy_p99_ns > 0 && optimized_p99_ns > 0);
}

fn collect_missing(source_track_capacity: usize, missing_count: usize) -> Vec<usize> {
    let mut missing_tracks = Vec::new();
    for track in 0..missing_count {
        if missing_tracks.is_empty() {
            missing_tracks.reserve(source_track_capacity);
        }
        missing_tracks.push(track);
    }
    missing_tracks
}

fn measure(preallocate_on_miss: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for pass in 0..PASSES_PER_SAMPLE {
        let mut missing_tracks = Vec::new();
        for track in 0..TRACKS_PER_SAMPLE {
            if preallocate_on_miss && missing_tracks.is_empty() {
                missing_tracks.reserve(TRACKS_PER_SAMPLE);
            }
            if track % 2 == 0 {
                missing_tracks.push(black_box(track + pass));
            }
        }
        checksum ^= black_box(missing_tracks.len() ^ missing_tracks.capacity() ^ pass);
        black_box(missing_tracks);
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn growth_events(item_count: usize) -> usize {
    let mut capacity = 0usize;
    let mut events = 0usize;
    for length in 1..=item_count {
        if length > capacity {
            capacity = if capacity == 0 {
                4
            } else {
                capacity.saturating_mul(2)
            };
            events += 1;
        }
    }
    events
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
