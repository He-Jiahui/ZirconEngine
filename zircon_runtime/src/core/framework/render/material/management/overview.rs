use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementIssueIndex, RenderMaterialManagementIssueKind,
    RenderMaterialManagementIssueView, RenderMaterialManagementQuery,
    RenderMaterialManagementQueryResult, RenderMaterialManagementRecord,
    RenderMaterialManagementRecordSet, RenderMaterialManagementRecordSummary,
    RenderMaterialManagementSortOrder, RenderMaterialManagementStatusIndex,
    RenderMaterialManagementStatusView,
};
use crate::core::framework::render::material::readiness_report::{
    RenderMaterialReadinessStatus, RenderMaterialReadinessSummary,
};
use crate::core::resource::ResourceId;

/// Compact row for list views that only need identity and readiness summary.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementOverviewRecord {
    pub material_id: ResourceId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub material_name: Option<String>,
    #[serde(default)]
    pub summary: RenderMaterialReadinessSummary,
}

/// Compact list payload for material management headers and table rows.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementOverview {
    #[serde(default)]
    pub summary: RenderMaterialManagementRecordSummary,
    #[serde(default)]
    pub status_index: RenderMaterialManagementStatusIndex,
    #[serde(default)]
    pub issue_index: RenderMaterialManagementIssueIndex,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<RenderMaterialManagementOverviewRecord>,
}

impl RenderMaterialManagementOverview {
    pub fn from_records(records: &[RenderMaterialManagementRecord]) -> Self {
        let summary = RenderMaterialManagementRecordSummary::from_records(records);
        let status_index = RenderMaterialManagementStatusIndex::from_records(records);
        let issue_index = RenderMaterialManagementIssueIndex::from_records(records);
        Self {
            summary,
            status_index,
            issue_index,
            records: records
                .iter()
                .map(RenderMaterialManagementRecord::overview)
                .collect(),
        }
    }

    pub fn from_records_sorted(
        records: &[RenderMaterialManagementRecord],
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        Self::from_records(records).sorted(sort_order)
    }

    pub fn from_record_set(record_set: &RenderMaterialManagementRecordSet) -> Self {
        Self {
            summary: record_set.summary,
            status_index: record_set.status_index.clone(),
            issue_index: record_set.issue_index.clone(),
            records: record_set
                .records
                .iter()
                .map(RenderMaterialManagementRecord::overview)
                .collect(),
        }
    }

    pub fn from_record_set_sorted(
        record_set: &RenderMaterialManagementRecordSet,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        Self::from_record_set(record_set).sorted(sort_order)
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn status_view(
        &self,
        status: RenderMaterialReadinessStatus,
    ) -> RenderMaterialManagementStatusView {
        RenderMaterialManagementStatusView::from_overview(self, status)
    }

    pub fn status_view_sorted(
        &self,
        status: RenderMaterialReadinessStatus,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementStatusView {
        RenderMaterialManagementStatusView::from_overview_sorted(self, status, sort_order)
    }

    pub fn issue_view(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> RenderMaterialManagementIssueView {
        RenderMaterialManagementIssueView::from_overview(self, issue_kind)
    }

    pub fn issue_view_sorted(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> RenderMaterialManagementIssueView {
        RenderMaterialManagementIssueView::from_overview_sorted(self, issue_kind, sort_order)
    }

    pub fn sorted(&self, sort_order: RenderMaterialManagementSortOrder) -> Self {
        let mut records = self.records.clone();
        sort_order.sort_overview_records(&mut records);
        Self {
            summary: self.summary,
            status_index: RenderMaterialManagementStatusIndex::from_overview_records(&records),
            issue_index: RenderMaterialManagementIssueIndex::from_overview_records(&records),
            records,
        }
    }

    pub fn query(
        &self,
        query: RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryResult {
        query.apply_to_overview_records(self.records.clone())
    }
}

impl RenderMaterialManagementOverviewRecord {
    pub fn status(&self) -> RenderMaterialReadinessStatus {
        self.summary.status
    }

    pub fn is_ready(&self) -> bool {
        self.summary.is_ready
    }
}
