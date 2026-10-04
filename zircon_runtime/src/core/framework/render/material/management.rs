//! 将资源流送生成的 readiness 快照投影为只读管理视图；筛选、排序和分页不修改材质资产或 GPU 资源。

mod issue_index;
pub use issue_index::{RenderMaterialManagementIssueIndex, RenderMaterialManagementIssueKind};
mod issue_view;
pub use issue_view::RenderMaterialManagementIssueView;
mod overview;
pub use overview::{RenderMaterialManagementOverview, RenderMaterialManagementOverviewRecord};
mod page;
pub use page::{RenderMaterialManagementPageInfo, RenderMaterialManagementPageRequest};
mod page_navigation;
pub use page_navigation::RenderMaterialManagementPageWindow;
mod record;
pub use record::{RenderMaterialManagementRecord, RenderMaterialManagementSnapshot};
mod record_set;
pub use record_set::RenderMaterialManagementRecordSet;
mod record_summary;
use record_summary::readiness_summary_has_issue_kind;
pub use record_summary::RenderMaterialManagementRecordSummary;
mod query;
pub use query::{RenderMaterialManagementQuery, RenderMaterialManagementQueryResult};
mod query_controls;
pub use query_controls::RenderMaterialManagementQueryControls;
mod query_facets;
pub use query_facets::{
    RenderMaterialManagementQueryFacet, RenderMaterialManagementQueryFacetKind,
    RenderMaterialManagementQueryFacets,
};
mod query_filters;
pub use query_filters::{
    RenderMaterialManagementQueryFilter, RenderMaterialManagementQueryFilterKind,
};
mod query_selection;
pub use query_selection::RenderMaterialManagementQuerySelection;
mod query_result_actions;
pub use query_result_actions::RenderMaterialManagementQueryResultActions;
mod query_result_state;
pub use query_result_state::{
    RenderMaterialManagementQueryResultState, RenderMaterialManagementQueryResultStateKind,
};
mod query_state;
pub use query_state::RenderMaterialManagementQueryState;
mod selection;
pub use selection::RenderMaterialManagementSelection;
mod sort_order;
pub use sort_order::{
    RenderMaterialManagementSortDirection, RenderMaterialManagementSortKey,
    RenderMaterialManagementSortOrder,
};
mod status_index;
pub use status_index::RenderMaterialManagementStatusIndex;
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
