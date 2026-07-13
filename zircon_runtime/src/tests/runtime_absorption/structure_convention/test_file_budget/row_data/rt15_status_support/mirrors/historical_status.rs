use super::*;

#[test]
fn runtime_15_status_support_row_data_historical_status_is_current() {
    let priority_plan_docs_row_data = read_runtime_src(PRIORITY_PLAN_DOCS_ROW_DATA_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_ROW_DATA_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let child_owner_status_anchors = [
        CHILD_OWNER_STATUS_NAME,
        CHILD_OWNER_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/row_data_and_budget.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/runtime_index_anchors.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/priority_plan_docs.rs",
        CHILD_OWNER_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "Runtime 15 status-support row-data owner",
            priority_plan_docs_row_data.as_str(),
        ),
        ("Runtime 15 expected status map", status_map.as_str()),
        ("Runtime 15 expected date map", date_map.as_str()),
    ] {
        assert_contains_all(label, source, &child_owner_status_anchors);
    }
}
