use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementIssueKind, RenderMaterialManagementQuery,
    RenderMaterialManagementQueryState,
};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessStatus;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialManagementQueryFilterKind {
    Status,
    IssueKind,
    Text,
}

/// Active query filter row with the exact query needed to remove that filter.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQueryFilter {
    pub kind: RenderMaterialManagementQueryFilterKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub status: Option<RenderMaterialReadinessStatus>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub issue_kind: Option<RenderMaterialManagementIssueKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub text: Option<String>,
    #[serde(default)]
    pub remove_query: RenderMaterialManagementQuery,
}

impl RenderMaterialManagementQueryFilter {
    fn status(
        status: RenderMaterialReadinessStatus,
        remove_query: RenderMaterialManagementQuery,
    ) -> Self {
        Self {
            kind: RenderMaterialManagementQueryFilterKind::Status,
            status: Some(status),
            issue_kind: None,
            text: None,
            remove_query,
        }
    }

    fn issue_kind(
        issue_kind: RenderMaterialManagementIssueKind,
        remove_query: RenderMaterialManagementQuery,
    ) -> Self {
        Self {
            kind: RenderMaterialManagementQueryFilterKind::IssueKind,
            status: None,
            issue_kind: Some(issue_kind),
            text: None,
            remove_query,
        }
    }

    fn text(text: String, remove_query: RenderMaterialManagementQuery) -> Self {
        Self {
            kind: RenderMaterialManagementQueryFilterKind::Text,
            status: None,
            issue_kind: None,
            text: Some(text),
            remove_query,
        }
    }
}

impl RenderMaterialManagementQuery {
    pub fn active_filters(&self) -> Vec<RenderMaterialManagementQueryFilter> {
        let mut filters = Vec::new();
        if let Some(status) = self.status {
            filters.push(RenderMaterialManagementQueryFilter::status(
                status,
                self.without_status_filter(),
            ));
        }
        if let Some(issue_kind) = self.issue_kind {
            filters.push(RenderMaterialManagementQueryFilter::issue_kind(
                issue_kind,
                self.without_issue_kind_filter(),
            ));
        }
        if let Some(text) = normalized_text_filter(&self.text_filter) {
            filters.push(RenderMaterialManagementQueryFilter::text(
                text,
                self.without_text_filter(),
            ));
        }
        filters
    }

    pub fn without_status_filter(&self) -> Self {
        let mut query = self.clone().first_page_query();
        query.status = None;
        query
    }

    pub fn without_issue_kind_filter(&self) -> Self {
        let mut query = self.clone().first_page_query();
        query.issue_kind = None;
        query
    }

    pub fn without_text_filter(&self) -> Self {
        let mut query = self.clone().first_page_query();
        query.text_filter = None;
        query
    }
}

impl RenderMaterialManagementQueryState {
    pub fn active_filters(&self) -> Vec<RenderMaterialManagementQueryFilter> {
        RenderMaterialManagementQuery {
            status: self.status,
            issue_kind: self.issue_kind,
            text_filter: self.text_filter.clone(),
            sort_order: self.sort_order,
            page: self.page,
        }
        .active_filters()
    }
}

fn normalized_text_filter(text_filter: &Option<String>) -> Option<String> {
    text_filter
        .as_deref()
        .map(str::trim)
        .filter(|text_filter| !text_filter.is_empty())
        .map(str::to_string)
}
