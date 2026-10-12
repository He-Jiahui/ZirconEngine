use std::hint::black_box;
use std::time::Instant;

use crate::core::resource::ResourceLocator;

use super::*;

#[test]
fn optimization_batch_da_material_record_indirect_sort_matches_stable_order() {
    let shader = AssetReference::from_locator(
        ResourceLocator::parse("res://shaders/material-management.zshader")
            .expect("valid shader locator"),
    );
    let repeated_id = ResourceId::from_stable_label("material/repeated");
    let mut expected = vec![
        record(repeated_id, "first", &shader),
        record(
            ResourceId::from_stable_label("material/other"),
            "other",
            &shader,
        ),
        record(repeated_id, "second", &shader),
    ];
    let actual_input = expected.clone();
    expected.sort_by_key(|record| record.material_id);

    let actual = MaterialAssetManagementRecordSet::from_records(actual_input).records;

    assert_eq!(actual, expected);
}

#[test]
fn optimization_batch_da_material_record_sort_uses_compact_index_order() {
    let source = include_str!("../management.rs");
    let production = source.split("#[cfg(test)]").next().unwrap();

    assert!(production.contains("sort_material_management_records(&mut records)"));
    assert!(production.contains("ordered_sources.sort_unstable()"));
    assert!(production.contains("destination_for_source"));
    assert!(!production.contains("records.sort_by_key"));
}

#[test]
#[ignore = "release-only performance evidence"]
fn optimization_batch_da_material_record_indirect_sort_p95() {
    const RECORD_COUNT: usize = 65_536;
    const SAMPLE_COUNT: usize = 17;
    let shader = AssetReference::from_locator(
        ResourceLocator::parse("res://shaders/material-management-bench.zshader")
            .expect("valid shader locator"),
    );
    let template = (0..RECORD_COUNT)
        .rev()
        .map(|index| {
            record(
                ResourceId::from_stable_label(&format!("material/bench/{index}")),
                "benchmark-material-record-payload",
                &shader,
            )
        })
        .collect::<Vec<_>>();

    let (legacy_samples, optimized_samples) = paired_samples::<SAMPLE_COUNT>(&template);
    assert_eq!(legacy_sort(&template), optimized_sort(&template));

    let legacy_p95 = percentile(&legacy_samples, 95);
    let optimized_p95 = percentile(&optimized_samples, 95);
    println!(
        "PERF_RESULT RUNTIME405_MATERIAL_RECORD_INDIRECT_SORT_BENCH_V1 records={RECORD_COUNT} samples={SAMPLE_COUNT} sample_order=alternating legacy_p95_ns={legacy_p95} optimized_p95_ns={optimized_p95}"
    );
    assert!(
        optimized_p95 * 10 <= legacy_p95 * 7,
        "optimized P95 {optimized_p95}ns must be no more than 70% of legacy P95 {legacy_p95}ns"
    );
}

fn record(
    material_id: ResourceId,
    name: &str,
    shader: &AssetReference,
) -> MaterialAssetManagementRecord {
    MaterialAssetManagementRecord {
        material_id,
        overview: MaterialAssetOverview {
            name: Some(name.to_string()),
            shader: shader.clone(),
            property_override_count: 3,
            texture_slot_count: 5,
            texture_reference_count: 4,
            fallback_texture_slot_count: 1,
            validation_error_count: 0,
            validation_diagnostic_count: 0,
            direct_reference_count: 5,
        },
    }
}

fn legacy_sort(template: &[MaterialAssetManagementRecord]) -> Vec<MaterialAssetManagementRecord> {
    let mut records = template.to_vec();
    records.sort_by_key(|record| record.material_id);
    records
}

fn optimized_sort(
    template: &[MaterialAssetManagementRecord],
) -> Vec<MaterialAssetManagementRecord> {
    let mut records = template.to_vec();
    sort_material_management_records(&mut records);
    records
}

fn paired_samples<const SAMPLE_COUNT: usize>(
    template: &[MaterialAssetManagementRecord],
) -> (Vec<u128>, Vec<u128>) {
    black_box(legacy_sort(template));
    black_box(optimized_sort(template));
    let mut legacy_samples = Vec::with_capacity(SAMPLE_COUNT);
    let mut optimized_samples = Vec::with_capacity(SAMPLE_COUNT);
    for sample_index in 0..SAMPLE_COUNT {
        if sample_index % 2 == 0 {
            legacy_samples.push(sample_sort(template, |records| {
                records.sort_by_key(|record| record.material_id);
            }));
            optimized_samples.push(sample_sort(template, sort_material_management_records));
        } else {
            optimized_samples.push(sample_sort(template, sort_material_management_records));
            legacy_samples.push(sample_sort(template, |records| {
                records.sort_by_key(|record| record.material_id);
            }));
        }
    }
    (legacy_samples, optimized_samples)
}

fn sample_sort(
    template: &[MaterialAssetManagementRecord],
    operation: impl FnOnce(&mut [MaterialAssetManagementRecord]),
) -> u128 {
    let mut records = template.to_vec();
    let started = Instant::now();
    operation(black_box(&mut records));
    let elapsed = started.elapsed().as_nanos();
    black_box(records);
    elapsed
}

fn percentile(samples: &[u128], percentile: usize) -> u128 {
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    assert!(!sorted.is_empty());
    assert!((1..=100).contains(&percentile));
    sorted[(sorted.len() * percentile).div_ceil(100) - 1]
}
