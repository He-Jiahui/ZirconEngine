use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::*;

const ROW_COUNT: usize = 16 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 128;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hn_editor206_boxes_owned_popup_rows() {
    let rows = (0..128).collect::<Vec<_>>();
    let allocation = rows.as_ptr();
    let boxed = boxed_popup_rows(rows);

    assert_eq!(boxed.as_ref(), &(0..128).collect::<Vec<_>>());
    assert_eq!(boxed.as_ptr(), allocation);
}

#[test]
fn optimization_batch_20260826hn_editor206_uses_non_shared_boxed_row_storage() {
    let source = include_str!("../pane_index.rs");

    assert!(source.contains("popup_rows: Box<[usize]>"));
    assert!(source.contains("boxed_popup_rows("));
    assert!(source.contains("rows.into_boxed_slice()"));
    assert!(!source.contains("popup_rows: Arc<[usize]>"));
    assert!(!source.contains("collect::<Vec<_>>()\n            .into()"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hn_editor206_boxed_popup_row_index_release_benchmark() {
    let source = (0..ROW_COUNT).collect::<Vec<_>>();

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(legacy_popup_rows(black_box(&source)));
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for _ in 0..OPERATIONS_PER_SAMPLE {
                black_box(optimized_popup_rows(black_box(&source)));
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "EDITOR206_BOXED_POPUP_ROW_INDEX_BENCH_V1 \
         row_count={ROW_COUNT} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_popup_rows(source: &[usize]) -> Arc<[usize]> {
    source.iter().copied().collect::<Vec<_>>().into()
}

fn optimized_popup_rows(source: &[usize]) -> Box<[usize]> {
    boxed_popup_rows(source.to_vec())
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
