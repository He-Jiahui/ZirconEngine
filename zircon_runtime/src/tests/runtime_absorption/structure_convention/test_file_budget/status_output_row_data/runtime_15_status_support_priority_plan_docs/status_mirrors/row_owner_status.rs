use super::*;

#[test]
fn runtime_15_priority_plan_docs_row_owner_status_rows_are_current() {
    let status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);
    let row_data_owner = read_runtime_src(PRIORITY_ROW_DATA_OWNER_PATH);

    let historical_status_anchors = [
        HISTORICAL_STATUS_NAME,
        HISTORICAL_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/integrity_guards.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/status_followups.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/row_data_owner.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_status_support_priority_plan_docs.rs",
        HISTORICAL_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, path) in [
        (
            "Runtime 15 plan",
            "docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md",
        ),
        (
            "runtime index",
            "docs/plans/zircon_runtime/runtime/index.md",
        ),
        (
            "structure convention plan",
            "docs/plans/engine-code-structure-convention.md",
        ),
        (
            "review findings plan",
            "docs/plans/engine-code-review-findings-2026-06.md",
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
        assert_contains_all(label, &source, &historical_status_anchors);
    }
    assert_contains_all(
        "priority-plan-doc row-data owner records historical guard row",
        &row_data_owner,
        &historical_status_anchors,
    );
    assert_contains_all(
        "status expected-slice map owns historical priority-plan-doc guard row",
        &status_map,
        &[HISTORICAL_STATUS_NAME, HISTORICAL_STATUS_ID],
    );
    assert_contains_all(
        "date expected-slice map owns historical priority-plan-doc guard row",
        &date_map,
        &[HISTORICAL_STATUS_NAME, "2026-07-02"],
    );
}
