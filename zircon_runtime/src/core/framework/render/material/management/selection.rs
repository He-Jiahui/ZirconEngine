use std::collections::{HashMap, HashSet};

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
        let mut records_by_id = HashMap::with_capacity(records.len());
        for record in records {
            records_by_id.entry(record.material_id).or_insert(record);
        }

        for material_id in &requested_material_ids {
            if let Some(record) = records_by_id.get(material_id) {
                selected_records.push((**record).clone());
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
    let mut seen_ids = HashSet::new();
    for material_id in material_ids {
        if seen_ids.insert(material_id) {
            unique_ids.push(material_id);
        }
    }
    unique_ids
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::framework::render::material::management::RenderMaterialManagementSnapshot;

    #[test]
    fn selection_owns_records_after_source_changes() {
        let material_id = ResourceId::from_stable_label("material:owned-selection");
        let mut records = vec![RenderMaterialManagementRecord {
            material_id,
            material_name: Some("Original".to_string()),
            snapshot: RenderMaterialManagementSnapshot::default(),
        }];

        let selection = RenderMaterialManagementSelection::from_records(&records, [material_id]);
        records[0].material_name = Some("Changed after selection".to_string());

        assert_eq!(selection.records.len(), 1);
        assert_eq!(
            selection.records[0].material_name.as_deref(),
            Some("Original")
        );
    }
}
