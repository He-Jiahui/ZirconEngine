use std::collections::BTreeMap;
use std::hint::black_box;
use std::time::Instant;

use toml::{map::Map, Value};

use super::{remove_path_value, set_path_value, value_map_value};

const MARKER: &str = "EDITOR187_STYLE_PATH_STREAMING_MUTATION_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const REPEATS: usize = 131_072;

#[test]
fn optimization_batch_20260826gu_editor187_style_path_stream_preserves_trim_and_cleanup() {
    let mut values = BTreeMap::new();
    set_path_value(&mut values, " layout..anchor . x ", Value::Integer(42));
    assert_eq!(
        value_map_value(&values, "layout.anchor.x").and_then(Value::as_integer),
        Some(42)
    );

    assert!(remove_path_value(&mut values, " layout..anchor . x "));
    assert!(values.is_empty());
}

#[test]
fn optimization_batch_20260826gu_editor187_style_path_mutation_uses_borrowed_stream() {
    let source = include_str!("../inspector_semantics.rs");
    let implementation = source
        .split("fn path_segments")
        .nth(1)
        .and_then(|tail| tail.split("fn selected_node").next())
        .expect("streaming path implementation");
    assert!(implementation.contains("Peekable<I>"));
    assert!(implementation.contains("fn set_path_value"));
    assert!(implementation.contains("fn remove_path_value"));
    assert!(!implementation.contains("fn split_path"));
    assert!(!implementation.contains("Vec<String>"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn optimization_batch_20260826gu_editor187_style_path_streaming_mutation_bench() {
    let mut values = BTreeMap::new();
    values.insert("layout".to_string(), Value::Table(Map::new()));
    let path = format!("layout.{}", "missing_style_segment".repeat(16));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);

    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(measure(&mut values, &path, legacy_remove_path_value));
            optimized_samples.push(measure(&mut values, &path, remove_path_value));
        } else {
            optimized_samples.push(measure(&mut values, &path, remove_path_value));
            legacy_samples.push(measure(&mut values, &path, legacy_remove_path_value));
        }
    }

    let legacy_p95_ns = p95(&mut legacy_samples);
    let optimized_p95_ns = p95(&mut optimized_samples);
    println!("{MARKER} legacy_p95_ns={legacy_p95_ns} optimized_p95_ns={optimized_p95_ns}");
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "borrowed path streaming must use at most 70% of legacy p95: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn legacy_remove_path_value(values: &mut BTreeMap<String, Value>, path: &str) -> bool {
    let path = path
        .split('.')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    legacy_remove_map_path_value(values, &path)
}

fn legacy_remove_map_path_value(values: &mut BTreeMap<String, Value>, path: &[String]) -> bool {
    let Some((first, rest)) = path.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return values.remove(first).is_some();
    }
    let Some(Value::Table(table)) = values.get_mut(first) else {
        return false;
    };
    let removed = legacy_remove_table_path_value(table, rest);
    if removed && table.is_empty() {
        let _ = values.remove(first);
    }
    removed
}

fn legacy_remove_table_path_value(values: &mut Map<String, Value>, path: &[String]) -> bool {
    let Some((first, rest)) = path.split_first() else {
        return false;
    };
    if rest.is_empty() {
        return values.remove(first).is_some();
    }
    let Some(Value::Table(table)) = values.get_mut(first) else {
        return false;
    };
    let removed = legacy_remove_table_path_value(table, rest);
    if removed && table.is_empty() {
        let _ = values.remove(first);
    }
    removed
}

fn measure(
    values: &mut BTreeMap<String, Value>,
    path: &str,
    implementation: fn(&mut BTreeMap<String, Value>, &str) -> bool,
) -> u64 {
    let started = Instant::now();
    let mut removed = 0usize;
    for _ in 0..REPEATS {
        removed += usize::from(implementation(black_box(&mut *values), black_box(path)));
    }
    black_box(removed);
    started.elapsed().as_nanos().min(u128::from(u64::MAX)) as u64
}

fn p95(samples: &mut [u64]) -> u64 {
    samples.sort_unstable();
    let index = (samples.len() * 95).div_ceil(100).saturating_sub(1);
    samples[index]
}
