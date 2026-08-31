use std::collections::VecDeque;
use std::hint::black_box;
use std::time::Instant;

use super::{HostWindowDiagnostic, HostWindowDiagnosticQueue, HostWindowDiagnosticSeverity};

const SAMPLE_PAIRS: usize = 21;
const DRAINS_PER_SAMPLE: usize = 8_192;
const DIAGNOSTICS_PER_DRAIN: usize = 64;

type BenchmarkDiagnostic = [usize; 4];

#[test]
fn optimization_batch_20260826ek_editor126_drain_preserves_eviction_report_and_resets_it() {
    let mut queue = HostWindowDiagnosticQueue::default();
    for index in 0..=DIAGNOSTICS_PER_DRAIN {
        queue.push(HostWindowDiagnostic::new(
            HostWindowDiagnosticSeverity::Info,
            format!("editor126 diagnostic {index}"),
        ));
    }

    let diagnostics = queue.drain();

    assert_eq!(diagnostics.len(), DIAGNOSTICS_PER_DRAIN + 1);
    assert_eq!(diagnostics[0].message(), "editor126 diagnostic 1");
    assert_eq!(
        diagnostics.last().unwrap().message(),
        "editor_host_window diagnostics_dropped=1"
    );
    assert!(queue.drain().is_empty());
}

#[test]
fn optimization_batch_20260826ek_editor126_drain_reserves_the_warning_slot_up_front() {
    let source = include_str!("../host_window.rs");
    let drain_start = source
        .find("pub(crate) fn drain(&mut self)")
        .expect("diagnostic drain should exist");
    let drain_end = source[drain_start..]
        .find("fn bounded_diagnostic")
        .map(|offset| drain_start + offset)
        .unwrap();
    let drain_source = &source[drain_start..drain_end];

    assert!(drain_source.contains("std::mem::take(&mut self.dropped_entries)"));
    assert!(drain_source.contains("Vec::with_capacity("));
    assert!(drain_source.contains("usize::from(dropped_entries != 0)"));
    assert!(drain_source.contains("diagnostics.extend(self.entries.drain(..))"));
    assert!(!drain_source.contains(".collect::<Vec<_>>()"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826ek_editor126_host_window_drain_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(legacy_drain));
            optimized_samples.push(measure(capacity_aware_drain));
        } else {
            optimized_samples.push(measure(capacity_aware_drain));
            legacy_samples.push(measure(legacy_drain));
        }
    }
    let legacy_p50_ns = percentile(&legacy_samples, 50);
    let optimized_p50_ns = percentile(&optimized_samples, 50);
    let legacy_p95_ns = percentile(&legacy_samples, 95);
    let optimized_p95_ns = percentile(&optimized_samples, 95);
    println!(
        "EDITOR126_HOST_WINDOW_DIAGNOSTIC_DRAIN_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
drains_per_sample={DRAINS_PER_SAMPLE} diagnostics_per_drain={DIAGNOSTICS_PER_DRAIN} \
legacy_output_allocations_per_drain=2 optimized_output_allocations_per_drain=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "capacity-aware diagnostic drain P95 {optimized_p95_ns}ns must be at most 70% of collect-then-grow P95 {legacy_p95_ns}ns"
    );
}

fn legacy_drain(entries: &mut VecDeque<BenchmarkDiagnostic>) -> Vec<BenchmarkDiagnostic> {
    let mut diagnostics = entries.drain(..).collect::<Vec<_>>();
    diagnostics.push([usize::MAX; 4]);
    diagnostics
}

fn capacity_aware_drain(entries: &mut VecDeque<BenchmarkDiagnostic>) -> Vec<BenchmarkDiagnostic> {
    let mut diagnostics = Vec::with_capacity(entries.len() + 1);
    diagnostics.extend(entries.drain(..));
    diagnostics.push([usize::MAX; 4]);
    diagnostics
}

fn measure(drain: fn(&mut VecDeque<BenchmarkDiagnostic>) -> Vec<BenchmarkDiagnostic>) -> u128 {
    let mut entries = benchmark_fixture();
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..DRAINS_PER_SAMPLE {
        let diagnostics = black_box(drain(black_box(&mut entries)));
        checksum ^= diagnostics[0][0] ^ diagnostics[diagnostics.len() - 1][0];
        drop(diagnostics);
        entries.extend(
            (0..DIAGNOSTICS_PER_DRAIN).map(|index| [index, index + 1, index + 2, index + 3]),
        );
    }
    black_box(checksum);
    started.elapsed().as_nanos().max(1)
}

fn benchmark_fixture() -> VecDeque<BenchmarkDiagnostic> {
    (0..DIAGNOSTICS_PER_DRAIN)
        .map(|index| [index, index + 1, index + 2, index + 3])
        .collect()
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
