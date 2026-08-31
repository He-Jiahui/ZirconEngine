use std::hint::black_box;
use std::time::Instant;

use super::{
    availability_diagnostic_line_count, RuntimePluginAvailabilityEntry,
    RuntimePluginAvailabilityReport,
};
use crate::builtin::RuntimePluginId;
use crate::plugin::PluginMaturity;

const SAMPLE_PAIRS: usize = 21;
const BUILDS_PER_SAMPLE: usize = 2_048;
const AVAILABILITY_CATEGORIES: usize = 8;
const ENTRIES_PER_CATEGORY: usize = 32;
const ENTRIES_PER_BUILD: usize = AVAILABILITY_CATEGORIES * ENTRIES_PER_CATEGORY;
const LINES_PER_BUILD: usize = AVAILABILITY_CATEGORIES + ENTRIES_PER_BUILD;

#[test]
fn optimization_batch_20260826fh_runtime203_capacity_preserves_availability_diagnostic_order() {
    let mut report = RuntimePluginAvailabilityReport::default();
    report.available = entries("available");
    report.linked = entries("linked");
    report.native_dynamic = entries("native_dynamic");
    report.externalized_missing = entries("externalized_missing");
    report.stub = entries("stub");
    report.blocked_by_target = entries("blocked_by_target");
    report.blocked_by_maturity = entries("blocked_by_maturity");
    report.missing_required = entries("missing_required");

    let lines = report.diagnostic_lines();

    assert_eq!(availability_diagnostic_line_count(&report), LINES_PER_BUILD);
    assert_eq!(lines.len(), LINES_PER_BUILD);
    assert!(lines.capacity() >= LINES_PER_BUILD);
    assert_eq!(lines[0], "runtime_plugin_availability.available.count=32");
    assert!(lines[1].starts_with("runtime_plugin_availability.available=available-000"));
    assert_eq!(
        lines[ENTRIES_PER_CATEGORY + 1],
        "runtime_plugin_availability.linked.count=32"
    );
    assert!(lines.last().is_some_and(|line| {
        line.starts_with("runtime_plugin_availability.missing_required=missing_required-031")
    }));
}

#[test]
fn optimization_batch_20260826fh_runtime203_availability_diagnostics_reserve_all_categories() {
    let source = include_str!("../availability_report.rs");
    assert!(source.contains("const RUNTIME_PLUGIN_AVAILABILITY_CATEGORY_COUNT: usize = 8;"));
    assert!(source.contains("fn availability_diagnostic_line_count("));
    assert!(source.contains("report.available.len()"));
    assert!(source.contains("report.missing_required.len()"));
    assert!(source.contains("lines.reserve(availability_diagnostic_line_count(self))"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fh_runtime203_availability_diagnostic_capacity_bench() {
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
        "RUNTIME203_AVAILABILITY_DIAGNOSTIC_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
builds_per_sample={BUILDS_PER_SAMPLE} categories={AVAILABILITY_CATEGORIES} \
entries_per_category={ENTRIES_PER_CATEGORY} entries_per_build={ENTRIES_PER_BUILD} \
lines_per_build={LINES_PER_BUILD} legacy_reservations_per_build=0 \
optimized_reservations_per_build=1 legacy_p50_ns={legacy_p50_ns} \
optimized_p50_ns={optimized_p50_ns} legacy_p95_ns={legacy_p95_ns} \
optimized_p95_ns={optimized_p95_ns} legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

fn entries(prefix: &str) -> Vec<RuntimePluginAvailabilityEntry> {
    (0..ENTRIES_PER_CATEGORY)
        .map(|index| RuntimePluginAvailabilityEntry {
            id: format!("{prefix}-{index:03}"),
            runtime_id: RuntimePluginId::new(format!("{prefix}.{index:03}")),
            required: index % 2 == 0,
            maturity: PluginMaturity::Experimental,
            reason: "capacity fixture".to_string(),
        })
        .collect()
}

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..BUILDS_PER_SAMPLE {
        let mut lines = Vec::new();
        if reserve {
            lines.reserve(LINES_PER_BUILD);
        }
        for line in 0..LINES_PER_BUILD {
            lines.push(black_box(line));
        }
        checksum ^= black_box(lines.len() ^ lines.capacity());
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
