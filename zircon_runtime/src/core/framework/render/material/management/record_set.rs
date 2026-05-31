use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementIssueIndex, RenderMaterialManagementIssueKind,
    RenderMaterialManagementIssueView, RenderMaterialManagementOverview,
    RenderMaterialManagementQuery, RenderMaterialManagementQueryResult,
    RenderMaterialManagementQuerySelection, RenderMaterialManagementRecord,
    RenderMaterialManagementRecordSummary, RenderMaterialManagementSelection,
    RenderMaterialManagementSortOrder, RenderMaterialManagementStatusIndex,
    RenderMaterialManagementStatusView,
};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessStatus;
use crate::core::resource::ResourceId;

/// List payload that keeps record ordering and summary counts together.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementRecordSet {
    #[serde(default)]
    pub summary: RenderMaterialManagementRecordSummary,
    #[serde(default)]
    pub status_index: RenderMaterialManagementStatusIndex,
    #[serde(default)]
    pub issue_index: RenderMaterialManagementIssueIndex,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<RenderMaterialManagementRecord>,
}

impl RenderMaterialManagementRecordSet {
    pub fn from_records(records: Vec<RenderMaterialManagementRecord>) -> Self {
        let summary = RenderMaterialManagementRecordSummary::from_records(&records);
        let status_index = RenderMaterialManagementStatusIndex::from_records(&records);
        let issue_index = RenderMaterialManagementIssueIndex::from_records(&records);
        Self {
            summary,
            status_index,
            issue_index,
            records,
        }
    }

    pub fn from_sorted_records(
        mut records: Vec<RenderMaterialManagementRecord>,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        sort_order.sort_records(&mut records);
        Self::from_records(records)
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn overview(&self) -> RenderMaterialManagementOverview {
        RenderMaterialManagementOverview::from_record_set(self)
    }

    pub fn overview_sorted(
        &self,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementOverview {
        RenderMaterialManagementOverview::from_record_set(self).sorted(sort_order)
    }

    pub fn status_view(
        &self,
        status: RenderMaterialReadinessStatus,
    ) -> RenderMaterialManagementStatusView {
        RenderMaterialManagementStatusView::from_record_set(self, status)
    }

    pub fn status_view_sorted(
        &self,
        status: RenderMaterialReadinessStatus,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementStatusView {
        RenderMaterialManagementStatusView::from_record_set_sorted(self, status, sort_order)
    }

    pub fn issue_view(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> RenderMaterialManagementIssueView {
        RenderMaterialManagementIssueView::from_record_set(self, issue_kind)
    }

    pub fn issue_view_sorted(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementIssueView {
        RenderMaterialManagementIssueView::from_record_set_sorted(self, issue_kind, sort_order)
    }

    pub fn sorted(&self, sort_order: RenderMaterialManagementSortOrder) -> Self {
        Self::from_sorted_records(self.records.clone(), sort_order)
    }

    pub fn query(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryResult {
        query.apply_to_records(&self.records)
    }

    pub fn query_selection(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQuerySelection {
        RenderMaterialManagementQuerySelection::from_record_set(self, query)
    }

    pub fn select(
        &self,
        material_ids: impl IntoIterator<Item = ResourceId>,
    ) -> RenderMaterialManagementSelection {
        RenderMaterialManagementSelection::from_record_set(self, material_ids)
    }
}
