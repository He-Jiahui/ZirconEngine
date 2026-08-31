use std::hint::black_box;
use std::time::Instant;

use super::*;

const BENCHMARK_MARKER: &str = "RUNTIME73_RESOURCE_DEPENDENCY_PATH_BUFFER_BENCH_V1";
const SAMPLE_PAIRS: usize = 17;
const TRAVERSALS_PER_SAMPLE: usize = 32;
const NESTING_DEPTH: usize = 512;

fn resource_table(uri: &str) -> Value {
    let mut table = toml::map::Map::new();
    table.insert("kind".to_string(), Value::String("image".to_string()));
    table.insert("uri".to_string(), Value::String(uri.to_string()));
    Value::Table(table)
}

fn nested_resource_value(depth: usize) -> Value {
    let mut value = resource_table("res://textures/deep-icon.png");
    for level in (0..depth).rev() {
        let mut table = toml::map::Map::new();
        table.insert(format!("level_{level:04}"), value);
        value = Value::Table(table);
    }
    value
}

fn legacy_collect_value(
    collector: &mut ResourceDependencyCollector,
    value: &Value,
    source: UiResourceDependencySource,
    path: String,
) -> Result<(), UiAssetError> {
    match value {
        Value::String(uri) if has_supported_scheme(uri) => {
            let reference = UiResourceRef {
                kind: UiResourceKind::infer_from_path_and_uri(&path, uri),
                uri: uri.clone(),
                fallback: UiResourceFallbackPolicy::default(),
            };
            collector.insert_validated(reference, source, path)?;
        }
        Value::Array(values) => {
            for (index, value) in values.iter().enumerate() {
                legacy_collect_value(collector, value, source, format!("{path}[{index}]"))?;
            }
        }
        Value::Table(table) if is_resource_table(table) => {
            let reference = parse_resource_table(table, &path)?;
            collector.insert_validated(reference, source, path)?;
        }
        Value::Table(table) => {
            for (key, value) in table {
                legacy_collect_value(collector, value, source, format!("{path}.{key}"))?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn run_legacy(value: &Value) -> usize {
    let mut collector = ResourceDependencyCollector::default();
    legacy_collect_value(
        &mut collector,
        value,
        UiResourceDependencySource::TokenValue,
        "tokens".to_string(),
    )
    .expect("legacy traversal should accept the fixture");
    collector.finish().dependencies.len()
}

fn run_optimized(value: &Value) -> usize {
    let mut collector = ResourceDependencyCollector::default();
    let mut path = "tokens".to_string();
    collector
        .collect_value(value, UiResourceDependencySource::TokenValue, &mut path)
        .expect("optimized traversal should accept the fixture");
    collector.finish().dependencies.len()
}

fn sample_ns(mut run: impl FnMut() -> usize) -> u128 {
    let started = Instant::now();
    let mut observed = 0usize;
    for _ in 0..TRAVERSALS_PER_SAMPLE {
        observed += black_box(run());
    }
    black_box(observed);
    started.elapsed().as_nanos()
}

fn percentile(samples: &mut [u128], percentile: usize) -> u128 {
    samples.sort_unstable();
    let rank = (samples.len() * percentile).div_ceil(100);
    samples[rank.saturating_sub(1)]
}

#[test]
fn optimization_batch_20260826ba_resource_dependency_path_buffer_preserves_paths() {
    let mut nested = toml::map::Map::new();
    nested.insert(
        "icon".to_string(),
        Value::String("res://textures/icon.png".to_string()),
    );
    let mut root = toml::map::Map::new();
    root.insert(
        "panel".to_string(),
        Value::Array(vec![Value::Table(nested)]),
    );
    let value = Value::Table(root);

    let mut collector = ResourceDependencyCollector::default();
    let mut path = "tokens.theme".to_string();
    collector
        .collect_value(&value, UiResourceDependencySource::TokenValue, &mut path)
        .expect("resource traversal should succeed");
    let report = collector.finish();

    assert_eq!(path, "tokens.theme");
    assert_eq!(report.dependencies.len(), 1);
    assert_eq!(report.dependencies[0].path, "tokens.theme.panel[0].icon");
    assert_eq!(
        report.dependencies[0].reference.uri,
        "res://textures/icon.png"
    );
}

#[test]
fn optimization_batch_20260826ba_resource_dependency_uses_backtracking_path_buffer() {
    let source = include_str!("../collect.rs");

    assert!(source.contains("fn push_path_segment("));
    assert!(source.contains("path.truncate(prefix_len);"));
    assert!(source.contains("write!(path, \"[{index}]\")"));
    assert!(!source.contains("format!(\"{path}.{key}\")"));
    assert!(!source.contains("format!(\"{path}[{index}]\")"));
}

#[test]
#[ignore = "managed release performance gate"]
fn optimization_batch_20260826ba_resource_dependency_path_buffer_p95() {
    let fixture = nested_resource_value(NESTING_DEPTH);
    assert_eq!(run_legacy(&fixture), run_optimized(&fixture));
    for _ in 0..4 {
        black_box(run_legacy(&fixture));
        black_box(run_optimized(&fixture));
    }

    let mut legacy_samples = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_samples.push(sample_ns(|| run_legacy(&fixture)));
            optimized_samples.push(sample_ns(|| run_optimized(&fixture)));
        } else {
            optimized_samples.push(sample_ns(|| run_optimized(&fixture)));
            legacy_samples.push(sample_ns(|| run_legacy(&fixture)));
        }
    }

    let legacy_p50 = percentile(&mut legacy_samples.clone(), 50);
    let legacy_p95 = percentile(&mut legacy_samples, 95);
    let optimized_p50 = percentile(&mut optimized_samples.clone(), 50);
    let optimized_p95 = percentile(&mut optimized_samples, 95);
    let reduction = 100.0 - (optimized_p95 as f64 * 100.0 / legacy_p95 as f64);
    println!(
        "{BENCHMARK_MARKER} legacy_p50_ns={legacy_p50} legacy_p95_ns={legacy_p95} optimized_p50_ns={optimized_p50} optimized_p95_ns={optimized_p95} reduction_percent={reduction:.3} depth={NESTING_DEPTH} traversals_per_sample={TRAVERSALS_PER_SAMPLE} sample_pairs={SAMPLE_PAIRS}"
    );

    assert!(
        optimized_p95.saturating_mul(100) <= legacy_p95.saturating_mul(20),
        "expected path-buffer P95 to be at least 80% below recursive full-path allocation; legacy={legacy_p95}ns optimized={optimized_p95}ns reduction={reduction:.3}%"
    );
}
