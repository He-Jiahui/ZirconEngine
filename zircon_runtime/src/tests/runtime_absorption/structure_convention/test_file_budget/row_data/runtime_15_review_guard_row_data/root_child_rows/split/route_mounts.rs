use super::*;

#[test]
fn runtime_15_review_guard_row_data_root_child_rows_are_folder_backed() {
    let parent = read_runtime_src(ROOT_CHILD_ROWS_PATH);
    let top_level = read_runtime_src(ROOT_CHILD_ROWS_TOP_LEVEL_CHILD_PATH);
    let delegation = read_runtime_src(ROOT_CHILD_ROWS_DELEGATION_CHILD_PATH);
    let typed_error_rows = read_runtime_src(ROOT_CHILD_ROWS_TYPED_ERROR_ROWS_CHILD_PATH);
    let aggregation = read_runtime_src(ROOT_CHILD_ROWS_AGGREGATION_CHILD_PATH);
    let split_layout = read_runtime_src(ROOT_CHILD_ROWS_SPLIT_LAYOUT_CHILD_PATH);
    let route_mounts = read_runtime_src(ROOT_CHILD_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH);

    assert_contains_all(
        "review-guard row-data root child rows mounts focused children",
        &parent,
        &[
            "#[path = \"root_child_rows/aggregation.rs\"]",
            "mod aggregation;",
            "#[path = \"root_child_rows/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"root_child_rows/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"root_child_rows/top_level.rs\"]",
            "mod top_level;",
            "#[path = \"root_child_rows/typed_error_rows.rs\"]",
            "mod typed_error_rows;",
        ],
    );
    for moved_anchor in [
        "pub(super) const REVIEW_GUARD_ROW_DATA_CHILDREN",
        "pub(super) const REVIEW_GUARD_ROW_DATA_DELEGATION_CHILDREN",
        "pub(super) const REVIEW_GUARD_TYPED_ERROR_ROWS_GUARD_CHILDREN",
        "pub(super) const REVIEW_GUARD_ROW_DATA_AGGREGATION_CHILDREN",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "root_child_rows.rs should stay a route owner and delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-guard root child row owners retain moved inventories",
        &format!(
            "{top_level}\n{delegation}\n{typed_error_rows}\n{aggregation}\n{split_layout}\n{route_mounts}"
        ),
        &[
            "REVIEW_GUARD_ROW_DATA_CHILDREN",
            "REVIEW_GUARD_ROW_DATA_DELEGATION_CHILDREN",
            "REVIEW_GUARD_TYPED_ERROR_ROWS_GUARD_CHILDREN",
            "REVIEW_GUARD_ROW_DATA_AGGREGATION_CHILDREN",
            ROOT_CHILD_ROWS_FOLDER_BACKED_GUARD_NAME,
        ],
    );
    assert_contains_all(
        "review-guard root child rows split-layout routes focused children",
        &split_layout,
        &[
            "#[path = \"split/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"split/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"split/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"split/status_current.rs\"]",
            "mod status_current;",
        ],
    );
}
