use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementIssueKind, RenderMaterialManagementPageRequest,
    RenderMaterialManagementQuery, RenderMaterialManagementSortOrder,
};
use crate::core::framework::render::material::readiness_report::RenderMaterialReadinessStatus;

/// Normalized query controls for filter badges, clear actions, and page reset.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQueryState {
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
    #[serde(default)]
    pub has_status_filter: bool,
    #[serde(default)]
    pub has_issue_filter: bool,
    #[serde(default)]
    pub has_text_filter: bool,
    #[serde(default)]
    pub has_active_filters: bool,
    #[serde(default)]
    pub is_paged: bool,
}

impl RenderMaterialManagementQueryState {
    pub fn from_query(query: &RenderMaterialManagementQuery) -> Self {
        let text_filter = normalized_text_filter(&query.text_filter);
        let has_status_filter = query.status.is_some();
        let has_issue_filter = query.issue_kind.is_some();
        let has_text_filter = text_filter.is_some();
        Self {
            status: query.status,
            issue_kind: query.issue_kind,
            text_filter,
            sort_order: query.sort_order,
            page: query.page,
            has_status_filter,
            has_issue_filter,
            has_text_filter,
            has_active_filters: has_status_filter || has_issue_filter || has_text_filter,
            is_paged: query.page.offset > 0 || query.page.limit.is_some(),
        }
    }
}

impl RenderMaterialManagementQuery {
    pub fn state(&self) -> RenderMaterialManagementQueryState {
        RenderMaterialManagementQueryState::from_query(self)
    }

    pub fn has_active_filters(&self) -> bool {
        self.status.is_some()
            || self.issue_kind.is_some()
            || normalized_text_filter(&self.text_filter).is_some()
    }

    pub fn is_paged(&self) -> bool {
        self.page.offset > 0 || self.page.limit.is_some()
    }

    pub fn clear_filters(mut self) -> Self {
        self.status = None;
        self.issue_kind = None;
        self.text_filter = None;
        self
    }

    pub fn first_page_query(&self) -> Self {
        let mut query = self.clone();
        query.page.offset = 0;
        query
    }
}

fn normalized_text_filter(text_filter: &Option<String>) -> Option<String> {
    text_filter
        .as_deref()
        .map(str::trim)
        .filter(|text_filter| !text_filter.is_empty())
        .map(str::to_string)
}
