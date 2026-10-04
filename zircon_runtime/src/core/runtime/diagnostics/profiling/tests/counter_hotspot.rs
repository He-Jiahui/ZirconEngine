use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

use zircon_runtime_interface::{ProfileCounterSnapshot, ProfileSnapshot};

use super::analyze_counter_hotspots;

#[test]
fn counter_hotspots_group_sort_and_track_latest() {
    let mut snapshot = ProfileSnapshot {
        session_id: "counter-test".to_string(),
        frame_budget_ms: 16.67,
        ..ProfileSnapshot::default()
    };
    snapshot.counters = vec![
        counter("runtime", "extract.rebuild_clones", 1.0, 10, Some(0)),
        counter("runtime", "extract.rebuild_clones", 2.0, 20, Some(1)),
        counter("runtime", "asset.worker.frame_completed", 4.0, 15, Some(1)),
        counter("runtime", "ignored.zero", 0.0, 30, Some(2)),
        counter("runtime", "ignored.nan", f64::NAN, 40, Some(2)),
    ];

    let report = analyze_counter_hotspots(&snapshot);

    assert_eq!(report.generated_from_counter_count, 3);
    assert_eq!(report.counters.len(), 2);
    assert_eq!(
        report.counters[0].path,
        "runtime/counter:asset.worker.frame_completed"
    );
    assert_eq!(report.counters[0].total, 4.0);
    assert_eq!(
        report.counters[1].path,
        "runtime/counter:extract.rebuild_clones"
    );
    assert_eq!(report.counters[1].count, 2);
    assert_eq!(report.counters[1].frame_count, 2);
    assert_eq!(report.counters[1].latest, 2.0);
    assert!(report
        .hints
        .iter()
        .any(|hint| hint.contains("runtime/counter:asset.worker.frame_completed")));
}

#[test]
fn optimization_batch_hy_runtime609_counter_group_keys_borrow_snapshot_text() {
    let source = include_str!("../counter_hotspot.rs");
    let implementation = source
        .split("#[cfg(test)]")
        .next()
        .expect("counter hotspot implementation");
    let key_conversion = implementation
        .split("impl<'a> From<&'a ProfileCounterSnapshot>")
        .nth(1)
        .and_then(|source| source.split("struct CounterHotspotAccumulator").next())
        .expect("borrowed counter hotspot key conversion");

    assert!(implementation.contains("struct CounterHotspotKey<'a>"));
    assert!(implementation.contains("stream: &'a str"));
    assert!(implementation.contains("name: &'a str"));
    assert!(!key_conversion.contains(".clone()"));
    assert!(!key_conversion.contains("format!("));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_hy_runtime609_counter_group_key_benchmark() {
    const GROUP_COUNT: usize = 256;
    const SAMPLES_PER_GROUP: usize = 64;
    const SAMPLE_PAIRS: usize = 17;
    let counters = (0..GROUP_COUNT)
        .flat_map(|group| {
            (0..SAMPLES_PER_GROUP).map(move |sample| {
                counter(
                    "runtime",
                    &format!("metric.{group}"),
                    1.0,
                    sample as u64,
                    Some(sample as u64),
                )
            })
        })
        .collect::<Vec<_>>();

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut borrowed_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_samples.push(measure_legacy_owned_grouping(&counters));
            borrowed_samples.push(measure_borrowed_grouping(&counters));
        } else {
            borrowed_samples.push(measure_borrowed_grouping(&counters));
            legacy_samples.push(measure_legacy_owned_grouping(&counters));
        }
    }
    legacy_samples.sort_unstable();
    borrowed_samples.sort_unstable();
    let legacy_p95_ns = legacy_samples[15];
    let borrowed_p95_ns = borrowed_samples[15];
    println!(
        "RUNTIME609_COUNTER_HOTSPOT_BORROWED_KEY_BENCH_V1 counters={} groups={} legacy_p95_ns={} borrowed_p95_ns={} target_ratio_bp=4500",
        counters.len(),
        GROUP_COUNT,
        legacy_p95_ns,
        borrowed_p95_ns,
    );
    assert!(
        borrowed_p95_ns.saturating_mul(10_000) <= legacy_p95_ns.saturating_mul(4_500),
        "borrowed grouping P95 {borrowed_p95_ns} ns exceeded 45% of legacy {legacy_p95_ns} ns"
    );
}

fn measure_legacy_owned_grouping(counters: &[ProfileCounterSnapshot]) -> u128 {
    let started = Instant::now();
    let mut groups = HashMap::<(String, String, String), usize>::new();
    for counter in black_box(counters) {
        *groups
            .entry((
                counter.stream.clone(),
                counter.name.clone(),
                format!("{}/counter:{}", counter.stream, counter.name),
            ))
            .or_default() += 1;
    }
    black_box(groups);
    started.elapsed().as_nanos().max(1)
}

fn measure_borrowed_grouping(counters: &[ProfileCounterSnapshot]) -> u128 {
    let started = Instant::now();
    let mut groups = HashMap::<(&str, &str), usize>::new();
    for counter in black_box(counters) {
        *groups
            .entry((counter.stream.as_str(), counter.name.as_str()))
            .or_default() += 1;
    }
    black_box(groups);
    started.elapsed().as_nanos().max(1)
}

fn counter(
    stream: &str,
    name: &str,
    value: f64,
    timestamp_us: u64,
    frame_index: Option<u64>,
) -> ProfileCounterSnapshot {
    ProfileCounterSnapshot {
        stream: stream.to_string(),
        name: name.to_string(),
        value,
        timestamp_us,
        frame_index,
    }
}
