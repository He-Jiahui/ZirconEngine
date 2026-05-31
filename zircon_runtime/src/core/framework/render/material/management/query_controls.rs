use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementPageInfo, RenderMaterialManagementPageWindow,
    RenderMaterialManagementQuery, RenderMaterialManagementQueryFacets,
    RenderMaterialManagementQueryFilter, RenderMaterialManagementQueryResult,
    RenderMaterialManagementQueryResultActions, RenderMaterialManagementQueryResultState,
    RenderMaterialManagementQuerySelection, RenderMaterialManagementQueryState,
};

/// Single DTO for material table controls derived from a query result.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQueryControls {
    #[serde(default)]
    pub query_state: RenderMaterialManagementQueryState,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_filters: Vec<RenderMaterialManagementQueryFilter>,
    #[serde(default)]
    pub result_state: RenderMaterialManagementQueryResultState,
    #[serde(default)]
    pub actions: RenderMaterialManagementQueryResultActions,
    #[serde(default)]
    pub facets: RenderMaterialManagementQueryFacets,
    #[serde(default)]
    pub page: RenderMaterialManagementPageInfo,
    #[serde(default)]
    pub page_window: RenderMaterialManagementPageWindow,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_start_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display_end_index: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub current_page_number: Option<usize>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub total_page_count: Option<usize>,
}

impl RenderMaterialManagementQueryControls {
    pub fn from_query_result(
        query: &RenderMaterialManagementQuery,
        query_result: &RenderMaterialManagementQueryResult,
    ) -> Self {
        let actions =
            RenderMaterialManagementQueryResultActions::from_query_result(query, query_result);
        let result_state = actions.state.clone();
        let query_state = result_state.query_state.clone();
        let page = query_result.page;
        Self {
            active_filters: query_state.active_filters(),
            result_state,
            query_state,
            actions,
            facets: query_result.facets(query),
            page,
            page_window: page.window(),
            display_start_index: page.display_start_index(),
            display_end_index: page.display_end_index(),
            current_page_number: page.current_page_number(),
            total_page_count: page.total_page_count(),
        }
    }
}

impl RenderMaterialManagementQueryResult {
    pub fn controls(
        &self,
        query: &RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryControls {
        RenderMaterialManagementQueryControls::from_query_result(query, self)
    }
}

impl RenderMaterialManagementQuerySelection {
    pub fn result_controls(&self) -> RenderMaterialManagementQueryControls {
        RenderMaterialManagementQueryControls::from_query_result(&self.query, &self.query_result)
    }
}
