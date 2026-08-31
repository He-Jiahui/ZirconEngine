use std::collections::BTreeSet;
use std::hint::black_box;
use std::time::{Duration, Instant};

use zircon_runtime_interface::ui::component::{
    UiComponentCategory, UiComponentDescriptor, UiOptionDescriptor, UiPropSchema, UiSlotSchema,
    UiValue, UiValueKind,
};

use super::*;

const SCHEMA_COUNT: usize = 65_536;
const SAMPLE_COUNT: usize = 17;

fn descriptor() -> UiComponentDescriptor {
    UiComponentDescriptor::new(
        "fixture.descriptor",
        "Fixture Descriptor",
        UiComponentCategory::Visual,
        "fixture",
    )
}

fn percentile_95(samples: &mut [Duration]) -> Duration {
    samples.sort_unstable();
    samples[(samples.len() - 1) * 95 / 100]
}

fn schemas() -> Vec<UiPropSchema> {
    (0..SCHEMA_COUNT)
        .map(|index| {
            UiPropSchema::new(
                format!("generated_descriptor_schema_with_long_identity_{index:05}"),
                UiValueKind::String,
            )
        })
        .collect()
}

fn ordered_schema_names_are_valid(schemas: &[UiPropSchema]) -> bool {
    let mut names = BTreeSet::new();
    for schema in schemas {
        if schema.name.trim().is_empty() || !names.insert(schema.name.as_str()) {
            return false;
        }
        let mut option_ids = BTreeSet::new();
        if schema
            .options
            .iter()
            .any(|option| !option_ids.insert(option.id.as_str()))
        {
            return false;
        }
    }
    true
}

#[test]
fn optimization_batch_20260826ag_runtime75_hash_validation_preserves_first_duplicate_errors() {
    let duplicate_prop = descriptor()
        .with_prop(UiPropSchema::new("first", UiValueKind::String))
        .with_prop(UiPropSchema::new("later", UiValueKind::String))
        .with_prop(UiPropSchema::new("first", UiValueKind::String));
    assert_eq!(
        validate_component_descriptor(&duplicate_prop),
        Err(UiComponentDescriptorError::DuplicateSchemaName {
            component_id: "fixture.descriptor".to_string(),
            schema_kind: "prop",
            name: "first".to_string(),
        })
    );

    let duplicate_option = descriptor().with_prop(
        UiPropSchema::new("choice", UiValueKind::String).with_options([
            UiOptionDescriptor::new("first", "First", UiValue::String("first".to_string())),
            UiOptionDescriptor::new("later", "Later", UiValue::String("later".to_string())),
            UiOptionDescriptor::new("first", "Duplicate", UiValue::String("first".to_string())),
        ]),
    );
    assert_eq!(
        validate_component_descriptor(&duplicate_option),
        Err(UiComponentDescriptorError::DuplicateSchemaName {
            component_id: "fixture.descriptor".to_string(),
            schema_kind: "option",
            name: "first".to_string(),
        })
    );

    let duplicate_slot = descriptor()
        .slot(UiSlotSchema::new("header"))
        .slot(UiSlotSchema::new("content"))
        .slot(UiSlotSchema::new("header"));
    assert_eq!(
        validate_component_descriptor(&duplicate_slot),
        Err(UiComponentDescriptorError::DuplicateSlotName {
            component_id: "fixture.descriptor".to_string(),
            name: "header".to_string(),
        })
    );
}

#[test]
fn optimization_batch_20260826ag_runtime75_descriptor_validation_uses_borrowed_hash_membership() {
    let source = include_str!("../validation.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("use std::collections::HashSet;"));
    assert_eq!(production.matches("HashSet::with_capacity").count(), 3);
    assert!(production.contains("names.insert(schema.name.as_str())"));
    assert!(production.contains("option_ids.insert(option.id.as_str())"));
    assert!(production.contains("names.insert(slot.name.as_str())"));
    assert!(!production.contains("BTreeSet"));
}

#[test]
#[ignore = "release performance evidence"]
fn optimization_batch_20260826ag_runtime75_descriptor_hash_validation_performance_evidence() {
    let descriptor = descriptor();
    let schemas = schemas();
    assert!(ordered_schema_names_are_valid(&schemas));
    assert!(validate_schema_names(&descriptor, "prop", &schemas).is_ok());

    let mut ordered_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut hash_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            let started = Instant::now();
            black_box(ordered_schema_names_are_valid(black_box(&schemas)));
            ordered_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(validate_schema_names(
                black_box(&descriptor),
                "prop",
                black_box(&schemas),
            ));
            hash_samples.push(started.elapsed());
        } else {
            let started = Instant::now();
            black_box(validate_schema_names(
                black_box(&descriptor),
                "prop",
                black_box(&schemas),
            ));
            hash_samples.push(started.elapsed());

            let started = Instant::now();
            black_box(ordered_schema_names_are_valid(black_box(&schemas)));
            ordered_samples.push(started.elapsed());
        }
    }

    let ordered_p95 = percentile_95(&mut ordered_samples);
    let hash_p95 = percentile_95(&mut hash_samples);
    println!(
        "RUNTIME75_DESCRIPTOR_HASH_VALIDATION_BENCH_V1 \
         schemas={SCHEMA_COUNT} borrowed_schema_identity=true \
         ordered_p95_ns={} hash_p95_ns={}",
        ordered_p95.as_nanos(),
        hash_p95.as_nanos(),
    );
    assert!(
        hash_p95.as_nanos() * 100 <= ordered_p95.as_nanos() * 60,
        "hash-validation P95 {:?} exceeded 60% of ordered-validation P95 {:?}",
        hash_p95,
        ordered_p95,
    );
}
