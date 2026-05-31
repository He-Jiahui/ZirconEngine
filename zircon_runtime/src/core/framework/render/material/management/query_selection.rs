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
    pub fn from_records(
        records: &[RenderMaterialManagementRecord],
        query: RenderMaterialManagementQuery,
    ) -> Self {
        let query_result = query.apply_to_records(records);
        let page_material_ids = query_result
            .records
            .iter()
            .map(|record| record.material_id)
            .collect::<Vec<ResourceId>>();
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
