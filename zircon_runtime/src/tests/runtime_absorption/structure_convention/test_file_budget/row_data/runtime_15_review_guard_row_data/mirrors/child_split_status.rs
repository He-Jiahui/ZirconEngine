use super::*;

#[test]
fn runtime_15_review_guard_row_data_status_mirror_status_rows_are_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let status_support_rows = read_runtime_src(STATUS_SUPPORT_ROWS_PATH);
    let status_support_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let status_support_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let status_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/mirrors/child_split_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/mirrors/folder_backed_status.rs",
        STATUS_MIRROR_CHILD_SPLIT_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "production guard support rows",
            status_support_rows.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status-support expected status map records review-guard row-data status-mirror split",
        &status_support_status_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, STATUS_MIRROR_CHILD_SPLIT_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records review-guard row-data status-mirror split",
        &status_support_date_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, "2026-07-04"],
    );
}
