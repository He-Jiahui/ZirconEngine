#[path = "paths/child_group_routes.rs"]
mod child_group_routes;
#[path = "paths/expected_slice_rows.rs"]
mod expected_slice_rows;
#[path = "paths/plan_doc_routes.rs"]
mod plan_doc_routes;
#[path = "paths/priority_plan_doc_routes.rs"]
mod priority_plan_doc_routes;
#[path = "paths/review_guard_routes.rs"]
mod review_guard_routes;
#[path = "paths/row_data_routes.rs"]
mod row_data_routes;
#[path = "paths/runtime_index_anchor_routes.rs"]
mod runtime_index_anchor_routes;

pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_PATH_CHILDREN: &[&str] = &[
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/paths/child_group_routes.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/paths/expected_slice_rows.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/paths/plan_doc_routes.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/paths/priority_plan_doc_routes.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/paths/review_guard_routes.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/paths/row_data_routes.rs",
    "tests/runtime_absorption/structure_convention/test_file_budget/status_slices/maps/rt15/review/status/paths/runtime_index_anchor_routes.rs",
];

pub(super) const STATUS_SUPPORT_CHILD_GROUP_ROW_DATA_ROUTE_CHILDREN: &[&str] =
    child_group_routes::STATUS_SUPPORT_CHILD_GROUP_ROW_DATA_ROUTE_CHILDREN;
pub(super) const DATE_SUPPORT_CHILD_GROUP_ROW_DATA_ROUTE_CHILDREN: &[&str] =
    child_group_routes::DATE_SUPPORT_CHILD_GROUP_ROW_DATA_ROUTE_CHILDREN;
pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_ROWS: &str =
    expected_slice_rows::STATUS_SUPPORT_EXPECTED_SLICE_ROWS;
pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_ROWS_CHILD: &str =
    expected_slice_rows::STATUS_SUPPORT_EXPECTED_SLICE_ROWS_CHILD;
pub(super) const STATUS_SUPPORT_EXPECTED_SLICE_ROW_CHILDREN: &[&str] =
    expected_slice_rows::STATUS_SUPPORT_EXPECTED_SLICE_ROW_CHILDREN;
pub(super) const STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILDREN: &[&str] =
    plan_doc_routes::STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILDREN;
pub(super) const DATE_SUPPORT_PLAN_DOC_ROUTE_CHILDREN: &[&str] =
    plan_doc_routes::DATE_SUPPORT_PLAN_DOC_ROUTE_CHILDREN;
pub(super) const STATUS_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN: &[&str] =
    priority_plan_doc_routes::STATUS_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN;
pub(super) const DATE_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN: &[&str] =
    priority_plan_doc_routes::DATE_SUPPORT_PRIORITY_PLAN_DOC_ROUTE_CHILDREN;
pub(super) const STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILDREN: &[&str] =
    review_guard_routes::STATUS_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILDREN;
pub(super) const DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILDREN: &[&str] =
    review_guard_routes::DATE_SUPPORT_REVIEW_GUARD_ROW_DATA_ROUTE_CHILDREN;
pub(super) const STATUS_SUPPORT_ROW_DATA_ROUTE_CHILDREN: &[&str] =
    row_data_routes::STATUS_SUPPORT_ROW_DATA_ROUTE_CHILDREN;
pub(super) const DATE_SUPPORT_ROW_DATA_ROUTE_CHILDREN: &[&str] =
    row_data_routes::DATE_SUPPORT_ROW_DATA_ROUTE_CHILDREN;
pub(super) const STATUS_SUPPORT_RUNTIME_INDEX_ANCHOR_ROUTE_CHILDREN: &[&str] =
    runtime_index_anchor_routes::STATUS_SUPPORT_RUNTIME_INDEX_ANCHOR_ROUTE_CHILDREN;
pub(super) const DATE_SUPPORT_RUNTIME_INDEX_ANCHOR_ROUTE_CHILDREN: &[&str] =
    runtime_index_anchor_routes::DATE_SUPPORT_RUNTIME_INDEX_ANCHOR_ROUTE_CHILDREN;
