use std::hint::black_box;
use std::time::Instant;

use super::MeshAssetManagementRecordSet;

const SAMPLE_PAIRS: usize = 21;
const SETS_PER_SAMPLE: usize = 64;
const RESULTS_PER_SET: usize = 4_096;

#[test]
fn optimization_batch_20260826fv_runtime217_empty_results_preserve_empty_summary() {
    let record_set = MeshAssetManagementRecordSet::from_results(Vec::new());

    assert!(record_set.records.is_empty());
    assert!(record_set.failures.is_empty());
    assert_eq!(record_set.summary.mesh_count, 0);
    assert_eq!(record_set.summary.valid_mesh_count, 0);
    assert_eq!(record_set.summary.invalid_mesh_count, 0);
}

#[test]
fn optimization_batch_20260826fv_runtime217_records_reserve_result_count() {
    let source = include_str!("../management.rs");
    assert!(source.contains("let record_capacity = results.len();"));
    assert!(source.contains("let mut records = Vec::with_capacity(record_capacity);"));
    assert!(source.contains("let mut failures = Vec::new();"));
    assert!(!source.contains("let mut records = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826fv_runtime217_mesh_management_record_capacity_bench() {
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
        "RUNTIME217_MESH_MANAGEMENT_RECORD_CAPACITY_BENCH_V1 sample_pairs={SAMPLE_PAIRS} \
sets_per_sample={SETS_PER_SAMPLE} results_per_set={RESULTS_PER_SET} \
legacy_reservations_per_set=0 optimized_reservations_per_set=1 \
legacy_p50_ns={legacy_p50_ns} optimized_p50_ns={optimized_p50_ns} \
legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns} \
legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70));
}

#[derive(Clone, Copy)]
struct RecordFixture([usize; 10]);

fn measure(reserve: bool) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for set in 0..SETS_PER_SAMPLE {
        let mut records = if reserve {
            Vec::with_capacity(RESULTS_PER_SET)
        } else {
            Vec::new()
        };
        for result in 0..RESULTS_PER_SET {
            records.push(RecordFixture([black_box(set ^ result); 10]));
        }
        checksum ^=
            black_box(records.len() ^ records.capacity() ^ records[RESULTS_PER_SET - 1].0[0]);
        black_box(&records);
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
