use serde::{Deserialize, Serialize};

use super::{
    RenderMaterialManagementQuery, RenderMaterialManagementQueryResult,
    RenderMaterialManagementQuerySelection, RenderMaterialManagementQueryState,
};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RenderMaterialManagementQueryResultStateKind {
    #[default]
    EmptyRecordSet,
    EmptyFilteredSet,
    EmptyPage,
    PageOutOfRange,
    PopulatedPage,
}

/// Table-state read model derived from a query plus its filtered page result.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct RenderMaterialManagementQueryResultState {
    #[serde(default)]
    pub kind: RenderMaterialManagementQueryResultStateKind,
    #[serde(default)]
    pub query_state: RenderMaterialManagementQueryState,
    #[serde(default)]
    pub total_count: usize,
    #[serde(default)]
    pub returned_count: usize,
    #[serde(default)]
    pub has_filtered_records: bool,
    #[serde(default)]
    pub has_page_records: bool,
    #[serde(default)]
    pub is_record_set_empty: bool,
    #[serde(default)]
    pub is_filter_empty: bool,
    #[serde(default)]
    pub is_empty_page: bool,
    #[serde(default)]
    pub is_page_out_of_range: bool,
}

impl RenderMaterialManagementQueryResultState {
    pub fn from_query_result(
        query: &RenderMaterialManagementQuery,
        query_result: &RenderMaterialManagementQueryResult,
    ) -> Self {
        let query_state = query.state();
        let total_count = query_result.page.total_count;
        let returned_count = query_result.page.returned_count;
        let has_filtered_records = total_count > 0;
        let has_page_records = returned_count > 0;
        let is_record_set_empty = !query_state.has_active_filters && total_count == 0;
        let is_filter_empty = query_state.has_active_filters && total_count == 0;
        let is_page_out_of_range = total_count > 0 && query_result.page.offset >= total_count;
        let is_empty_page = total_count > 0 && returned_count == 0 && !is_page_out_of_range;
        let kind = if has_page_records {
            RenderMaterialManagementQueryResultStateKind::PopulatedPage
        } else if is_page_out_of_range {
            RenderMaterialManagementQueryResultStateKind::PageOutOfRange
        } else if is_empty_page {
            RenderMaterialManagementQueryResultStateKind::EmptyPage
        } else if is_filter_empty {
            RenderMaterialManagementQueryResultStateKind::EmptyFilteredSet
        } else {
            RenderMaterialManagementQueryResultStateKind::EmptyRecordSet
        };

        Self {
            kind,
            query_state,
            total_count,
            returned_count,
            has_filtered_records,
            has_page_records,
            is_record_set_empty,
            is_filter_empty,
            is_empty_page,
            is_page_out_of_range,
        }
    }
}

impl RenderMaterialManagementQueryResult {
    pub fn state(
        &self,
        query: &RenderMaterialManagementQuery,
    ) -> RenderMaterialManagementQueryResultState {
        RenderMaterialManagementQueryResultState::from_query_result(query, self)
    }
}

impl RenderMaterialManagementQuerySelection {
    pub fn result_state(&self) -> RenderMaterialManagementQueryResultState {
        RenderMaterialManagementQueryResultState::from_query_result(&self.query, &self.query_result)
    }
}
