use super::*;

#[test]
fn runtime_15_review_guard_status_support_folder_backed_split_layout_guard_is_folder_backed() {
    let parent = read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_CHILD_PATH);
    let route_mounts =
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_ROUTE_MOUNTS_CHILD_PATH);
    let status_current =
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_STATUS_CURRENT_CHILD_PATH);
    let budgets =
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_BUDGETS_CHILD_PATH);
    let split_layout =
        read_runtime_src(STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_SPLIT_LAYOUT_CHILD_PATH);
    let status_rows = review_guard_status_support_review_rows_source_blob();
    let status_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_STATUS_MAP_PATH);
    let date_map = read_runtime_src(REVIEW_GUARD_TYPED_ERROR_DATE_MAP_PATH);

    assert_contains_all(
        "review-guard status-support folder-backed split-layout mounts focused children",
        &parent,
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
    for moved_anchor in [
        "fn runtime_15_review_guard_status_support_folder_backed_guard_is_folder_backed",
        "review-guard status-support rows record folder-backed guard split",
        "should stay under its focused status-support folder-backed budget",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "folder_backed/split_layout.rs should stay a route owner and delegate {moved_anchor}"
        );
    }
    assert_contains_all(
        "review-guard status-support folder-backed split-layout children retain moved checks",
        &format!("{route_mounts}\n{status_current}\n{budgets}\n{split_layout}"),
        &[
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_GUARD_FOLDER_BACKED_GUARD_NAME,
            "runtime_15_review_guard_status_support_folder_backed_guard_status_is_current",
            "runtime_15_review_guard_status_support_folder_backed_guard_budgets_are_current",
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_GUARD_NAME,
        ],
    );

    let status_anchors = [
        STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_NAME,
        STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/folder_backed/split_layout.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/folder_backed/split/route_mounts.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/folder_backed/split/status_current.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/folder_backed/split/budgets.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_support_rows/folder_backed/split/split_layout.rs",
        STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "review-guard status-support rows record folder-backed split-layout guard split",
        &status_rows,
        &status_anchors,
    );
    assert_contains_all(
        "review status map records folder-backed split-layout guard split",
        &status_map,
        &[
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_NAME,
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "review date map records folder-backed split-layout guard split",
        &date_map,
        &[
            STATUS_SUPPORT_ROWS_FOLDER_BACKED_SPLIT_LAYOUT_GUARD_FOLDER_BACKED_STATUS_NAME,
            "2026-07-06",
        ],
    );

    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "Runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "Frameworks 02 plan",
            "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
        ),
        (
            "review findings",
            "docs/plans/engine-code-review-findings-2026-06.md",
        ),
        (
            "structure convention",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "module convention doc",
            "docs/zircon_runtime/structure/module-convention.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
}
