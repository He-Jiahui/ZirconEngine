use std::{collections::BTreeMap, hint::black_box, time::Instant};

use super::*;

const SAMPLE_PAIRS: usize = 17;

#[test]
fn optimization_batch_20260826az_style_declaration_path_buffer_preserves_flattened_order() {
    let block = UiStyleDeclarationBlock {
        self_values: BTreeMap::from([
            ("alpha".to_string(), Value::Float(0.5)),
            (
                "layout".to_string(),
                Value::Table(Map::from_iter([("width".to_string(), Value::Integer(320))])),
            ),
        ]),
        slot: BTreeMap::from([("padding".to_string(), Value::Integer(8))]),
    };

    assert_eq!(
        declaration_entries(&block),
        vec![
            UiStyleRuleDeclarationEntry {
                path: "self.alpha".to_string(),
                literal: "0.5".to_string(),
            },
            UiStyleRuleDeclarationEntry {
                path: "self.layout.width".to_string(),
                literal: "320".to_string(),
            },
            UiStyleRuleDeclarationEntry {
                path: "slot.padding".to_string(),
                literal: "8".to_string(),
            },
        ]
    );
}

#[test]
fn optimization_batch_20260826az_style_declaration_uses_backtracking_path_buffer() {
    let source = include_str!("../style_rule_declarations.rs");
    let collection = bounded_source(source, "fn collect_map_entries(", "fn set_in_value_map(");

    assert!(collection.contains("path.push('.')"));
    assert!(collection.contains("path.push_str(key)"));
    assert!(collection.contains("path.truncate(prefix_len)"));
    assert!(collection.contains("path: path.clone()"));
    assert!(!collection.contains("format!(\"{prefix}.{key}\")"));
    assert!(!collection.contains("format!(\"{path}.{key}\")"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826az_style_declaration_path_buffer_p95() {
    const PATH_DEPTH: usize = 1_024;
    const BUILDS: usize = 64;
    let block = deep_block(PATH_DEPTH);
    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);

    for sample_index in 0..SAMPLE_PAIRS {
        if sample_index % 2 == 0 {
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_declaration_entries(black_box(&block)).len()
            }));
            optimized_ns.push(measure_ns(BUILDS, || {
                declaration_entries(black_box(&block)).len()
            }));
        } else {
            optimized_ns.push(measure_ns(BUILDS, || {
                declaration_entries(black_box(&block)).len()
            }));
            legacy_ns.push(measure_ns(BUILDS, || {
                legacy_declaration_entries(black_box(&block)).len()
            }));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(3),
        "backtracking declaration path buffer P95 must be at least 70% below recursive full-path allocation: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );

    println!(
        "EDITOR23_STYLE_DECLARATION_PATH_BUFFER_BENCH_V1 path_depth={PATH_DEPTH} builds_per_sample={BUILDS} sample_pairs={SAMPLE_PAIRS} pair_order=alternating_legacy_even legacy_intermediate_path_allocations_per_sample={} optimized_path_buffers_per_sample={BUILDS} legacy_output_path_clones_per_sample={BUILDS} optimized_output_path_clones_per_sample={BUILDS} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        (PATH_DEPTH + 1) * BUILDS,
        join_samples(&legacy_ns),
        join_samples(&optimized_ns),
    );
}

fn deep_block(depth: usize) -> UiStyleDeclarationBlock {
    let mut value = Value::String("leaf".to_string());
    for index in (0..depth).rev() {
        value = Value::Table(Map::from_iter([(format!("segment-{index:04}"), value)]));
    }
    UiStyleDeclarationBlock {
        self_values: BTreeMap::from([("root".to_string(), value)]),
        slot: BTreeMap::new(),
    }
}

fn legacy_declaration_entries(block: &UiStyleDeclarationBlock) -> Vec<UiStyleRuleDeclarationEntry> {
    let mut entries = Vec::new();
    legacy_collect_map_entries(&mut entries, "self", &block.self_values);
    legacy_collect_map_entries(&mut entries, "slot", &block.slot);
    entries
}

fn legacy_collect_map_entries(
    output: &mut Vec<UiStyleRuleDeclarationEntry>,
    prefix: &str,
    values: &BTreeMap<String, Value>,
) {
    for (key, value) in values {
        legacy_collect_value_entries(output, &format!("{prefix}.{key}"), value);
    }
}

fn legacy_collect_value_entries(
    output: &mut Vec<UiStyleRuleDeclarationEntry>,
    path: &str,
    value: &Value,
) {
    match value {
        Value::Table(table) if !table.is_empty() => {
            for (key, child) in table {
                legacy_collect_value_entries(output, &format!("{path}.{key}"), child);
            }
        }
        _ => output.push(UiStyleRuleDeclarationEntry {
            path: path.to_string(),
            literal: value.to_string(),
        }),
    }
}

fn measure_ns(iterations: usize, mut operation: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut checksum = 0usize;
    for _ in 0..iterations {
        checksum = checksum.wrapping_add(black_box(operation()));
    }
    black_box(checksum);
    started.elapsed().as_nanos()
}

fn bounded_source<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split(start)
        .nth(1)
        .expect("source start")
        .split(end)
        .next()
        .expect("source end")
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}

fn join_samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
