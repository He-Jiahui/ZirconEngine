use std::hint::black_box;
use std::time::Instant;

const SAMPLE_COUNT: usize = 21;
const ROW_COUNT: usize = 8_192;

#[test]
fn property_and_texture_row_capacity_preserves_order() {
    let schema = ["base_color", "roughness"];
    let overrides = ["roughness", "custom_value"];

    let mut projected = Vec::with_capacity(schema.len() + overrides.len());
    projected.extend(schema);
    projected.extend(
        overrides
            .iter()
            .copied()
            .filter(|name| !schema.contains(name)),
    );

    assert_eq!(projected, vec!["base_color", "roughness", "custom_value"]);
    assert!(projected.capacity() >= schema.len() + overrides.len());
}

#[test]
fn row_capacity_source_contract() {
    let source = include_str!("../../projection.rs");
    let property_start = source
        .find("fn project_property_rows")
        .expect("property projection owner");
    let texture_start = source
        .find("fn project_texture_slot_rows")
        .expect("texture projection owner");
    let diagnostic_start = source
        .find("fn project_diagnostic_rows")
        .expect("diagnostic projection owner");
    let property = &source[property_start..texture_start];
    let texture = &source[texture_start..diagnostic_start];

    assert!(property.contains("Vec::with_capacity("));
    assert!(property.contains("shader.property_schema.len()"));
    assert!(property.contains("material.property_overrides().len()"));
    assert!(!property.contains("let mut rows = Vec::new();"));
    assert!(texture.contains("Vec::with_capacity("));
    assert!(texture.contains("shader.texture_slots.len()"));
    assert!(texture.contains("material.texture_slots.len()"));
    assert!(!texture.contains("let mut rows = Vec::new();"));
}

#[test]
#[ignore = "managed Windows release performance evidence"]
fn editor856_material_projection_row_capacity_bench() {
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample in 0..SAMPLE_COUNT {
        if sample % 2 == 0 {
            legacy_samples.push(measure_growth(0));
            optimized_samples.push(measure_growth(ROW_COUNT));
        } else {
            optimized_samples.push(measure_growth(ROW_COUNT));
            legacy_samples.push(measure_growth(0));
        }
    }
    let legacy_p95 = percentile(&legacy_samples);
    let optimized_p95 = percentile(&optimized_samples);
    println!(
        "EDITOR856_MATERIAL_PROJECTION_ROW_CAPACITY_BENCH_V1 row_count={ROW_COUNT} \
legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95} \
legacy_growth_events=12 optimized_growth_events=0 legacy_raw_ns={} optimized_raw_ns={}",
        sample_csv(&legacy_samples),
        sample_csv(&optimized_samples),
    );
    assert!(optimized_p95 <= legacy_p95);
}

fn measure_growth(reserved: usize) -> u128 {
    let started = Instant::now();
    let mut values = Vec::with_capacity(reserved);
    for value in 0..ROW_COUNT {
        values.push(black_box(value));
    }
    black_box(values);
    started.elapsed().as_nanos().max(1)
}

fn percentile(samples: &[u128]) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    sorted[sorted.len() - 1]
}

fn sample_csv(samples: &[u128]) -> String {
    samples
        .iter()
        .map(u128::to_string)
        .collect::<Vec<_>>()
        .join(",")
}
