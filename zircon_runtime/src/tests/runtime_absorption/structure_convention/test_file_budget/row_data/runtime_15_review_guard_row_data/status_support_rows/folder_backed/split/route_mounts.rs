use super::*;

#[test]
fn runtime_15_review_guard_status_support_folder_backed_guard_is_folder_backed() {
    let parent = read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_CHILD_PATH);
    let row_layout = read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_ROW_LAYOUT_CHILD_PATH);
    let status_current =
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_STATUS_CURRENT_CHILD_PATH);
    let split_layout = read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_CHILD_PATH);
    let split_layout_children = [
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_BUDGETS_CHILD_PATH),
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH),
    ]
    .join("\n");

    assert_contains_all(
        "review-guard status-support folder-backed guard mounts focused children",
        &parent,
        &[
            "#[path = \"folder_backed/row_layout.rs\"]",
            "mod row_layout;",
            "#[path = \"folder_backed/split_layout.rs\"]",
            "mod split_layout;",
            "#[path = \"folder_backed/status_current.rs\"]",
            "mod status_current;",
        ],
    );
    for moved_anchor in [
        "fn runtime_15_review_guard_status_support_rows_are_folder_backed",
        "status-support children retain representative row topics",
        "review-guard status-support row data records its folder-backed split",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "folder_backed.rs should stay a route owner and delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-guard status-support folder-backed children retain moved checks",
        &format!("{row_layout}\n{status_current}\n{split_layout}\n{split_layout_children}"),
        &[
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_GUARD_NAME,
            "runtime_15_review_guard_status_support_folder_backed_status_is_current",
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );
}
