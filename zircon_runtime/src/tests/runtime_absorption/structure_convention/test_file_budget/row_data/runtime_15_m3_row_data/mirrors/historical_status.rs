use super::*;

#[test]
fn runtime_15_m3_row_data_historical_status_is_current() {
    let status_support_parent = read_runtime_src(RUNTIME_15_M3_STATUS_SUPPORT_ROW_DATA_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let historical_status_anchors = [
        ROW_DATA_SPLIT_STATUS_NAME,
        ROW_DATA_SPLIT_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
        ROW_DATA_SPLIT_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 status-support parent mirror",
            status_support_parent.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &historical_status_anchors);
    }
}
