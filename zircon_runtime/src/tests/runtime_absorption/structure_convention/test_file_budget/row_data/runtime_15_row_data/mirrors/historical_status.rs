use super::*;

#[test]
fn runtime_15_runtime_15_row_data_historical_status_is_current() {
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let status_support_row_data = read_runtime_src(RUNTIME_15_ROW_DATA_STATUS_ROWS_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");

    let historical_status_anchors = [
        ROW_DATA_SPLIT_STATUS_NAME,
        ROW_DATA_SPLIT_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
        ROW_DATA_SPLIT_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "status-output Runtime 15 M3 status-support row data",
            status_support_row_data.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &historical_status_anchors);
    }
    assert_contains_all(
        "Runtime 15 expected status map owns historical Runtime 15 row-data split",
        &status_map,
        &[ROW_DATA_SPLIT_STATUS_NAME, ROW_DATA_SPLIT_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 expected date map owns historical Runtime 15 row-data split",
        &date_map,
        &[ROW_DATA_SPLIT_STATUS_NAME, "2026-06-23"],
    );
}
