use serde::{Deserialize, Serialize};

use super::{
    readiness_summary_has_issue_kind, RenderMaterialManagementIssueKind,
    RenderMaterialManagementOverview, RenderMaterialManagementOverviewRecord,
    RenderMaterialManagementRecord, RenderMaterialManagementRecordSet,
    RenderMaterialManagementSortOrder,
};
use crate::core::resource::ResourceId;

/// Compact rows for one issue-row bucket, preserving the source display order.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementIssueView {
    #[serde(default)]
    pub issue_kind: RenderMaterialManagementIssueKind,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub material_ids: Vec<ResourceId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<RenderMaterialManagementOverviewRecord>,
}

impl RenderMaterialManagementIssueView {
    pub fn from_records(
        records: &[RenderMaterialManagementRecord],
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> Self {
        let records = records
            .iter()
            .filter(|record| readiness_summary_has_issue_kind(record.snapshot.summary, issue_kind))
            .map(RenderMaterialManagementRecord::overview)
            .collect::<Vec<_>>();
        Self::from_overview_records(records, issue_kind)
    }

    pub fn from_records_sorted(
        records: &[RenderMaterialManagementRecord],
        issue_kind: RenderMaterialManagementIssueKind,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        Self::from_records(records, issue_kind).sorted(sort_order)
    }

    pub fn from_overview(
        overview: &RenderMaterialManagementOverview,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> Self {
        let records = overview
            .records
            .iter()
            .filter(|record| readiness_summary_has_issue_kind(record.summary, issue_kind))
            .cloned()
            .collect();
        Self::from_overview_records(records, issue_kind)
    }

    pub fn from_overview_sorted(
        overview: &RenderMaterialManagementOverview,
        issue_kind: RenderMaterialManagementIssueKind,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        Self::from_overview(overview, issue_kind).sorted(sort_order)
    }

    pub fn from_record_set(
        record_set: &RenderMaterialManagementRecordSet,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> Self {
        Self::from_records(&record_set.records, issue_kind)
    }

    pub fn from_record_set_sorted(
        record_set: &RenderMaterialManagementRecordSet,
        issue_kind: RenderMaterialManagementIssueKind,
        sort_order: RenderMaterialManagementSortOrder,
    ) -> Self {
        Self::from_record_set(record_set, issue_kind).sorted(sort_order)
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }

    pub fn len(&self) -> usize {
        self.records.len()
    }

    pub fn sorted(&self, sort_order: RenderMaterialManagementSortOrder) -> Self {
        let mut records = self.records.clone();
        sort_order.sort_overview_records(&mut records);
        Self::from_overview_records(records, self.issue_kind)
    }

    fn from_overview_records(
        records: Vec<RenderMaterialManagementOverviewRecord>,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> Self {
        let material_ids = records.iter().map(|record| record.material_id).collect();
        Self {
            issue_kind,
            material_ids,
            records,
        }
    }
}
