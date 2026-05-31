use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementIssueIndex, RenderMaterialManagementRecord,
    RenderMaterialManagementRecordSet, RenderMaterialManagementRecordSummary,
    RenderMaterialManagementStatusIndex,
};
use crate::core::resource::ResourceId;

/// Full management records selected by material id, preserving request order.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementSelection {
    /// Count of unique material ids after duplicate selection ids are collapsed.
    #[serde(default)]
    pub requested_count: usize,
    #[serde(default)]
    pub summary: RenderMaterialManagementRecordSummary,
    #[serde(default)]
    pub status_index: RenderMaterialManagementStatusIndex,
    #[serde(default)]
    pub issue_index: RenderMaterialManagementIssueIndex,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<RenderMaterialManagementRecord>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing_material_ids: Vec<ResourceId>,
}

impl RenderMaterialManagementSelection {
    pub fn from_records(
        records: &[RenderMaterialManagementRecord],
        material_ids: impl IntoIterator<Item = ResourceId>,
    ) -> Self {
        let requested_material_ids = unique_material_ids(material_ids);
        let mut selected_records = Vec::new();
        let mut missing_material_ids = Vec::new();

        for material_id in &requested_material_ids {
            if let Some(record) = records
                .iter()
                .find(|record| record.material_id == *material_id)
            {
                selected_records.push(record.clone());
            } else {
                missing_material_ids.push(*material_id);
            }
        }

        let summary = RenderMaterialManagementRecordSummary::from_records(&selected_records);
        let status_index = RenderMaterialManagementStatusIndex::from_records(&selected_records);
        let issue_index = RenderMaterialManagementIssueIndex::from_records(&selected_records);
        Self {
            requested_count: requested_material_ids.len(),
            summary,
            status_index,
            issue_index,
            records: selected_records,
            missing_material_ids,
        }
    }

    pub fn from_record_set(
        record_set: &RenderMaterialManagementRecordSet,
        material_ids: impl IntoIterator<Item = ResourceId>,
    ) -> Self {
        Self::from_records(&record_set.records, material_ids)
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn missing_count(&self) -> usize {
        self.missing_material_ids.len()
    }

    pub fn is_complete(&self) -> bool {
        self.missing_material_ids.is_empty()
    }
}

fn unique_material_ids(material_ids: impl IntoIterator<Item = ResourceId>) -> Vec<ResourceId> {
    let mut unique_ids = Vec::new();
    for material_id in material_ids {
        if !unique_ids.contains(&material_id) {
            unique_ids.push(material_id);
        }
    }
    unique_ids
}
