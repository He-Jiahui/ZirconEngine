use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementIssueKind, RenderMaterialManagementQuery,
    RenderMaterialManagementQueryResult, RenderMaterialManagementQuerySelection,
};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialManagementQueryFacetKind {
    Status,
    IssueKind,
}

/// Facet row for query panels, scoped to the filtered set before paging.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQueryFacet {
    pub kind: RenderMaterialManagementQueryFacetKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<RenderMaterialReadinessStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_kind: Option<RenderMaterialManagementIssueKind>,
    #[serde(default)]
    pub material_count: usize,
    #[serde(default)]
    pub is_active: bool,
    #[serde(default)]
    pub select_query: RenderMaterialManagementQuery,
}

/// Status and issue-kind facets derived beside a material management query result.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQueryFacets {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub status_facets: Vec<RenderMaterialManagementQueryFacet>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub issue_facets: Vec<RenderMaterialManagementQueryFacet>,
}

impl RenderMaterialManagementQueryFacets {
    pub fn from_query_result(
        query: &RenderMaterialManagementQuery,
        query_result: &RenderMaterialManagementQueryResult,
    ) -> Self {
        let status_facets = status_facet_order()
            .into_iter()
            .map(|status| {
                RenderMaterialManagementQueryFacet::status(
                    status,
                    query_result.status_index.ids_for_status(status).len(),
                    query.status == Some(status),
                    query.status_facet_query(status),
                )
            })
            .collect();
        let issue_facets = issue_facet_order()
            .into_iter()
            .map(|issue_kind| {
                RenderMaterialManagementQueryFacet::issue_kind(
                    issue_kind,
                    query_result
                        .issue_index
                        .ids_for_issue_kind(issue_kind)
                        .len(),
                    query.issue_kind == Some(issue_kind),
                    query.issue_kind_facet_query(issue_kind),
                )
            })
            .collect();

        Self {
            status_facets,
            issue_facets,
        }
    }
}

impl RenderMaterialManagementQueryFacet {
    fn status(
        status: RenderMaterialReadinessStatus,
        material_count: usize,
        is_active: bool,
        select_query: RenderMaterialManagementQuery,
    ) -> Self {
        Self {
            kind: RenderMaterialManagementQueryFacetKind::Status,
            status: Some(status),
            issue_kind: None,
            material_count,
            is_active,
            select_query,
        }
    }

    fn issue_kind(
        issue_kind: RenderMaterialManagementIssueKind,
        material_count: usize,
        is_active: bool,
        select_query: RenderMaterialManagementQuery,
    ) -> Self {
        Self {
            kind: RenderMaterialManagementQueryFacetKind::IssueKind,
            status: None,
            issue_kind: Some(issue_kind),
            material_count,
            is_active,
            select_query,
        }
    }
}

impl RenderMaterialManagementQuery {
    pub fn status_facet_query(&self, status: RenderMaterialReadinessStatus) -> Self {
        let mut query = self.first_page_query();
        query.status = Some(status);
        query
    }

    pub fn issue_kind_facet_query(&self, issue_kind: RenderMaterialManagementIssueKind) -> Self {
        let mut query = self.first_page_query();
        query.issue_kind = Some(issue_kind);
        query
    }
}

impl RenderMaterialManagementQueryResult {
    pub fn facets(
        &self,
        query: &RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryFacets {
        RenderMaterialManagementQueryFacets::from_query_result(query, self)
    }
}

impl RenderMaterialManagementQuerySelection {
    pub fn result_facets(&self) -> RenderMaterialManagementQueryFacets {
        RenderMaterialManagementQueryFacets::from_query_result(&self.query, &self.query_result)
    }
}

fn status_facet_order() -> [RenderMaterialReadinessStatus; 4] {
    [
        RenderMaterialReadinessStatus::Ready,
        RenderMaterialReadinessStatus::Diagnostic,
        RenderMaterialReadinessStatus::Fallback,
        RenderMaterialReadinessStatus::Invalid,
    ]
}

fn issue_facet_order() -> [RenderMaterialManagementIssueKind; 3] {
    [
        RenderMaterialManagementIssueKind::ValidationError,
        RenderMaterialManagementIssueKind::FallbackUsage,
        RenderMaterialManagementIssueKind::Diagnostic,
    ]
}
