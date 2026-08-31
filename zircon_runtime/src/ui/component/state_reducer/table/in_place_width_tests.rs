use std::hint::black_box;
use std::time::Instant;

use super::*;

const FIELD_NAME_BYTES: usize = 32 * 1024;
const OPERATIONS_PER_SAMPLE: usize = 512;
const SAMPLE_PAIRS: usize = 21;

#[test]
fn optimization_batch_20260826hh_runtime254_preserves_column_width_updates() {
    let mut widths = BTreeMap::new();
    widths.insert("name".to_string(), UiValue::Float(96.0));

    let mut name_column = BTreeMap::new();
    name_column.insert("field".to_string(), UiValue::String("name".to_string()));
    name_column.insert("width".to_string(), UiValue::Float(96.0));
    let mut age_column = BTreeMap::new();
    age_column.insert("field".to_string(), UiValue::String("age".to_string()));
    age_column.insert("width".to_string(), UiValue::Float(72.0));

    let mut state = UiComponentState::new()
        .with_value("column_widths", UiValue::Map(widths))
        .with_value(
            "columns",
            UiValue::Array(vec![UiValue::Map(name_column), UiValue::Map(age_column)]),
        );

    apply_column_width(&mut state, "name", 144.0);
    let Some(UiValue::Map(widths)) = state.value("column_widths") else {
        panic!("column_widths must remain a map");
    };
    assert_eq!(widths.get("name"), Some(&UiValue::Float(144.0)));

    let Some(UiValue::Array(columns)) = state.value("columns") else {
        panic!("columns must remain an array");
    };
    assert!(matches!(
        &columns[0],
        UiValue::Map(values) if values.get("width") == Some(&UiValue::Float(144.0))
    ));
    assert!(matches!(
        &columns[1],
        UiValue::Map(values) if values.get("width") == Some(&UiValue::Float(72.0))
    ));

    apply_column_width(&mut state, "status", 88.0);
    let Some(UiValue::Map(widths)) = state.value("column_widths") else {
        panic!("column_widths must remain a map");
    };
    assert_eq!(widths.get("status"), Some(&UiValue::Float(88.0)));
}

#[test]
fn optimization_batch_20260826hh_runtime254_updates_existing_width_keys_in_place() {
    let source = include_str!("../table.rs");
    let start = source
        .find("fn apply_column_width(")
        .expect("apply_column_width function");
    let end = source[start..]
        .find("\nfn column_matches")
        .map(|offset| start + offset)
        .expect("column_matches boundary");
    let body = &source[start..end];

    assert!(body.contains("set_borrowed_map_value(widths, field"));
    assert!(body.contains("set_borrowed_map_value(values, \"width\""));
    assert!(body.contains("values.get_mut(key)"));
    assert!(body.contains("values.insert(key.to_string(), value)"));
}

#[test]
#[ignore = "managed release performance evidence"]
fn optimization_batch_20260826hh_runtime254_table_width_in_place_release_benchmark() {
    let field = "column".repeat(FIELD_NAME_BYTES / "column".len());
    let baseline = BTreeMap::from([(field.clone(), UiValue::Float(0.0))]);
    let mut legacy = baseline.clone();
    let mut optimized = baseline;

    let mut legacy_ns = Vec::with_capacity(SAMPLE_PAIRS);
    let mut optimized_ns = Vec::with_capacity(SAMPLE_PAIRS);
    for sample_index in 0..SAMPLE_PAIRS {
        let mut measure_legacy = || {
            let started = Instant::now();
            for operation in 0..OPERATIONS_PER_SAMPLE {
                legacy_set_map_value(
                    black_box(&mut legacy),
                    black_box(&field),
                    UiValue::Float(operation as f64),
                );
            }
            legacy_ns.push(started.elapsed().as_nanos().max(1));
        };
        let mut measure_optimized = || {
            let started = Instant::now();
            for operation in 0..OPERATIONS_PER_SAMPLE {
                set_borrowed_map_value(
                    black_box(&mut optimized),
                    black_box(&field),
                    UiValue::Float(operation as f64),
                );
            }
            optimized_ns.push(started.elapsed().as_nanos().max(1));
        };
        if sample_index % 2 == 0 {
            measure_legacy();
            measure_optimized();
        } else {
            measure_optimized();
            measure_legacy();
        }
    }
    assert_eq!(legacy, optimized);

    let legacy_p50_ns = percentile(&legacy_ns, 50);
    let legacy_p95_ns = percentile(&legacy_ns, 95);
    let optimized_p50_ns = percentile(&optimized_ns, 50);
    let optimized_p95_ns = percentile(&optimized_ns, 95);
    println!(
        "RUNTIME254_TABLE_WIDTH_IN_PLACE_BENCH_V1 \
         field_name_bytes={FIELD_NAME_BYTES} operations_per_sample={OPERATIONS_PER_SAMPLE} \
         sample_pairs={SAMPLE_PAIRS} legacy_p50_ns={legacy_p50_ns} \
         legacy_p95_ns={legacy_p95_ns} optimized_p50_ns={optimized_p50_ns} \
         optimized_p95_ns={optimized_p95_ns} legacy_ns={} optimized_ns={}",
        samples(&legacy_ns),
        samples(&optimized_ns),
    );
    assert!(
        optimized_p95_ns.saturating_mul(100) <= legacy_p95_ns.saturating_mul(70),
        "optimized P95 {optimized_p95_ns}ns must be at most 70% of legacy P95 {legacy_p95_ns}ns"
    );
}

fn legacy_set_map_value(values: &mut BTreeMap<String, UiValue>, key: &str, value: UiValue) {
    values.insert(key.to_string(), value);
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut ordered = samples.to_vec();
    ordered.sort_unstable();
    let rank = ordered.len().saturating_mul(percentile).div_ceil(100);
    ordered[rank.saturating_sub(1)]
}

fn samples(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
