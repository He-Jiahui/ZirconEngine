use std::collections::HashSet;
use std::hint::black_box;
use std::time::Instant;

const ITEM_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iz_editor639_reserves_menu_operation_index() {
    let source = include_str!("../extension_menu.rs");
    let projection = source
        .split("fn menu_operation_paths")
        .nth(1)
        .expect("menu operation index remains present")
        .split("#[cfg(test)]")
        .next()
        .expect("menu operation index remains bounded");

    assert!(projection.contains("let operation_capacity ="));
    assert!(projection.contains("count_menu_items"));
    assert!(projection.contains("HashSet::with_capacity(operation_capacity)"));
    assert!(projection.contains("operation_paths.insert(operation_path.clone())"));
    assert!(!projection.contains("let mut operation_paths = HashSet::new();"));
}

#[test]
fn optimization_batch_iz_editor639_menu_operation_capacity_preserves_first_path() {
    let mut paths = HashSet::with_capacity(2);
    assert!(paths.insert("editor.first"));
    assert!(!paths.insert("editor.first"));
    assert!(paths.contains("editor.first"));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iz_editor639_preallocated_menu_operation_index_benchmark() {
    let operations = (0..ITEM_COUNT)
        .map(|index| format!("editor.synthetic.operation.{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_index(&operations, false));
        black_box(measure_index(&operations, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_index(&operations, false));
            preallocated_samples.push(measure_index(&operations, true));
        } else {
            preallocated_samples.push(measure_index(&operations, true));
            unreserved_samples.push(measure_index(&operations, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR639_PREALLOCATED_MENU_OPERATION_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} item_count={ITEM_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_index(operations: &[String], preallocated: bool) -> u128 {
    let mut index = if preallocated {
        HashSet::with_capacity(operations.len())
    } else {
        HashSet::new()
    };
    let started = Instant::now();
    for operation in operations {
        black_box(index.insert(operation.as_str()));
    }
    black_box(index);
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
