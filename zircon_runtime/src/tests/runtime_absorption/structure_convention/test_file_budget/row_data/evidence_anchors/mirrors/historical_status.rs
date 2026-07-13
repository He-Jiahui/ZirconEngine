use super::*;

#[test]
fn runtime_15_evidence_anchors_historical_status_is_current() {
    let production_guard_support =
        read_runtime_src(RUNTIME_15_M3_PRODUCTION_GUARD_SUPPORT_ROW_DATA_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let historical_status_anchors = [
        VARIABLE_EVIDENCE_STATUS_NAME,
        VARIABLE_EVIDENCE_STATUS_ID,
        "plan_status/status_output_tables/expected_status_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
        VARIABLE_EVIDENCE_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 production support row data",
            production_guard_support.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &historical_status_anchors);
    }
}
