use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementQuery, RenderMaterialManagementQueryResult,
    RenderMaterialManagementRecord, RenderMaterialManagementRecordSet,
    RenderMaterialManagementSelection,
};
use crate::core::resource::ResourceId;

/// Query page paired with full records for the same visible material ids.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQuerySelection {
    #[serde(default)]
    pub query: RenderMaterialManagementQuery,
    #[serde(default)]
    pub query_result: RenderMaterialManagementQueryResult,
    #[serde(default)]
    pub selection: RenderMaterialManagementSelection,
}

impl RenderMaterialManagementQuerySelection {
    /// 在同一记录切片上生成分页摘要和对应的完整详情；页面 ID 顺序即详情顺序，避免二次查询跨快照。
    pub fn from_records(
        records: &[RenderMaterialManagementRecord],
        query: RenderMaterialManagementQuery,
    ) -> Self {
        let query_result = query.apply_to_records(records);
        let mut page_material_ids = Vec::with_capacity(query_result.records.len());
        page_material_ids.extend(query_result.records.iter().map(|record| record.material_id));
        let selection = RenderMaterialManagementSelection::from_records(records, page_material_ids);

        Self {
            query,
            query_result,
            selection,
        }
    }

    pub fn from_record_set(
        record_set: &RenderMaterialManagementRecordSet,
        query: RenderMaterialManagementQuery,
    ) -> Self {
        Self::from_records(&record_set.records, query)
    }

    pub fn is_empty(&self) -> bool {
        self.query_result.records.is_empty()
    }

    pub fn len(&self) -> usize {
        self.query_result.records.len()
    }

    pub fn is_complete(&self) -> bool {
        self.selection.is_complete()
    }
}

#[cfg(test)]
#[path = "tests/query_selection.rs"]
mod tests;
