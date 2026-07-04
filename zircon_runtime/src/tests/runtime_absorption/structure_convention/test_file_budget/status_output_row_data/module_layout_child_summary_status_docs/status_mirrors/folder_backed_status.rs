use super::*;

#[test]
fn runtime_15_module_layout_child_summary_status_docs_folder_backed_status_mirrors_are_current() {
    let production_guard_support =
        read_runtime_src(PRODUCTION_GUARD_SUPPORT_MODULE_LAYOUT_ROWS_PATH);
    let expected_status_map = read_runtime_src(EXPECTED_STATUS_MAP_PATH);
    let expected_date_map = read_runtime_src(EXPECTED_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let folder_backed_status_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/delegation.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/source_ownership.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/status_mirrors.rs",
        "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs/budgets.rs",
        FOLDER_BACKED_GUARD_NAME,
        HISTORICAL_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "status-output Runtime 15 M3 production support row data",
            production_guard_support.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &folder_backed_status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status-support map owns module-layout child-summary status-doc guard folder-backed split",
        &expected_status_map,
        &[FOLDER_BACKED_STATUS_NAME, FOLDER_BACKED_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns module-layout child-summary status-doc guard folder-backed split",
        &expected_date_map,
        &[FOLDER_BACKED_STATUS_NAME, "2026-07-03"],
    );
}
