use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_row_data_status_maps_are_child_owned() {
    let row_data_owner_rows = read_runtime_src(DIRECT_ASSERTION_ROW_DATA_OWNER_ROWS_PATH);
    let status_support_expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let status_support_expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        ROW_DATA_FOLDER_BACKED_STATUS_NAME,
        ROW_DATA_FOLDER_BACKED_STATUS_ID,
        DIRECT_ASSERTION_ROWS_PATH,
        DIRECT_ASSERTION_CORE_ROWS_PATH,
        DIRECT_ASSERTION_F12_ROWS_PATH,
        DIRECT_ASSERTION_ROOT_PARENT_ROWS_PATH,
        DIRECT_ASSERTION_RENDER_ROWS_PATH,
        DIRECT_ASSERTION_F8_ROWS_PATH,
        DIRECT_ASSERTION_P0_ROWS_PATH,
        DIRECT_ASSERTION_ROW_DATA_OWNER_ROWS_PATH,
        ROW_DATA_FOLDER_BACKED_GUARD_NAME,
        "scoped rustfmt/static scans passed",
        "Cargo gate deferred",
    ];
    assert_contains_all(
        "direct-assertion row-data owner rows record folder-backed split",
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
        "Runtime 15 status-support expected status map records direct-assertion row-data split",
        &status_support_expected_status_map,
        &[
            ROW_DATA_FOLDER_BACKED_STATUS_NAME,
            ROW_DATA_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records direct-assertion row-data split",
        &status_support_expected_date_map,
        &[ROW_DATA_FOLDER_BACKED_STATUS_NAME, "2026-07-05"],
    );
}
