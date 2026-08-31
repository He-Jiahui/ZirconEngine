use std::hint::black_box;
use std::time::Instant;

use super::{asset_refresh_stream_capacity, MAX_ASSET_REFRESH_EVENTS_PER_STREAM};

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const STREAMS_PER_BUILD: usize = 3;

#[test]
fn optimization_batch_20260826fj_editor151_capacity_tracks_empty_partial_and_full_streams() {
    assert_eq!(asset_refresh_stream_capacity(0), 0);
    assert_eq!(asset_refresh_stream_capacity(127), 127);
    assert_eq!(
        asset_refresh_stream_capacity(MAX_ASSET_REFRESH_EVENTS_PER_STREAM),
        MAX_ASSET_REFRESH_EVENTS_PER_STREAM
    );
    assert_eq!(
        asset_refresh_stream_capacity(MAX_ASSET_REFRESH_EVENTS_PER_STREAM + 1_024),
        MAX_ASSET_REFRESH_EVENTS_PER_STREAM
    );
}

#[test]
fn optimization_batch_20260826fj_editor151_all_refresh_streams_reserve_pending_counts() {
    let source = include_str!("../runtime.rs");
    assert_eq!(
        source
            .matches("Vec::with_capacity(asset_refresh_stream_capacity(")
            .count(),
        STREAMS_PER_BUILD
    );
    assert!(source.contains("self.asset_change_events.len()"));
    assert!(source.contains("self.editor_asset_change_events.pending_len()"));
    assert!(source.contains("self.resource_change_events.len()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fj_editor151_asset_refresh_stream_capacity_bench() {
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
        "EDITOR151_ASSET_REFRESH_STREAM_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} streams_per_build={STREAMS_PER_BUILD} \
events_per_stream={MAX_ASSET_REFRESH_EVENTS_PER_STREAM} legacy_reservations_per_stream=0 \
optimized_reservations_per_stream=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        for _ in 0..STREAMS_PER_BUILD {
            let mut events = if reserve {
                Vec::with_capacity(MAX_ASSET_REFRESH_EVENTS_PER_STREAM)
            } else {
                Vec::new()
            };
            for event in 0..MAX_ASSET_REFRESH_EVENTS_PER_STREAM {
                events.push(black_box(event));
            }
            checksum ^= black_box(events.len() ^ events.capacity());
        }
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
