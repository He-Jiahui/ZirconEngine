use super::*;

pub(in super::super) const REVIEW_GUARD_TYPED_ERROR_ROWS_GUARD_CHILDREN: &[(&str, &str, &str)] = &[
    (
        "route_children",
        TYPED_ERROR_ROWS_ROUTE_CHILDREN_PATH,
        TYPED_ERROR_ROW_DATA_GUARD_NAME,
    ),
    (
        "representative_rows",
        TYPED_ERROR_ROWS_REPRESENTATIVE_ROWS_PATH,
        "runtime_15_review_guard_typed_error_child_rows_keep_representative_anchors",
    ),
    (
        "export_chain",
        TYPED_ERROR_ROWS_EXPORT_CHAIN_PATH,
        "runtime_15_review_guard_typed_error_row_groups_export_through_status_chain",
    ),
    (
        "status_mirrors",
        TYPED_ERROR_ROWS_STATUS_MIRRORS_PATH,
        "runtime_15_review_guard_typed_error_row_data_status_mirrors_are_current",
    ),
    (
        "split_layout",
        TYPED_ERROR_ROWS_SPLIT_LAYOUT_PATH,
        TYPED_ERROR_ROWS_GUARD_FOLDER_BACKED_GUARD_NAME,
    ),
];
