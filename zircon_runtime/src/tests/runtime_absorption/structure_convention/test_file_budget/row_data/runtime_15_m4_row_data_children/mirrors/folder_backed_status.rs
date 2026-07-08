use super::*;

#[test]
fn runtime_15_m4_row_data_children_guard_folder_backed_status_mirrors_are_current() {
    let row_data_owner = read_runtime_src(RUNTIME_15_M4_ROW_DATA_OWNER_PATH);
    let status_map = read_runtime_src(M4_STATUS_MAP_PATH);
    let date_map = read_runtime_src(M4_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    assert_contains_all(
        "Runtime 15 M4 expected status map owns M4 row-data children guard folder-backed split",
        &status_map,
        &[FOLDER_BACKED_STATUS_NAME, FOLDER_BACKED_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 M4 expected date map owns M4 row-data children guard folder-backed split",
        &date_map,
        &[FOLDER_BACKED_STATUS_NAME, "2026-07-03"],
    );

    let folder_backed_status_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/row_ownership.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_m4_row_data_children/budgets.rs",
        FOLDER_BACKED_GUARD_NAME,
        ROW_DATA_OWNER_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        ("Runtime 15 M4 status row data", row_data_owner.as_str()),
    ] {
        assert_contains_all(label, source, &folder_backed_status_anchors);
    }
}
