use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use super::reserve_empty_report_rows;

const REPORT_ROWS: usize = 4_096;
const PERF_SAMPLE_PAIRS: usize = 21;

#[derive(Clone, Copy)]
struct ReportRow([u64; 16]);

fn nearest_rank(samples: &[Duration], percentile: usize) -> Duration {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (percentile * sorted.len()).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

#[test]
fn dynamic_scene_asset_reload_report_capacity_is_exact_and_reused() {
    let mut rows = Vec::<usize>::new();
    reserve_empty_report_rows(&mut rows, 32);
    let initial_capacity = rows.capacity();
    assert!(initial_capacity >= 32);

    rows.extend(0..32);
    reserve_empty_report_rows(&mut rows, 64);
    assert_eq!(rows.capacity(), initial_capacity);
}

#[test]
#[ignore = "managed Runtime53 performance evidence"]
fn dynamic_scene_asset_reload_runtime53_performance_report_capacity() {
    fn collect_legacy() -> (usize, usize) {
        let mut rows = Vec::new();
        let mut growths = 0usize;
        for index in 0..REPORT_ROWS {
            growths += usize::from(rows.len() == rows.capacity());
            rows.push(ReportRow([index as u64; 16]));
        }
        (
            black_box(rows.len() + rows[REPORT_ROWS - 1].0[0] as usize),
            growths,
        )
    }

    fn collect_reserved() -> (usize, usize) {
        let mut rows = Vec::new();
        reserve_empty_report_rows(&mut rows, REPORT_ROWS);
        let mut growths = 0usize;
        for index in 0..REPORT_ROWS {
            growths += usize::from(rows.len() == rows.capacity());
            rows.push(ReportRow([index as u64; 16]));
        }
        (
            black_box(rows.len() + rows[REPORT_ROWS - 1].0[0] as usize),
            growths,
        )
    }

    let (_, legacy_growths) = collect_legacy();
    let (_, reserved_growths) = collect_reserved();
    assert!(legacy_growths > 0);
    assert_eq!(reserved_growths, 0);
    black_box(collect_legacy());
    black_box(collect_reserved());

    let mut legacy_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    let mut reserved_samples = Vec::with_capacity(PERF_SAMPLE_PAIRS);
    for pair in 0..PERF_SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            black_box(collect_legacy());
            legacy_samples.push(started.elapsed());
        };
        let mut measure_reserved = || {
            let started = Instant::now();
            black_box(collect_reserved());
            reserved_samples.push(started.elapsed());
        };
        if pair % 2 == 0 {
            measure_legacy();
            measure_reserved();
        } else {
            measure_reserved();
            measure_legacy();
        }
    }

    let legacy_p50 = nearest_rank(&legacy_samples, 50);
    let legacy_p95 = nearest_rank(&legacy_samples, 95);
    let reserved_p50 = nearest_rank(&reserved_samples, 50);
    let reserved_p95 = nearest_rank(&reserved_samples, 95);
    let legacy_ns = legacy_samples
        .iter()
        .map(Duration::as_nanos)
        .collect::<Vec<_>>();
    let reserved_ns = reserved_samples
        .iter()
        .map(Duration::as_nanos)
        .collect::<Vec<_>>();
    let legacy_csv = legacy_ns
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");
    let reserved_csv = reserved_ns
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",");

    eprintln!(
        "RUNTIME53_REPORT_CAPACITY_BENCH_V1 report_rows={REPORT_ROWS} sample_pairs={PERF_SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_growths={legacy_growths} reserved_growths={reserved_growths} legacy_p50_ns={} legacy_p95_ns={} reserved_p50_ns={} reserved_p95_ns={} legacy_ns={legacy_csv} reserved_ns={reserved_csv}",
        legacy_p50.as_nanos(),
        legacy_p95.as_nanos(),
        reserved_p50.as_nanos(),
        reserved_p95.as_nanos(),
    );
    assert!(
        reserved_p95.as_nanos().saturating_mul(100) <= legacy_p95.as_nanos().saturating_mul(75),
        "reserved report rows must reduce P95 by at least 25%: legacy={legacy_p95:?}, reserved={reserved_p95:?}"
    );
}
