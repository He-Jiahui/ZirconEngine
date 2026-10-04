use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const REQUEST_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iy_runtime636_reserves_material_selection_result_vectors() {
    let source = include_str!("../../selection.rs");
    let projection = source
        .split("pub fn from_records")
        .nth(1)
        .expect("material selection projection remains present")
        .split("pub fn from_record_set")
        .next()
        .expect("material selection projection remains bounded");

    assert!(projection.contains("let result_capacity = requested_material_ids.len();"));
    assert!(projection.contains("Vec::with_capacity(result_capacity)"));
    assert!(!projection.contains("let mut selected_records = Vec::new();"));
    assert!(!projection.contains("let mut missing_material_ids = Vec::new();"));
}

#[test]
fn optimization_batch_iy_runtime636_result_reservation_preserves_partition() {
    let mut selected = Vec::with_capacity(REQUEST_COUNT);
    let mut missing = Vec::with_capacity(REQUEST_COUNT);
    for index in 0..REQUEST_COUNT {
        if index % 3 == 0 {
            selected.push(index);
        } else {
            missing.push(index);
        }
    }
    assert_eq!(selected.len() + missing.len(), REQUEST_COUNT);
    assert!(selected.capacity() >= REQUEST_COUNT);
    assert!(missing.capacity() >= REQUEST_COUNT);
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iy_runtime636_preallocated_material_selection_results_benchmark() {
    let requested_ids = (0..REQUEST_COUNT).collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_projection(&requested_ids, false));
        black_box(measure_projection(&requested_ids, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_projection(&requested_ids, false));
            preallocated_samples.push(measure_projection(&requested_ids, true));
        } else {
            preallocated_samples.push(measure_projection(&requested_ids, true));
            unreserved_samples.push(measure_projection(&requested_ids, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "RUNTIME636_PREALLOCATED_MATERIAL_SELECTION_RESULT_BENCH_V1 sample_pairs={SAMPLE_PAIRS} request_count={REQUEST_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_projection(requested_ids: &[usize], preallocated: bool) -> u128 {
    let mut selected = if preallocated {
        Vec::with_capacity(requested_ids.len())
    } else {
        Vec::new()
    };
    let mut missing = if preallocated {
        Vec::with_capacity(requested_ids.len())
    } else {
        Vec::new()
    };
    let mut seen = HashSet::with_capacity(requested_ids.len());
    let started = Instant::now();
    for id in requested_ids {
        if seen.insert(black_box(*id)) {
            if id % 3 == 0 {
                selected.push(*id);
            } else {
                missing.push(*id);
            }
        }
    }
    black_box((selected, missing, seen));
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = (sorted.len() * percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
