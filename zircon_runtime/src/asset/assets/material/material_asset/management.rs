//! 材质管理记录是编辑器/资产管理界面的只读投影；从资源与就绪诊断汇总，不能代替实际材质加载状态。

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
#[path = "tests/management_optimization_tests.rs"]
mod optimization_tests;
