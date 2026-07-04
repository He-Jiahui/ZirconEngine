use super::*;

#[test]
fn runtime_15_foundation_row_data_historical_status_is_current() {
    let production_guard_support = read_runtime_src(PRODUCTION_GUARD_SUPPORT_ROWS_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let child_owner_status_anchors = [
        CHILD_OWNER_STATUS_NAME,
        CHILD_OWNER_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_row_data.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_foundation_row_data.rs",
        CHILD_OWNER_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 production support row data",
            production_guard_support.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &child_owner_status_anchors);
    }
}
