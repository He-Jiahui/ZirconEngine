use super::*;

#[test]
fn runtime_15_m4_row_data_children_status_mirror_status_rows_are_current() {
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

    let status_anchors = [
        STATUS_MIRROR_CHILD_SPLIT_NAME,
        STATUS_MIRROR_CHILD_SPLIT_ID,
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/status_mirrors.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/status_mirrors/child_split_status.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/status_mirrors/historical_status.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m4_row_data_children/status_mirrors/folder_backed_status.rs",
        STATUS_MIRROR_CHILD_SPLIT_GUARD_NAME,
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
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "Runtime 15 M4 expected status map records M4 row-data children status-mirror split",
        &status_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, STATUS_MIRROR_CHILD_SPLIT_ID],
    );
    assert_contains_all(
        "Runtime 15 M4 expected date map records M4 row-data children status-mirror split",
        &date_map,
        &[STATUS_MIRROR_CHILD_SPLIT_NAME, "2026-07-04"],
    );
}
