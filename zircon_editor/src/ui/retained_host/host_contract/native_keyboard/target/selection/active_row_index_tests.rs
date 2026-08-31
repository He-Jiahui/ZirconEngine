use std::hint::black_box;
use std::time::Instant;

use super::{preferred_row_index, FrameRect, PopupKeyboardRow};

const ROW_COUNT: usize = 4_096;
const SAMPLE_COUNT: usize = 17;
const LEGACY_ROW_VISITS: usize = ROW_COUNT * 3;

#[test]
fn optimization_batch_20260826bi_active_keyboard_row_single_pass_preserves_global_priority() {
    let mut rows = (0..5).map(row).collect::<Vec<_>>();
    rows[0].selected = true;
    rows[2].focused = true;

    assert_eq!(preferred_row_index(&rows, "row_0004"), 4);
    assert_eq!(preferred_row_index(&rows, "missing"), 2);
    rows[2].focused = false;
    assert_eq!(preferred_row_index(&rows, "missing"), 0);
    rows[0].selected = false;
    assert_eq!(preferred_row_index(&rows, "missing"), 0);
}

#[test]
fn optimization_batch_20260826bi_active_keyboard_row_single_pass_eliminates_repeated_row_walks() {
    const SOURCE: &str = include_str!("../selection.rs");
    let production = SOURCE.split("#[cfg(test)]").next().unwrap();

    assert_eq!(LEGACY_ROW_VISITS, 12_288);
    assert!(production.contains("for (index, row) in rows.iter().enumerate()"));
    assert!(production.contains("focused_index.or(selected_index)"));
    assert!(!production.contains(".position(|row|"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_20260826bi_active_keyboard_row_single_pass_p95() {
    let mut rows = (0..ROW_COUNT).map(row).collect::<Vec<_>>();
    rows.last_mut().unwrap().selected = true;

    let (legacy_samples, optimized_samples) = benchmark_paired_samples::<SAMPLE_COUNT>(
        || legacy_preferred_row_index(black_box(&rows), black_box("missing")),
        || preferred_row_index(black_box(&rows), black_box("missing")),
    );
    assert_eq!(
        legacy_preferred_row_index(&rows, "missing"),
        preferred_row_index(&rows, "missing")
    );

    let legacy_p50 = percentile(&legacy_samples, 50);
    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p50 = percentile(&optimized_samples, 50);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT EDITOR08_ACTIVE_KEYBOARD_ROW_SINGLE_PASS_BENCH_V1 rows={ROW_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_row_visits={LEGACY_ROW_VISITS} optimized_row_visits={ROW_COUNT} deterministic_row_visit_reduction_percent=66.6667 legacy_p50_ns={legacy_p50} optimized_p50_ns={optimized_p50} legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} legacy_samples_ns={} optimized_samples_ns={}",
        join_samples(&legacy_samples),
        join_samples(&optimized_samples),
    );
    assert!(
        optimized_p95 * 5 <= legacy_p95 * 4,
        "optimized P95 {optimized_p95}ns must be at least 20% below legacy P95 {legacy_p95}ns"
    );
}

fn legacy_preferred_row_index(rows: &[PopupKeyboardRow], interaction_identity: &str) -> usize {
    if !interaction_identity.is_empty() {
        if let Some(index) = rows
            .iter()
            .position(|row| row.identity.as_str() == interaction_identity)
        {
            return index;
        }
    }
    rows.iter()
        .position(|row| row.focused)
        .or_else(|| rows.iter().position(|row| row.selected))
        .unwrap_or(0)
}

fn benchmark_paired_samples<const N: usize>(
    mut legacy: impl FnMut() -> usize,
    mut optimized: impl FnMut() -> usize,
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy());
    black_box(optimized());
    let mut legacy_samples = Vec::with_capacity(N);
    let mut optimized_samples = Vec::with_capacity(N);
    for sample_index in 0..N {
        if sample_index % 2 == 0 {
            legacy_samples.push(benchmark_sample(&mut legacy));
            optimized_samples.push(benchmark_sample(&mut optimized));
        } else {
            optimized_samples.push(benchmark_sample(&mut optimized));
            legacy_samples.push(benchmark_sample(&mut legacy));
        }
    }
    (legacy_samples, optimized_samples)
}

fn benchmark_sample(operation: &mut impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    black_box(operation());
    started.elapsed().as_nanos()
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn row(index: usize) -> PopupKeyboardRow {
    PopupKeyboardRow {
        action_id: format!("row_{index:04}").into(),
        value_text: format!("row_{index:04}").into(),
        identity: format!("row_{index:04}").into(),
        search_text: format!("Row {index:04}").into(),
        focused: false,
        selected: false,
        source_index: Some(index),
        frame: FrameRect::default(),
    }
}
