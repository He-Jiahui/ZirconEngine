use std::collections::{BTreeMap, HashMap};
use std::hint::black_box;
use std::sync::Arc;
use std::time::Instant;

use super::WorldInspectionFieldsArtifact;
use crate::scene::inspection::WorldInspectionField;
use zircon_runtime_interface::reflect::ReflectedValue;

const PERF_MARKER: &str = "RUNTIME137_INSPECTION_FIELD_HASH_DELTA_BENCH_V1";

#[test]
fn optimization_batch_20260826ct_runtime_inspection_hash_delta_preserves_slice_order() {
    let previous = artifact(
        3,
        vec![
            field("Removed", "gone", "old"),
            field("Stable", "same", "value"),
            field("Changed", "value", "old"),
        ],
    );
    let current = artifact(
        4,
        vec![
            field("Stable", "same", "value"),
            field("Changed", "value", "new"),
            field("Added", "fresh", "new"),
        ],
    );

    let delta = current.delta_from(&previous);

    assert_eq!(delta.previous_generation(), 3);
    assert_eq!(delta.generation(), 4);
    assert_eq!(delta.entity(), 77);
    assert_eq!(
        delta
            .changed_fields()
            .iter()
            .map(|field| field.component_type_path.as_str())
            .collect::<Vec<_>>(),
        ["Changed", "Added"]
    );
    assert_eq!(delta.removed_fields().len(), 1);
    assert_eq!(delta.removed_fields()[0].component_type_path(), "Removed");
    assert_eq!(delta.removed_fields()[0].field_name(), "gone");
}

#[test]
fn optimization_batch_20260826ct_runtime_inspection_hash_delta_source_contract() {
    let source = include_str!("../fields.rs");
    let production = source
        .split("#[cfg(test)]")
        .next()
        .expect("inspection field artifact production source");

    assert!(production.contains("use std::collections::HashMap;"));
    assert_eq!(production.matches("collect::<HashMap<_, _>>()").count(), 2);
    assert!(!production.contains("BTreeMap"));
    assert_eq!(
        PERF_MARKER,
        "RUNTIME137_INSPECTION_FIELD_HASH_DELTA_BENCH_V1"
    );
}

#[test]
#[ignore = "release-only paired P95 performance evidence"]
fn optimization_batch_20260826ct_runtime_inspection_hash_delta_p95() {
    const SAMPLE_PAIRS: usize = 21;
    const FIELD_COUNT: usize = 16_384;
    let previous = fields(FIELD_COUNT, false);
    let current = fields(FIELD_COUNT, true);

    black_box(measure_legacy(&previous, &current));
    black_box(measure_optimized(&previous, &current));

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for pair in 0..SAMPLE_PAIRS {
        if pair % 2 == 0 {
            legacy_ns.push(measure_legacy(&previous, &current));
            optimized_ns.push(measure_optimized(&previous, &current));
        } else {
            optimized_ns.push(measure_optimized(&previous, &current));
            legacy_ns.push(measure_legacy(&previous, &current));
        }
    }

    let legacy_p50_ns = nearest_rank(&legacy_ns, 50);
    let legacy_p95_ns = nearest_rank(&legacy_ns, 95);
    let optimized_p50_ns = nearest_rank(&optimized_ns, 50);
    let optimized_p95_ns = nearest_rank(&optimized_ns, 95);
    let reduction = 100.0 * (legacy_p95_ns.saturating_sub(optimized_p95_ns)) as f64
        / legacy_p95_ns.max(1) as f64;

    println!(
        "{PERF_MARKER} sample_pairs={SAMPLE_PAIRS} fields={FIELD_COUNT} order=alternating_legacy_first_even legacy_tree_admissions={} optimized_hash_admissions={} lookups_per_sample={} legacy_p50_ns={legacy_p50_ns} legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} optimized_p95_ns={optimized_p95_ns} p95_reduction_percent={reduction:.2}",
        FIELD_COUNT * 2,
        FIELD_COUNT * 2,
        FIELD_COUNT * 2
    );
    assert!(
        optimized_p95_ns.saturating_mul(10) <= legacy_p95_ns.saturating_mul(7),
        "hash inspection-field delta must reduce P95 by at least 30%: legacy={legacy_p95_ns}ns optimized={optimized_p95_ns}ns"
    );
}

fn artifact(generation: u64, fields: Vec<WorldInspectionField>) -> WorldInspectionFieldsArtifact {
    WorldInspectionFieldsArtifact {
        generation,
        entity: 77,
        fields: Arc::from(fields),
    }
}

fn field(component: &str, name: &str, value: &str) -> WorldInspectionField {
    WorldInspectionField {
        component_type_path: component.to_owned(),
        component_display_name: component.to_owned(),
        field_name: name.to_owned(),
        field_display_name: name.to_owned(),
        value_type_path: "string".to_owned(),
        value: ReflectedValue::String(value.to_owned()),
        writable: true,
        serializable: true,
        plugin_owned: false,
    }
}

fn fields(count: usize, changed: bool) -> Vec<(String, String, u64)> {
    (0..count)
        .map(|index| {
            (
                format!("component.{:05}", index % 256),
                format!("field.{index:05}"),
                index as u64 + u64::from(changed && index % 8 == 0),
            )
        })
        .collect()
}

fn measure_legacy(previous: &[(String, String, u64)], current: &[(String, String, u64)]) -> u128 {
    let started = Instant::now();
    let previous_index = previous
        .iter()
        .map(|(component, field, value)| ((component.as_str(), field.as_str()), *value))
        .collect::<BTreeMap<_, _>>();
    let current_index = current
        .iter()
        .map(|(component, field, value)| ((component.as_str(), field.as_str()), *value))
        .collect::<BTreeMap<_, _>>();
    let changed = current
        .iter()
        .filter(|(component, field, value)| {
            previous_index.get(&(component.as_str(), field.as_str())) != Some(value)
        })
        .count();
    let removed = previous
        .iter()
        .filter(|(component, field, _)| {
            !current_index.contains_key(&(component.as_str(), field.as_str()))
        })
        .count();
    black_box((previous_index, current_index, changed, removed));
    started.elapsed().as_nanos()
}

fn measure_optimized(
    previous: &[(String, String, u64)],
    current: &[(String, String, u64)],
) -> u128 {
    let started = Instant::now();
    let previous_index = previous
        .iter()
        .map(|(component, field, value)| ((component.as_str(), field.as_str()), *value))
        .collect::<HashMap<_, _>>();
    let current_index = current
        .iter()
        .map(|(component, field, value)| ((component.as_str(), field.as_str()), *value))
        .collect::<HashMap<_, _>>();
    let changed = current
        .iter()
        .filter(|(component, field, value)| {
            previous_index.get(&(component.as_str(), field.as_str())) != Some(value)
        })
        .count();
    let removed = previous
        .iter()
        .filter(|(component, field, _)| {
            !current_index.contains_key(&(component.as_str(), field.as_str()))
        })
        .count();
    black_box((previous_index, current_index, changed, removed));
    started.elapsed().as_nanos()
}

fn nearest_rank(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let rank = sorted.len().saturating_mul(percentile).div_ceil(100);
    sorted[rank.saturating_sub(1)]
}
