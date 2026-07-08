use super::*;

#[test]
fn runtime_15_m3_child_group_moved_row_folder_backed_status_mirrors_are_current() {
    let status_rows = read_runtime_src(STATUS_SUPPORT_STATUS_DOCS_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "Runtime 15 status-support map owns M3 child-group moved-row folder-backed split",
        &status_map,
        &[FOLDER_BACKED_STATUS_NAME, FOLDER_BACKED_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns M3 child-group moved-row folder-backed split",
        &date_map,
        &[FOLDER_BACKED_STATUS_NAME, "2026-07-03"],
    );

    let folder_backed_status_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/lock_poison_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/module_convention_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/review_top_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m3_child_group_moved_rows/budgets.rs",
        FOLDER_BACKED_GUARD_NAME,
        CHILD_OWNER_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        (
            "status-output Runtime 15 M3 production support status-doc rows",
            status_rows.as_str(),
        ),
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("runtime implementation session", session_note.as_str()),
    ] {
        assert_contains_all(label, source, &folder_backed_status_anchors);
    }
}
