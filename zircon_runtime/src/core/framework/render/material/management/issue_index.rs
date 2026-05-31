use serde::{Deserialize, Serialize};

use super::{RenderMaterialManagementOverviewRecord, RenderMaterialManagementRecord};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessSummary;
use crate::core::resource::ResourceId;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialManagementIssueKind {
    #[default]
    ValidationError,
    FallbackUsage,
    Diagnostic,
}

/// Material ids bucketed by issue-row type; one material may appear in several buckets.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementIssueIndex {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub validation_errors: Vec<ResourceId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub fallback_usages: Vec<ResourceId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub diagnostics: Vec<ResourceId>,
}

impl RenderMaterialManagementIssueIndex {
    pub fn from_records(records: &[RenderMaterialManagementRecord]) -> Self {
        let mut index = Self::default();
        for record in records {
            index.push_summary(record.material_id, record.snapshot.summary);
        }
        index
    }

    pub fn from_overview_records(records: &[RenderMaterialManagementOverviewRecord]) -> Self {
        let mut index = Self::default();
        for record in records {
            index.push_summary(record.material_id, record.summary);
        }
        index
    }

    pub fn push_summary(
        &mut self,
        material_id: ResourceId,
        summary: RenderMaterialReadinessSummary,
    ) {
        if summary.validation_error_count > 0 {
            self.validation_errors.push(material_id);
        }
        if summary.fallback_usage_count > 0 {
            self.fallback_usages.push(material_id);
        }
        if summary.diagnostic_count > 0 {
            self.diagnostics.push(material_id);
        }
    }

    pub fn ids_for_issue_kind(
        &self,
        issue_kind: RenderMaterialManagementIssueKind,
    ) -> &[ResourceId] {
        match issue_kind {
            RenderMaterialManagementIssueKind::ValidationError => &self.validation_errors,
            RenderMaterialManagementIssueKind::FallbackUsage => &self.fallback_usages,
            RenderMaterialManagementIssueKind::Diagnostic => &self.diagnostics,
        }
    }

    pub fn bucket_entry_count(&self) -> usize {
        self.validation_errors.len() + self.fallback_usages.len() + self.diagnostics.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bucket_entry_count() == 0
    }
}
