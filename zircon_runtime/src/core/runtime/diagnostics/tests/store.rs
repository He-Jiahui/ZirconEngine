use std::collections::VecDeque;
use std::time::Instant;

use super::{DiagnosticMeasurement, DiagnosticSeries, DiagnosticStore};

#[test]
fn current_snapshot_omits_retained_history_and_tags() {
    let mut store = DiagnosticStore::new(4);
    for frame_index in 1..=4 {
        store.record(
            "time.frame_time",
            frame_index,
            frame_index as f64,
            Some("ms"),
            ["time", "frame"],
        );
    }

    let full = store.snapshot();
    let current = store.current_snapshot();

    assert_eq!(full.series[0].history.len(), 4);
    assert_eq!(full.series[0].subsystem_tags, ["frame", "time"]);
    assert_eq!(current.series.len(), 1);
    assert_eq!(current.series[0].path.as_str(), "time.frame_time");
    assert_eq!(current.series[0].unit.as_deref(), Some("ms"));
    assert_eq!(current.series[0].current, 4.0);
    assert_eq!(current.series[0].smoothed, full.series[0].smoothed);
    assert_eq!(current.series[0].min, Some(1.0));
    assert_eq!(current.series[0].max, Some(4.0));
}

#[test]
fn static_diagnostic_series_reuses_path_and_metadata_allocations() {
    let mut store = DiagnosticStore::new(4);
    store.record_static("time.frame_time", 1, 16.0, Some("ms"), &["time", "frame"]);

    let series = store.series.get("time.frame_time").unwrap();
    let path_ptr = store.series.keys().next().unwrap().as_str().as_ptr();
    let unit_ptr = series.unit.as_ref().unwrap().as_ptr();
    let tags_ptr = series.subsystem_tags.as_ptr();

    store.record_static("time.frame_time", 2, 17.0, Some("ms"), &["time", "frame"]);

    let series = store.series.get("time.frame_time").unwrap();
    assert_eq!(
        store.series.keys().next().unwrap().as_str().as_ptr(),
        path_ptr
    );
    assert_eq!(series.unit.as_ref().unwrap().as_ptr(), unit_ptr);
    assert_eq!(series.subsystem_tags.as_ptr(), tags_ptr);
    assert_eq!(series.history.len(), 2);
    assert_eq!(series.current, Some(17.0));
}

#[test]
fn diagnostic_store_records_history_summary_and_tags() {
    let mut store = DiagnosticStore::new(2);

    store.record("render.frame_ms", 1, 16.0, Some("ms"), ["render"]);
    store.record("render.frame_ms", 2, 20.0, Some("ms"), ["render", "frame"]);
    store.record("render.frame_ms", 3, 18.0, Some("ms"), ["render"]);

    let snapshot = store.snapshot();
    assert_eq!(snapshot.series.len(), 1);
    let series = &snapshot.series[0];
    assert_eq!(series.path.as_str(), "render.frame_ms");
    assert_eq!(series.unit.as_deref(), Some("ms"));
    assert_eq!(series.current, Some(18.0));
    assert_eq!(series.min, Some(16.0));
    assert_eq!(series.max, Some(20.0));
    assert_eq!(series.subsystem_tags, ["frame", "render"]);
    assert_eq!(series.history.len(), 2);
    assert_eq!(series.history[0].frame_index, 2);
    assert_eq!(series.history[1].value, 18.0);
}

#[test]
fn static_metadata_match_checks_same_cardinality_tags_once() {
    let source = include_str!("../store.rs");
    let method = source
        .split("fn metadata_matches(")
        .nth(1)
        .and_then(|source| source.split("fn record_measurement(").next())
        .expect("metadata matching implementation");

    assert!(method.contains("self.subsystem_tags.len() == subsystem_tags.len()"));
    assert!(method.contains("subsystem_tags.contains(&existing.as_str())"));
}

#[test]
fn static_metadata_match_preserves_duplicate_tag_compatibility() {
    let mut series = DiagnosticSeries::new(4);
    series.record(1, 16.0, Some("ms"), ["time", "frame"]);

    assert!(series.metadata_matches(Some("ms"), &["frame", "time"]));
    assert!(series.metadata_matches(Some("ms"), &["time", "frame", "time"]));
    assert!(!series.metadata_matches(Some("ms"), &["time", "other", "time"]));
}

#[test]
fn optimization_wave_20260824b_runtime03_history_eviction_keeps_bounded_capacity() {
    const HISTORY_LIMIT: usize = 64;
    const WRITES: u64 = 4_096;

    let mut series = DiagnosticSeries::new(HISTORY_LIMIT);
    for frame_index in 0..WRITES {
        series.record_measurement(frame_index, frame_index as f64);
    }

    assert_eq!(series.history.len(), HISTORY_LIMIT);
    assert!(series.history.capacity() <= HISTORY_LIMIT);
    assert_eq!(series.history.front().unwrap().frame_index, 4_032);
    assert_eq!(series.history.back().unwrap().frame_index, 4_095);
}

#[test]
fn optimization_wave_20260824b_runtime03_history_eviction_source_contract() {
    let source = include_str!("../store.rs");
    let pop = source
        .find("if self.history.len() == self.history_limit")
        .expect("history must check capacity before insertion");
    let push = source
        .find("self.history\n            .push_back")
        .expect("history must append the new measurement");

    assert!(pop < push);
    assert!(!source.contains("while self.history.len() > self.history_limit"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_wave_20260824b_runtime03_history_ring_capacity_evidence() {
    const SERIES: usize = 4_096;
    const HISTORY_LIMIT: usize = 64;
    const WRITES: usize = HISTORY_LIMIT * 2;
    const MAX_ELAPSED_NS: u128 = 2_000_000_000;

    let mut legacy = VecDeque::new();
    for frame_index in 0..WRITES {
        legacy.push_back(DiagnosticMeasurement {
            frame_index: frame_index as u64,
            value: frame_index as f64,
        });
        while legacy.len() > HISTORY_LIMIT {
            legacy.pop_front();
        }
    }
    let legacy_capacity_per_series = legacy.capacity();

    let started = Instant::now();
    let mut optimized_capacity_slots = 0usize;
    for _ in 0..SERIES {
        let mut series = DiagnosticSeries::new(HISTORY_LIMIT);
        for frame_index in 0..WRITES {
            series.record_measurement(frame_index as u64, frame_index as f64);
        }
        optimized_capacity_slots =
            optimized_capacity_slots.saturating_add(series.history.capacity());
    }
    let elapsed_ns = started.elapsed().as_nanos();
    let legacy_capacity_slots = legacy_capacity_per_series.saturating_mul(SERIES);
    let capacity_reduction_bps = legacy_capacity_slots
        .saturating_sub(optimized_capacity_slots)
        .saturating_mul(10_000)
        / legacy_capacity_slots;

    println!(
        "RUNTIME_DIAGNOSTIC_HISTORY_BENCH_V1 series={SERIES} history_limit={HISTORY_LIMIT} writes_per_series={WRITES} legacy_capacity_slots={legacy_capacity_slots} optimized_capacity_slots={optimized_capacity_slots} capacity_reduction_bps={capacity_reduction_bps} elapsed_ns={elapsed_ns} max_elapsed_ns={MAX_ELAPSED_NS}"
    );

    assert!(legacy_capacity_per_series >= HISTORY_LIMIT * 2);
    assert!(optimized_capacity_slots <= SERIES * HISTORY_LIMIT);
    assert!(capacity_reduction_bps >= 5_000);
    assert!(elapsed_ns <= MAX_ELAPSED_NS);
}
