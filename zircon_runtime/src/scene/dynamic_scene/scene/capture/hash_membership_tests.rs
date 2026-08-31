use std::collections::{BTreeSet, HashSet};
use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime_interface::reflect::{
    ReflectEditorHint, ReflectFieldId, ReflectFieldInfo, ReflectFieldValue,
    ReflectSerializationStrategy, ReflectTypeInfo, ReflectTypePath, ReflectTypeRegistration,
    ReflectedValue,
};

use super::*;

const TYPE_ADMISSION_COUNT: usize = 65_536;
const UNIQUE_TYPE_COUNT: usize = 8_192;
const FIELD_COUNT: usize = 2_048;
const SAMPLE_COUNT: usize = 17;
const FIXTURE_TYPE_PATH: &str = "optimization::CaptureFixture";

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn registration(fields: Vec<ReflectFieldInfo>) -> ReflectTypeRegistration {
    ReflectTypeRegistration::new(
        ReflectTypePath::new(FIXTURE_TYPE_PATH, "CaptureFixture").unwrap(),
        "Capture Fixture",
        ReflectTypeInfo::struct_with_fields(fields),
        ReflectSerializationStrategy::Value,
    )
}

fn legacy_serializable_fields(
    metadata: &ReflectTypeRegistration,
    fields: Vec<ReflectFieldValue>,
) -> Vec<ReflectFieldValue> {
    fields
        .into_iter()
        .filter(|field| {
            metadata
                .type_info
                .fields
                .iter()
                .any(|info| info.name == field.field_name && info.serializable)
        })
        .collect()
}

fn ordered_descriptor_match_count(required: &[String], descriptors: &[String]) -> usize {
    let required = required.iter().map(String::as_str).collect::<BTreeSet<_>>();
    descriptors
        .iter()
        .filter(|descriptor| required.contains(descriptor.as_str()))
        .count()
}

fn hash_descriptor_match_count(required: &[String], descriptors: &[String]) -> usize {
    let required = required.iter().map(String::as_str).collect::<HashSet<_>>();
    descriptors
        .iter()
        .filter(|descriptor| required.contains(descriptor.as_str()))
        .count()
}

#[test]
fn optimization_batch_20260826ai_runtime52_dense_field_filter_preserves_schema_order() {
    let metadata = registration(vec![
        fixture_field("alpha"),
        fixture_field("hidden").with_serializable(false),
        fixture_field("gamma"),
    ]);
    let fields = vec![
        fixture_value("alpha", 1),
        fixture_value("hidden", 2),
        fixture_value("gamma", 3),
    ];

    let filtered = serializable_fields(&metadata, fields);

    assert_eq!(
        filtered
            .iter()
            .map(|field| field.field_name.as_str())
            .collect::<Vec<_>>(),
        vec!["alpha", "gamma"]
    );
}

#[test]
fn optimization_batch_20260826ai_runtime52_capture_uses_dense_schema_zip() {
    let source = include_str!("../capture.rs");
    let type_selection = source
        .split("fn component_type_descriptors_from_world")
        .nth(1)
        .and_then(|body| body.split("fn node_record_from_scene_node").next())
        .expect("component type selection implementation");
    let field_filter = source
        .split("fn serializable_fields")
        .nth(1)
        .and_then(|body| body.split("#[cfg(test)]").next())
        .expect("serializable field filter implementation");

    assert!(source.contains("use std::collections::HashSet;"));
    assert!(type_selection.contains("let mut required_type_ids = HashSet::new();"));
    assert!(type_selection.contains("required_type_ids.insert(component.type_path.as_str())"));
    assert!(field_filter.contains(".fields\n        .iter()\n        .zip(fields)"));
    assert!(field_filter.contains("debug_assert_eq!(info.id, value.field_id)"));
    assert!(field_filter.contains("info.serializable.then_some(value)"));
    assert!(!source
        .split("#[cfg(test)]")
        .next()
        .unwrap()
        .contains("BTreeSet"));
    assert!(!field_filter.contains("HashSet::with_capacity"));
    assert!(!field_filter.contains("field.field_name"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ai_runtime52_capture_hash_membership_performance_evidence() {
    let required = (0..TYPE_ADMISSION_COUNT)
        .map(|index| {
            format!(
                "plugin.capture.component_type_with_long_identity_{:05}",
                index % UNIQUE_TYPE_COUNT
            )
        })
        .collect::<Vec<_>>();
    let descriptors = (0..UNIQUE_TYPE_COUNT * 2)
        .map(|index| format!("plugin.capture.component_type_with_long_identity_{index:05}"))
        .collect::<Vec<_>>();
    assert_eq!(
        ordered_descriptor_match_count(&required, &descriptors),
        hash_descriptor_match_count(&required, &descriptors)
    );

    let metadata = registration(
        (0..FIELD_COUNT)
            .map(|index| fixture_field(&format!("capture_field_with_long_identity_{index:05}")))
            .collect(),
    );
    let fields = (0..FIELD_COUNT)
        .map(|index| {
            fixture_value(
                &format!("capture_field_with_long_identity_{index:05}"),
                index as u64,
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        legacy_serializable_fields(&metadata, fields.clone()),
        serializable_fields(&metadata, fields.clone())
    );

    let mut ordered_type_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_type_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut nested_field_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut dense_field_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_descriptor_match_count(
                black_box(&required),
                black_box(&descriptors),
            ));
            ordered_type_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(hash_descriptor_match_count(
                black_box(&required),
                black_box(&descriptors),
            ));
            hash_type_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(legacy_serializable_fields(
                black_box(&metadata),
                black_box(fields.clone()),
            ));
            nested_field_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(serializable_fields(
                black_box(&metadata),
                black_box(fields.clone()),
            ));
            dense_field_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(hash_descriptor_match_count(
                black_box(&required),
                black_box(&descriptors),
            ));
            hash_type_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(ordered_descriptor_match_count(
                black_box(&required),
                black_box(&descriptors),
            ));
            ordered_type_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(serializable_fields(
                black_box(&metadata),
                black_box(fields.clone()),
            ));
            dense_field_samples.push(started.elapsed());
            let started = Instant::now();
            black_box(legacy_serializable_fields(
                black_box(&metadata),
                black_box(fields.clone()),
            ));
            nested_field_samples.push(started.elapsed());
        }
    }

    let ordered_type_p95 = percentile_95(&mut ordered_type_samples);
    let hash_type_p95 = percentile_95(&mut hash_type_samples);
    let nested_field_p95 = percentile_95(&mut nested_field_samples);
    let dense_field_p95 = percentile_95(&mut dense_field_samples);
    println!(
        "RUNTIME52_CAPTURE_DENSE_FIELD_FILTER_BENCH_V2 \
         type_admissions={TYPE_ADMISSION_COUNT} unique_types={UNIQUE_TYPE_COUNT} \
         fields={FIELD_COUNT} borrowed_identity=true \
         ordered_type_p95_ns={} hash_type_p95_ns={} \
         nested_field_p95_ns={} dense_field_p95_ns={}",
        ordered_type_p95.as_nanos(),
        hash_type_p95.as_nanos(),
        nested_field_p95.as_nanos(),
        dense_field_p95.as_nanos(),
    );
    assert!(
        hash_type_p95.as_nanos() * 100 <= ordered_type_p95.as_nanos() * 60,
        "hash type-membership P95 {:?} exceeded 60% of ordered P95 {:?}",
        hash_type_p95,
        ordered_type_p95,
    );
    assert!(
        dense_field_p95.as_nanos() * 100 <= nested_field_p95.as_nanos() * 60,
        "dense field-filter P95 {:?} exceeded 60% of nested P95 {:?}",
        dense_field_p95,
        nested_field_p95,
    );
}

fn fixture_field(name: &str) -> ReflectFieldInfo {
    ReflectFieldInfo::new(
        ReflectFieldId::from_stable_keys(FIXTURE_TYPE_PATH, name),
        name,
        "Unsigned",
        ReflectEditorHint::Unsigned,
    )
}

fn fixture_value(name: &str, value: u64) -> ReflectFieldValue {
    ReflectFieldValue::new(
        ReflectFieldId::from_stable_keys(FIXTURE_TYPE_PATH, name),
        name,
        ReflectedValue::Unsigned(value),
    )
}
