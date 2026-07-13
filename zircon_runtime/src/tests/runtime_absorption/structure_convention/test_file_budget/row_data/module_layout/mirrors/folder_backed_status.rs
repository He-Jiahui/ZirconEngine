use super::*;

#[test]
fn runtime_15_module_layout_folder_backed_status_rows_are_current() {
    let production_guard_support = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let folder_backed_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/module_layout.rs",
        "structure_convention/test_file_budget/row_data/module_layout/delegation.rs",
        "structure_convention/test_file_budget/row_data/module_layout/child_summaries.rs",
        "structure_convention/test_file_budget/row_data/module_layout/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/module_layout/budgets.rs",
        FOLDER_BACKED_GUARD_NAME,
        HISTORICAL_GUARD_NAME,
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
    ] {
        let source = read_repo(path);
        assert_contains_all(label, &source, &folder_backed_anchors);
    }
    assert_contains_all(
        "status-output Runtime 15 M3 production support row data records module-layout folder-backed split",
        &production_guard_support,
        &folder_backed_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support map owns module-layout guard folder-backed split",
        &status_map,
        &[FOLDER_BACKED_STATUS_NAME, FOLDER_BACKED_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns module-layout guard folder-backed split",
        &date_map,
        &[FOLDER_BACKED_STATUS_NAME, "2026-07-03"],
    );
}
