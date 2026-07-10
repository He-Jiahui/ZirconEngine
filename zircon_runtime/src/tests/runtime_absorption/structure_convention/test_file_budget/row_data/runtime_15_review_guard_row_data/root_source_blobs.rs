use super::*;

pub(super) fn review_guard_row_data_child_source_blob() -> String {
    let mut blob = String::new();
    for (_, path, _) in REVIEW_GUARD_ROW_DATA_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    for (_, path, _) in REVIEW_GUARD_ROW_DATA_DELEGATION_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    for path in [
        DELEGATION_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH,
        DELEGATION_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH,
        DELEGATION_SPLIT_LAYOUT_BUDGETS_CHILD_PATH,
        DELEGATION_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH,
    ] {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    for (_, path, _) in REVIEW_GUARD_TYPED_ERROR_ROWS_GUARD_CHILDREN {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    for path in [
        TYPED_ERROR_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH,
        TYPED_ERROR_ROWS_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH,
        TYPED_ERROR_ROWS_SPLIT_LAYOUT_BUDGETS_CHILD_PATH,
        TYPED_ERROR_ROWS_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH,
    ] {
        blob.push_str(&read_runtime_src(path));
        blob.push('\n');
    }
    blob
}

pub(super) fn review_guard_row_data_root_child_rows_source_blob() -> String {
    [
        read_runtime_src(ROOT_CHILD_ROWS_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_TOP_LEVEL_CHILD_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_DELEGATION_CHILD_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_TYPED_ERROR_ROWS_CHILD_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_AGGREGATION_CHILD_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_SPLIT_LAYOUT_CHILD_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_SPLIT_LAYOUT_BUDGETS_CHILD_PATH),
        read_runtime_src(ROOT_CHILD_ROWS_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH),
    ]
    .join("\n")
}

pub(super) fn status_support_rows_guard_child_source_blob() -> String {
    [
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_ROW_LAYOUT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_STATUS_CURRENT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_BUDGETS_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_ROW_CLEANUP_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_STATUS_CURRENT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_ANCHOR_MIRROR_CLEANUP_SPLIT_LAYOUT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_BUDGETS_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH),
    ]
    .join("\n")
}

pub(super) fn review_guard_status_support_review_rows_source_blob() -> String {
    [
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_ROWS_PATH),
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_REVIEW_ROWS_CORE_PATH),
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_REVIEW_ROWS_STATUS_SUPPORT_GUARD_PATH),
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_REVIEW_ROWS_TYPED_ERROR_GUARD_PATH),
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_GUARD_PATH),
        read_runtime_src(REVIEW_GUARD_STATUS_SUPPORT_REVIEW_ROWS_ROW_DATA_OWNER_PATH),
    ]
    .join("\n")
}

pub(super) fn status_support_review_rows_guard_child_source_blob() -> String {
    [
        read_runtime_src(STATUS_SUPPORT_REVIEW_ROWS_ROUTE_CHILDREN_PATH),
        read_runtime_src(STATUS_SUPPORT_REVIEW_ROWS_EXPORT_CHAIN_PATH),
        read_runtime_src(STATUS_SUPPORT_REVIEW_ROWS_STATUS_CURRENT_PATH),
        read_runtime_src(STATUS_SUPPORT_REVIEW_ROWS_SPLIT_LAYOUT_PATH),
    ]
    .join("\n")
}

pub(super) fn moved_rows_child_source_blob() -> String {
    [
        MOVED_ROWS_DELEGATION_PATH,
        MOVED_ROWS_CODE_REVIEW_PATH,
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/source_delegation.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/review_guard_rows.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/structure_guard_rows.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/typed_error_structure_rows.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data_moved_rows/code_review_rows/plugin_importer_rows.rs",
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support/review_guard/moved_row_rows.rs",
        MOVED_ROWS_TYPED_ERROR_PATH,
        MOVED_ROWS_STATUS_MIRRORS_PATH,
    ].iter().map(|path| read_runtime_src(path)).collect::<Vec<_>>().join("\n")
}
