use std::collections::HashMap;
use std::hint::black_box;
use std::time::Instant;

const NODE_COUNT: usize = 65_536;
const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_iz_editor638_reserves_control_id_index_from_node_bound() {
    let source = include_str!("../../preview_projection.rs");
    let projection = source
        .split("fn control_id_index")
        .nth(1)
        .expect("control ID index remains present")
        .split("#[cfg(test)]")
        .next()
        .expect("control ID index remains bounded");

    assert!(projection.contains("let (node_capacity, _) = document.iter_nodes().size_hint();"));
    assert!(projection.contains("HashMap::with_capacity(node_capacity)"));
    assert!(projection.contains("index.entry(control_id).or_insert(node)"));
    assert!(!projection.contains("let mut index = HashMap::new();"));
}

#[test]
fn optimization_batch_iz_editor638_control_id_index_preserves_first_node() {
    let mut index = HashMap::new();
    let first = "first";
    let second = "second";
    index.entry("control").or_insert(first);
    index.entry("control").or_insert(second);
    assert_eq!(index.get("control"), Some(&first));
}

#[test]
#[ignore = "release performance gate"]
fn optimization_batch_iz_editor638_preallocated_control_id_index_benchmark() {
    let nodes = (0..NODE_COUNT)
        .map(|index| format!("control.synthetic.{index:08}"))
        .collect::<Vec<_>>();
    for _ in 0..4 {
        black_box(measure_index(&nodes, false));
        black_box(measure_index(&nodes, true));
    }

    let mut unreserved_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut preallocated_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            unreserved_samples.push(measure_index(&nodes, false));
            preallocated_samples.push(measure_index(&nodes, true));
        } else {
            preallocated_samples.push(measure_index(&nodes, true));
            unreserved_samples.push(measure_index(&nodes, false));
        }
    }

    let unreserved_p95 = percentile(&unreserved_samples, 95);
    let preallocated_p95 = percentile(&preallocated_samples, 95);
    let improvement_percent = unreserved_p95
        .saturating_sub(preallocated_p95)
        .saturating_mul(100)
        / unreserved_p95.max(1);
    println!(
        "EDITOR638_PREALLOCATED_CONTROL_ID_INDEX_BENCH_V1 sample_pairs={SAMPLE_PAIRS} node_count={NODE_COUNT} unreserved_ns={} preallocated_ns={} unreserved_p95_ns={unreserved_p95} preallocated_p95_ns={preallocated_p95} improvement_percent={improvement_percent} threshold_percent=20",
        csv(&unreserved_samples),
        csv(&preallocated_samples),
    );
    assert!(preallocated_p95 <= unreserved_p95 * 80 / 100);
}

fn measure_index(nodes: &[String], preallocated: bool) -> u128 {
    let mut index = if preallocated {
        HashMap::with_capacity(nodes.len())
    } else {
        HashMap::new()
    };
    let started = Instant::now();
    for node in nodes {
        index.entry(node.as_str()).or_insert(node.as_str());
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
