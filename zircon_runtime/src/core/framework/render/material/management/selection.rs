use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementIssueIndex, RenderMaterialManagementRecord,
    RenderMaterialManagementRecordSet, RenderMaterialManagementRecordSummary,
    RenderMaterialManagementStatusIndex,
};
use crate::core::resource::ResourceId;

#[cfg(test)]
#[path = "selection/tests/id_capacity_tests.rs"]
mod id_capacity_tests;

#[cfg(test)]
#[path = "selection/tests/optimization_batch_iy_runtime636_tests.rs"]
mod optimization_batch_iy_runtime636_tests;

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
    // 选择先按请求顺序去重，再保留记录表中每个材质 ID 的首个记录；缺失 ID 单独返回，并据选中记录重建摘要、状态索引和问题索引。
    pub fn from_records(
        records: &[RenderMaterialManagementRecord],
        material_ids: impl IntoIterator<Item = ResourceId>,
    ) -> Self {
        let requested_material_ids = unique_material_ids(material_ids);
        let result_capacity = requested_material_ids.len();
        let mut selected_records = Vec::with_capacity(result_capacity);
        let mut missing_material_ids = Vec::with_capacity(result_capacity);
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
    let material_ids = material_ids.into_iter();
    let (minimum_ids, _) = material_ids.size_hint();
    let mut unique_ids = Vec::with_capacity(minimum_ids);
    let mut seen_ids = HashSet::with_capacity(minimum_ids);
    for material_id in material_ids {
        if seen_ids.insert(material_id) {
            unique_ids.push(material_id);
        }
    }
    unique_ids
}

#[cfg(test)]
#[path = "tests/selection.rs"]
mod tests;
