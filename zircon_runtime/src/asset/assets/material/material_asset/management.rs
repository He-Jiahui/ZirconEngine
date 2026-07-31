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
        records.sort_by_key(|record| record.material_id);
        let summary = MaterialAssetManagementRecordSetSummary::from_records(&records);
        Self { records, summary }
    }
}
