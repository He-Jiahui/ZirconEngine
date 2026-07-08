use super::*;

#[test]
fn runtime_15_review_guard_direct_assertion_folder_backed_status_is_current() {
    let review_guard_rows = read_runtime_src(STATUS_SUPPORT_REVIEW_GUARD_ROWS_PATH);
    let status_support_expected_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let status_support_expected_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let folder_backed_status_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/status_mirrors.rs",
        FOLDER_BACKED_GUARD_NAME,
        CHILD_OWNER_GUARD_NAME,
        "Cargo gate deferred",
    ];
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
        assert_contains_all(label, &source, &folder_backed_status_anchors);
    }
    assert_contains_all(
        "production guard support rows record direct-assertion folder-backed split",
        &review_guard_rows,
        &folder_backed_status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support expected status map records direct-assertion folder-backed split",
        &status_support_expected_status_map,
        &[FOLDER_BACKED_STATUS_NAME, FOLDER_BACKED_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records direct-assertion folder-backed split",
        &status_support_expected_date_map,
        &[FOLDER_BACKED_STATUS_NAME, "2026-07-02"],
    );
}
