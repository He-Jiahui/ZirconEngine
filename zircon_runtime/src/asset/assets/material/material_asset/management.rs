use serde::{Deserialize, Serialize};

use crate::asset::AssetReference;
use crate::core::resource::ResourceId;

/// Asset-level material summary that does not require renderer preparation.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialAssetOverview {
    pub name: Option<String>,
    pub shader: AssetReference,
    pub property_override_count: usize,
    pub texture_slot_count: usize,
    pub texture_reference_count: usize,
    pub fallback_texture_slot_count: usize,
    pub validation_error_count: usize,
    pub validation_diagnostic_count: usize,
    pub direct_reference_count: usize,
}

/// Stable list row for registered `.zmaterial` assets.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialAssetManagementRecord {
    pub material_id: ResourceId,
    pub overview: MaterialAssetOverview,
}

/// Cross-row totals for material assets before renderer readiness is considered.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialAssetManagementRecordSetSummary {
    pub material_count: usize,
    pub ready_count: usize,
    pub issue_material_count: usize,
    pub property_override_count: usize,
    pub texture_slot_count: usize,
    pub texture_reference_count: usize,
    pub fallback_texture_slot_count: usize,
    pub validation_error_count: usize,
    pub validation_diagnostic_count: usize,
    pub direct_reference_count: usize,
}

/// Sorted material asset rows plus aggregate authoring/dependency counts.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaterialAssetManagementRecordSet {
    pub records: Vec<MaterialAssetManagementRecord>,
    pub summary: MaterialAssetManagementRecordSetSummary,
}

impl MaterialAssetManagementRecordSetSummary {
    pub fn from_records(records: &[MaterialAssetManagementRecord]) -> Self {
        let mut summary = Self {
            material_count: records.len(),
            ..Self::default()
        };
        for record in records {
            let overview = &record.overview;
            summary.issue_material_count += usize::from(
                overview.validation_error_count + overview.validation_diagnostic_count > 0,
            );
            summary.property_override_count += overview.property_override_count;
            summary.texture_slot_count += overview.texture_slot_count;
            summary.texture_reference_count += overview.texture_reference_count;
            summary.fallback_texture_slot_count += overview.fallback_texture_slot_count;
            summary.validation_error_count += overview.validation_error_count;
            summary.validation_diagnostic_count += overview.validation_diagnostic_count;
            summary.direct_reference_count += overview.direct_reference_count;
        }
        summary.ready_count = summary.material_count - summary.issue_material_count;
        summary
    }

    pub fn degraded_count(&self) -> usize {
        self.issue_material_count
    }

    pub fn issue_row_count(&self) -> usize {
        self.validation_error_count + self.validation_diagnostic_count
    }
}

impl MaterialAssetManagementRecordSet {
    pub fn from_records(mut records: Vec<MaterialAssetManagementRecord>) -> Self {
        sort_material_management_records(&mut records);
        let summary = MaterialAssetManagementRecordSetSummary::from_records(&records);
        Self { records, summary }
    }
}

fn sort_material_management_records(records: &mut [MaterialAssetManagementRecord]) {
    let mut ordered_sources = records
        .iter()
        .enumerate()
        .map(|(source_index, record)| (record.material_id, source_index))
        .collect::<Vec<_>>();
    ordered_sources.sort_unstable();

    let mut destination_for_source = vec![0; records.len()];
    for (destination_index, (_, source_index)) in ordered_sources.into_iter().enumerate() {
        destination_for_source[source_index] = destination_index;
    }
    for current_index in 0..records.len() {
        while destination_for_source[current_index] != current_index {
            let destination_index = destination_for_source[current_index];
            records.swap(current_index, destination_index);
            destination_for_source.swap(current_index, destination_index);
        }
    }
}

#[cfg(test)]
mod optimization_tests {
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
        let source = include_str!("management.rs");
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

    fn legacy_sort(
        template: &[MaterialAssetManagementRecord],
    ) -> Vec<MaterialAssetManagementRecord> {
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
}
