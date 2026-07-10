use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_row_ownership_status_is_current() {
    let row_data_owner_rows = read_runtime_src(DIRECT_ASSERTION_ROW_DATA_OWNER_ROWS_PATH);
    let status_support_expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let status_support_expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        ROW_OWNERSHIP_CHILD_SPLIT_STATUS_NAME,
        ROW_OWNERSHIP_CHILD_SPLIT_STATUS_ID,
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/child_owner_rows.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/folder_backed_rows.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/folder_backed_rows/exports.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/folder_backed_rows/status_maps.rs",
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership/status_current.rs",
        CHILD_OWNER_GUARD_NAME,
        ROW_DATA_FOLDER_BACKED_GUARD_NAME,
        ROW_OWNERSHIP_CHILD_SPLIT_GUARD_NAME,
        "scoped rustfmt/static scans passed",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "direct-assertion row-ownership child split is recorded in status rows",
        &row_data_owner_rows,
        &status_anchors,
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
        (
            "session note",
            ".codex/sessions/20260612-0847-runtime-architecture-implementation.md",
        ),
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status-support expected status map records row-ownership child split",
        &status_support_expected_status_map,
        &[
            ROW_OWNERSHIP_CHILD_SPLIT_STATUS_NAME,
            ROW_OWNERSHIP_CHILD_SPLIT_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records row-ownership child split",
        &status_support_expected_date_map,
        &[ROW_OWNERSHIP_CHILD_SPLIT_STATUS_NAME, "2026-07-05"],
    );
}
