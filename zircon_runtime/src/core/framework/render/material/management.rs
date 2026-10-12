//! 将资源流送生成的 readiness 快照投影为只读管理视图；筛选、排序和分页不修改材质资产或 GPU 资源。

#[path = "management/issue_index.rs"]
mod issue_index;
pub use issue_index::{RenderMaterialManagementIssueIndex, RenderMaterialManagementIssueKind};
#[path = "management/issue_view.rs"]
mod issue_view;
pub use issue_view::RenderMaterialManagementIssueView;
#[path = "management/overview.rs"]
mod overview;
pub use overview::{RenderMaterialManagementOverview, RenderMaterialManagementOverviewRecord};
#[path = "management/page.rs"]
mod page;
pub use page::{RenderMaterialManagementPageInfo, RenderMaterialManagementPageRequest};
#[path = "management/page_navigation.rs"]
mod page_navigation;
pub use page_navigation::RenderMaterialManagementPageWindow;
#[path = "management/record.rs"]
mod record;
pub use record::{RenderMaterialManagementRecord, RenderMaterialManagementSnapshot};
#[path = "management/record_set.rs"]
mod record_set;
pub use record_set::RenderMaterialManagementRecordSet;
#[path = "management/record_summary.rs"]
mod record_summary;
use record_summary::readiness_summary_has_issue_kind;
pub use record_summary::RenderMaterialManagementRecordSummary;
#[path = "management/query.rs"]
mod query;
pub use query::{RenderMaterialManagementQuery, RenderMaterialManagementQueryResult};
#[path = "management/query_controls.rs"]
mod query_controls;
pub use query_controls::RenderMaterialManagementQueryControls;
#[path = "management/query_facets.rs"]
mod query_facets;
pub use query_facets::{
    RenderMaterialManagementQueryFacet, RenderMaterialManagementQueryFacetKind,
    RenderMaterialManagementQueryFacets,
};
#[path = "management/query_filters.rs"]
mod query_filters;
pub use query_filters::{
    RenderMaterialManagementQueryFilter, RenderMaterialManagementQueryFilterKind,
};
#[path = "management/query_selection.rs"]
mod query_selection;
pub use query_selection::RenderMaterialManagementQuerySelection;
#[path = "../../../render/material/management/query_result_actions.rs"]
mod query_result_actions;
pub use query_result_actions::RenderMaterialManagementQueryResultActions;
#[path = "../../../render/material/management/query_result_state.rs"]
mod query_result_state;
pub use query_result_state::{
    RenderMaterialManagementQueryResultState, RenderMaterialManagementQueryResultStateKind,
};
#[path = "management/query_state.rs"]
mod query_state;
pub use query_state::RenderMaterialManagementQueryState;
#[path = "management/selection.rs"]
mod selection;
pub use selection::RenderMaterialManagementSelection;
#[path = "management/sort_order.rs"]
mod sort_order;
pub use sort_order::{
    RenderMaterialManagementSortDirection, RenderMaterialManagementSortKey,
    RenderMaterialManagementSortOrder,
};
#[path = "management/status_index.rs"]
mod status_index;
pub use status_index::RenderMaterialManagementStatusIndex;
#[path = "management/status_view.rs"]
mod status_view;
pub use status_view::RenderMaterialManagementStatusView;

#[cfg(test)]
use crate::core::framework::render::material::{
    RenderMaterialReadinessStatus, RenderMaterialReadinessSummary,
};
#[cfg(test)]
use crate::core::resource::ResourceId;

#[cfg(test)]
#[path = "management/tests/cases.rs"]
mod tests;
