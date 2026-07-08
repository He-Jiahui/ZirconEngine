use super::*;

#[path = "status_support_review_guard_rows/export_chain.rs"]
mod export_chain;
#[path = "status_support_review_guard_rows/route_children.rs"]
mod route_children;
#[path = "status_support_review_guard_rows/split_layout.rs"]
mod split_layout;
#[path = "status_support_review_guard_rows/status_current.rs"]
mod status_current;

const REVIEW_GUARD_STATUS_SUPPORT_ROW_GROUPS: &[(&str, &str)] = &[
    ("core_rows", "EXPECTED_STATUS_OUTPUT_SLICES"),
    (
        "status_support_guard_rows",
        "STATUS_SUPPORT_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "typed_error_guard_rows",
        "TYPED_ERROR_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "row_data_guard_rows",
        "ROW_DATA_GUARD_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
    (
        "row_data_owner",
        "ROW_DATA_OWNER_EXPECTED_STATUS_OUTPUT_SLICES",
    ),
];

#[test]
fn runtime_15_review_guard_status_support_review_rows_row_data_owner_is_child_backed() {
    route_children::assert_status_support_review_rows_route_children_are_current();
    export_chain::assert_status_support_review_rows_exports_are_current();
    status_current::assert_status_support_review_rows_row_data_status_is_current();
}
