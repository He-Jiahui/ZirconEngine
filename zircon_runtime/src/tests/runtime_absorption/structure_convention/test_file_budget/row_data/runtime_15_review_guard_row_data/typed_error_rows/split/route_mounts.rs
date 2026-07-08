use super::*;

#[test]
fn runtime_15_review_guard_typed_error_rows_guard_is_folder_backed() {
    let parent = read_runtime_src(TYPED_ERROR_ROWS_GUARD_PATH);
    let route_children = read_runtime_src(TYPED_ERROR_ROWS_ROUTE_CHILDREN_PATH);
    let representative_rows = read_runtime_src(TYPED_ERROR_ROWS_REPRESENTATIVE_ROWS_PATH);
    let export_chain = read_runtime_src(TYPED_ERROR_ROWS_EXPORT_CHAIN_PATH);
    let status_mirrors = read_runtime_src(TYPED_ERROR_ROWS_STATUS_MIRRORS_PATH);
    let split_layout = read_runtime_src(TYPED_ERROR_ROWS_SPLIT_LAYOUT_PATH);
    let split_layout_children = [
        read_runtime_src(TYPED_ERROR_ROWS_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH),
        read_runtime_src(TYPED_ERROR_ROWS_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH),
        read_runtime_src(TYPED_ERROR_ROWS_SPLIT_LAYOUT_BUDGETS_CHILD_PATH),
        read_runtime_src(TYPED_ERROR_ROWS_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH),
    ]
    .join("\n");

    assert_contains_all(
        "review-guard typed-error rows guard mounts focused children",
        &parent,
        &[
            "#[path = \"typed_error_rows/export_chain.rs\"]",
            "mod export_chain;",
            "#[path = \"typed_error_rows/representative_rows.rs\"]",
            "mod representative_rows;",
            "#[path = \"typed_error_rows/route_children.rs\"]",
            "mod route_children;",
            "#[path = \"typed_error_rows/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"typed_error_rows/status_mirrors.rs\"]",
            "mod status_mirrors;",
        ],
    );
    for moved_anchor in [
        "let native_plugin_rows = read_runtime_src",
        "typed-error row-data children own representative rows",
        "typed-error row groups are exported through the status-output chain",
        "typed-error row-data split is mirrored in docs and session state",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "typed_error_rows.rs should stay a route owner and delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-guard typed-error rows guard children retain moved checks",
        &format!(
            "{route_children}\n{representative_rows}\n{export_chain}\n{status_mirrors}\n{split_layout}\n{split_layout_children}"
        ),
        &[
            TYPED_ERROR_ROW_DATA_GUARD_NAME,
            "runtime_15_review_guard_typed_error_child_rows_keep_representative_anchors",
            "runtime_15_review_guard_typed_error_row_groups_export_through_status_chain",
            "runtime_15_review_guard_typed_error_row_data_status_mirrors_are_current",
            TYPED_ERROR_ROWS_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
}
