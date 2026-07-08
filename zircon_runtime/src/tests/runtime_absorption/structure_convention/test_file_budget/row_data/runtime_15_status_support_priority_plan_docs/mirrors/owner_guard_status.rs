use super::*;

#[test]
fn runtime_15_priority_plan_docs_owner_guard_status_rows_are_current() {
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps/row_data_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/plan_doc_support_maps/priority_plan_doc_maps/row_data_guard_maps.rs",
    );
    let row_data_owner = read_runtime_src(PRIORITY_ROW_DATA_OWNER_PATH);

    let owner_guard_child_status_anchors = [
        OWNER_GUARD_CHILD_STATUS_NAME,
        OWNER_GUARD_CHILD_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/layout_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs/owner_guards/inventory_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_status_support_priority_plan_docs/row_sources.rs",
        OWNER_GUARD_CHILD_GUARD_NAME,
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
        assert_contains_all(label, &source, &owner_guard_child_status_anchors);
    }
    assert_contains_all(
        "priority-plan-doc row-data owner records owner-guard child split",
        &row_data_owner,
        &owner_guard_child_status_anchors,
    );
    assert_contains_all(
        "Runtime 15 status-support map owns priority-plan-doc owner-guard row-data child split",
        &status_map,
        &[OWNER_GUARD_CHILD_STATUS_NAME, OWNER_GUARD_CHILD_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support date map owns priority-plan-doc owner-guard row-data child split",
        &date_map,
        &[OWNER_GUARD_CHILD_STATUS_NAME, "2026-07-04"],
    );
}
