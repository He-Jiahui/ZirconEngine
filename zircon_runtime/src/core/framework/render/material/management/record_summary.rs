use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementIssueKind, RenderMaterialManagementOverviewRecord,
    RenderMaterialManagementRecord,
};
use crate::core::framework::render::material::readiness_report::{
    RenderMaterialReadinessStatus, RenderMaterialReadinessSummary,
};

/// Aggregate status buckets and issue-row totals for a derived material list.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementRecordSummary {
    #[serde(default)]
    pub status: RenderMaterialReadinessStatus,
    pub total_count: usize,
    pub ready_count: usize,
    /// Number of materials whose readiness status is `Diagnostic`.
    pub diagnostic_count: usize,
    pub fallback_count: usize,
    pub invalid_count: usize,
    #[serde(default)]
    pub validation_error_count: usize,
    #[serde(default)]
    pub fallback_usage_count: usize,
    /// Non-blocking diagnostic issue rows, distinct from `diagnostic_count`.
    #[serde(default)]
    pub diagnostic_row_count: usize,
}

impl RenderMaterialManagementRecordSummary {
    pub fn from_records(records: &[RenderMaterialManagementRecord]) -> Self {
        let mut summary = Self {
            total_count: records.len(),
            ..Self::default()
        };

        for record in records {
            summary.add_issue_rows(record.snapshot.summary);
            match record.status() {
                RenderMaterialReadinessStatus::Ready => summary.ready_count += 1,
                RenderMaterialReadinessStatus::Diagnostic => summary.diagnostic_count += 1,
                RenderMaterialReadinessStatus::Fallback => summary.fallback_count += 1,
                RenderMaterialReadinessStatus::Invalid => summary.invalid_count += 1,
            }
        }

        summary.status = RenderMaterialReadinessStatus::from_issue_counts(
            summary.invalid_count,
            summary.fallback_count,
            summary.diagnostic_count,
        );
        summary
    }

    pub fn from_overview_records(records: &[RenderMaterialManagementOverviewRecord]) -> Self {
        let mut summary = Self {
            total_count: records.len(),
            ..Self::default()
        };

        for record in records {
            summary.add_issue_rows(record.summary);
            match record.status() {
                RenderMaterialReadinessStatus::Ready => summary.ready_count += 1,
                RenderMaterialReadinessStatus::Diagnostic => summary.diagnostic_count += 1,
                RenderMaterialReadinessStatus::Fallback => summary.fallback_count += 1,
                RenderMaterialReadinessStatus::Invalid => summary.invalid_count += 1,
            }
        }

        summary.status = RenderMaterialReadinessStatus::from_issue_counts(
            summary.invalid_count,
            summary.fallback_count,
            summary.diagnostic_count,
        );
        summary
    }

    pub fn degraded_count(&self) -> usize {
        self.diagnostic_count + self.fallback_count + self.invalid_count
    }

    pub fn issue_row_count(&self) -> usize {
        self.validation_error_count + self.fallback_usage_count + self.diagnostic_row_count
    }

    pub fn has_issue_rows(&self) -> bool {
        self.issue_row_count() > 0
    }

    fn add_issue_rows(&mut self, summary: RenderMaterialReadinessSummary) {
        self.validation_error_count += summary.validation_error_count;
        self.fallback_usage_count += summary.fallback_usage_count;
        self.diagnostic_row_count += summary.diagnostic_count;
    }
}

pub(super) fn readiness_summary_has_issue_kind(
    summary: RenderMaterialReadinessSummary,
    issue_kind: RenderMaterialManagementIssueKind,
) -> bool {
    match issue_kind {
        RenderMaterialManagementIssueKind::ValidationError => summary.validation_error_count > 0,
        RenderMaterialManagementIssueKind::FallbackUsage => summary.fallback_usage_count > 0,
        RenderMaterialManagementIssueKind::Diagnostic => summary.diagnostic_count > 0,
    }
}
