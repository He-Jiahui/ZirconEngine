use std::{hint::black_box, time::Instant};

use zircon_runtime_interface::ui::surface::{
    UiDebugTimelineFrameHandle, UiSurfaceDebugOptions, UiSurfaceDebugSnapshot,
};

use super::UiDebugTimelineStore;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826an_timeline_handle_range_preserves_retention_membership() {
    let mut store = UiDebugTimelineStore::new(3);
    let options = UiSurfaceDebugOptions::default();
    let handles = (0..5)
        .map(|_| store.capture_snapshot(UiSurfaceDebugSnapshot::default(), options.clone()))
        .collect::<Vec<_>>();

    assert!(!store.select_frame(handles[1]));
    assert!(store.select_frame(handles[2]));
    assert!(store.select_frame(handles[4]));
    assert!(!store.select_frame(UiDebugTimelineFrameHandle(6)));
    assert!(store.selected_snapshot().is_some());
    let retention = store.snapshot().retention;
    assert_eq!(retention.first_frame, Some(UiDebugTimelineFrameHandle(3)));
    assert_eq!(retention.latest_frame, Some(UiDebugTimelineFrameHandle(5)));
}

#[test]
fn optimization_batch_20260826an_timeline_membership_uses_contiguous_handle_range() {
    let source = include_str!("../timeline.rs");
    let contains = source
        .split("fn contains_handle")
        .nth(1)
        .expect("timeline handle membership")
        .split("fn frame_summary")
        .next()
        .expect("bounded membership function");
    let capture = source
        .split("pub fn capture_snapshot")
        .nth(1)
        .expect("timeline capture")
        .split("pub fn select_frame")
        .next()
        .expect("bounded capture function");

    assert!(contains.contains("self.frames.front().zip(self.frames.back())"));
    assert!(contains.contains("first.summary.handle.0 <= handle.0"));
    assert!(contains.contains("handle.0 <= last.summary.handle.0"));
    assert!(!contains.contains(".iter()"));
    assert!(!capture.contains("contains_handle(handle)"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826an_timeline_handle_range_p95() {
    const FRAMES: usize = 16_384;
    const PROBES: usize = 512;
    let mut store = UiDebugTimelineStore::new(FRAMES);
    let options = UiSurfaceDebugOptions::default();
    for _ in 0..FRAMES {
        store.capture_snapshot(UiSurfaceDebugSnapshot::default(), options.clone());
    }
    let missing = UiDebugTimelineFrameHandle((FRAMES + 1) as u64);

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(PROBES, || {
                legacy_contains_handle(&store, missing)
            }));
            optimized_ns.push(measure_ns(PROBES, || store.contains_handle(missing)));
        } else {
            optimized_ns.push(measure_ns(PROBES, || store.contains_handle(missing)));
            legacy_ns.push(measure_ns(PROBES, || {
                legacy_contains_handle(&store, missing)
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(20) <= legacy_p95_ns,
        "contiguous timeline range P95 must be at least 95% below linear membership: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "RUNTIME03_UI_DEBUG_TIMELINE_HANDLE_RANGE_BENCH_V1 frames={FRAMES} probes_per_sample={PROBES} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_first_pairs=9 optimized_first_pairs=8 legacy_frame_visits_per_sample={} optimized_boundary_reads_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        FRAMES * PROBES,
        PROBES * 2,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn legacy_contains_handle(
    store: &UiDebugTimelineStore,
    handle: UiDebugTimelineFrameHandle,
) -> bool {
    store
        .frames
        .iter()
        .any(|entry| entry.summary.handle == handle)
}

fn measure_ns(probes: usize, operation: impl Fn() -> bool) -> u128 {
    let started = Instant::now();
    for _ in 0..probes {
        assert!(!black_box(operation()));
    }
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
