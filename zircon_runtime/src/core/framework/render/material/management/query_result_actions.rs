use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementQuery, RenderMaterialManagementQueryResult,
    RenderMaterialManagementQueryResultState, RenderMaterialManagementQuerySelection,
};

/// Derived table actions that keep UI controls aligned with query/result state.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQueryResultActions {
    #[serde(default)]
    pub state: RenderMaterialManagementQueryResultState,
    #[serde(default)]
    pub can_clear_filters: bool,
    #[serde(default)]
    pub can_reset_page: bool,
    #[serde(default)]
    pub can_go_to_previous_page: bool,
    #[serde(default)]
    pub can_go_to_next_page: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub clear_filters_query: Option<RenderMaterialManagementQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub first_page_query: Option<RenderMaterialManagementQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub previous_page_query: Option<RenderMaterialManagementQuery>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub next_page_query: Option<RenderMaterialManagementQuery>,
}

impl RenderMaterialManagementQueryResultActions {
    pub fn from_query_result(
        query: &RenderMaterialManagementQuery,
        query_result: &RenderMaterialManagementQueryResult,
    ) -> Self {
        let state =
            RenderMaterialManagementQueryResultState::from_query_result(query, query_result);
        let can_clear_filters = state.query_state.has_active_filters;
        let can_reset_page = query.page.offset > 0;
        let previous_page_query = query.previous_page_query(query_result.page);
        let next_page_query = query.next_page_query(query_result.page);

        Self {
            state,
            can_clear_filters,
            can_reset_page,
            can_go_to_previous_page: previous_page_query.is_some(),
            can_go_to_next_page: next_page_query.is_some(),
            clear_filters_query: can_clear_filters.then(|| query.clone().clear_filters()),
            first_page_query: can_reset_page.then(|| query.first_page_query()),
            previous_page_query,
            next_page_query,
        }
    }
}

impl RenderMaterialManagementQueryResult {
    pub fn actions(
        &self,
        query: &RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryResultActions {
        RenderMaterialManagementQueryResultActions::from_query_result(query, self)
    }
}

impl RenderMaterialManagementQuerySelection {
    pub fn result_actions(&self) -> RenderMaterialManagementQueryResultActions {
        RenderMaterialManagementQueryResultActions::from_query_result(
            &self.query,
            &self.query_result,
        )
    }
}
