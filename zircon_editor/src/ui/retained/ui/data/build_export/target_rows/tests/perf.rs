use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const TARGET_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_ja_editor640_reserves_build_export_target_indexes() {
    let source = include_str!("../mod.rs");
    let projection = source
        .split("pub(super) fn build_export_target_row_nodes")
        .nth(1)
        .expect("build export target projection remains present")
        .split("fn build_export_target_list_frame")
        .next()
        .expect("build export target projection remains bounded");

    assert!(projection.contains("let target_capacity = targets_with_platform_id.len();"));
    assert_eq!(
        projection
            .matches("HashMap::with_capacity(target_capacity)")
            .count(),
        2
    );
    assert!(!projection.contains("let mut platform_counts = HashMap::new();"));
    assert!(!projection.contains("let mut target_id_counts = HashMap::new();"));
    assert!(projection.contains("let mut target_id_occurrences = HashMap::new();"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_ja_editor640_preallocated_build_export_target_index_benchmark() {
    let targets = (0..TARGET_COUNT)
        .map(|index| format!("target.synthetic.{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_indexes(&targets, false));
        black_box(measure_indexes(&targets, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_indexes(&targets, false));
            preallocated_samples.push(measure_indexes(&targets, true));
        } else {
            preallocated_samples.push(measure_indexes(&targets, true));
            unreserved_samples.push(measure_indexes(&targets, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR640_PREALLOCATED_BUILD_EXPORT_TARGET_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} target_count={TARGET_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_indexes(targets: &[String], preallocated: bool) -> u128 {
    let mut platform_counts = if preallocated {
        HashMap::with_capacity(targets.len())
    } else {
        HashMap::new()
    };
    let mut target_counts = if preallocated {
        HashMap::with_capacity(targets.len())
    } else {
        HashMap::new()
    };
    let started = Instant::now();
    for target in targets {
        *platform_counts.entry(target.as_str()).or_insert(0usize) += 1;
        *target_counts.entry(target.as_str()).or_insert(0usize) += 1;
    }
    black_box((platform_counts, target_counts));
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
