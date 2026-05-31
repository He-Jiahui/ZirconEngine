use serde::{Deserialize, Serialize};

use super::{
    readiness_summary_has_issue_kind, RenderMaterialManagementIssueIndex,
    RenderMaterialManagementIssueKind, RenderMaterialManagementOverviewRecord,
    RenderMaterialManagementPageInfo, RenderMaterialManagementPageRequest,
    RenderMaterialManagementRecord, RenderMaterialManagementRecordSummary,
    RenderMaterialManagementSortOrder, RenderMaterialManagementStatusIndex,
};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessStatus;

/// Filter, sort, and page contract for material management list rows.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQuery {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<RenderMaterialReadinessStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_kind: Option<RenderMaterialManagementIssueKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text_filter: Option<String>,
    #[serde(default)]
    pub sort_order: RenderMaterialManagementSortOrder,
    #[serde(default)]
    pub page: RenderMaterialManagementPageRequest,
}

/// Page of compact rows plus aggregate information for the filtered row set.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQueryResult {
    #[serde(default)]
    pub summary: RenderMaterialManagementRecordSummary,
    #[serde(default)]
    pub status_index: RenderMaterialManagementStatusIndex,
    #[serde(default)]
    pub issue_index: RenderMaterialManagementIssueIndex,
    #[serde(default)]
    pub page: RenderMaterialManagementPageInfo,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub records: Vec<RenderMaterialManagementOverviewRecord>,
}

impl RenderMaterialManagementQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_status(mut self, status: RenderMaterialReadinessStatus) -> Self {
        self.status = Some(status);
        self
    }

    pub fn with_issue_kind(mut self, issue_kind: RenderMaterialManagementIssueKind) -> Self {
        self.issue_kind = Some(issue_kind);
        self
    }

    pub fn with_text_filter(mut self, text_filter: impl Into<String>) -> Self {
        let text_filter = text_filter.into();
        let text_filter = text_filter.trim();
        self.text_filter = if text_filter.is_empty() {
            None
        } else {
            Some(text_filter.to_string())
        };
        self
    }

    pub fn with_sort_order(mut self, sort_order: RenderMaterialManagementSortOrder) -> Self {
        self.sort_order = sort_order;
        self
    }

    pub fn with_page(mut self, page: RenderMaterialManagementPageRequest) -> Self {
        self.page = page;
        self
    }

    pub fn apply_to_records(
        &self,
        records: &[RenderMaterialManagementRecord],
    ) -> RenderMaterialManagementQueryResult {
        self.apply_to_overview_records(
            records
                .iter()
                .map(RenderMaterialManagementRecord::overview)
                .collect::<Vec<_>>(),
        )
    }

    pub fn apply_to_overview_records(
        &self,
        records: impl IntoIterator<Item = RenderMaterialManagementOverviewRecord>,
    ) -> RenderMaterialManagementQueryResult {
        let text_filter = self.text_filter.as_deref().and_then(normalize_text_filter);
        let mut records = records
            .into_iter()
            .filter(|record| self.status.map_or(true, |status| record.status() == status))
            .filter(|record| {
                self.issue_kind.map_or(true, |issue_kind| {
                    readiness_summary_has_issue_kind(record.summary, issue_kind)
                })
            })
            .filter(|record| {
                text_filter.as_deref().map_or(true, |filter| {
                    overview_record_matches_text_filter(record, filter)
                })
            })
            .collect::<Vec<_>>();

        self.sort_order.sort_overview_records(&mut records);

        let summary = RenderMaterialManagementRecordSummary::from_overview_records(&records);
        let status_index = RenderMaterialManagementStatusIndex::from_overview_records(&records);
        let issue_index = RenderMaterialManagementIssueIndex::from_overview_records(&records);
        let total_count = records.len();
        let records = page_overview_records(&records, self.page);
        let page = RenderMaterialManagementPageInfo::from_page_request(
            self.page,
            total_count,
            records.len(),
        );

        RenderMaterialManagementQueryResult {
            summary,
            status_index,
            issue_index,
            page,
            records,
        }
    }
}

fn normalize_text_filter(text_filter: &str) -> Option<String> {
    let text_filter = text_filter.trim();
    if text_filter.is_empty() {
        None
    } else {
        Some(text_filter.to_ascii_lowercase())
    }
}

fn overview_record_matches_text_filter(
    record: &RenderMaterialManagementOverviewRecord,
    text_filter: &str,
) -> bool {
    record
        .material_name
        .as_deref()
        .map_or(false, |name| text_matches_filter(name, text_filter))
        || text_matches_filter(&record.material_id.to_string(), text_filter)
}

fn text_matches_filter(value: &str, text_filter: &str) -> bool {
    value.to_ascii_lowercase().contains(text_filter)
}

fn page_overview_records(
    records: &[RenderMaterialManagementOverviewRecord],
    page: RenderMaterialManagementPageRequest,
) -> Vec<RenderMaterialManagementOverviewRecord> {
    let remaining = records.iter().skip(page.offset);
    match page.limit {
        Some(limit) => remaining.take(limit).cloned().collect(),
        None => remaining.cloned().collect(),
    }
}
