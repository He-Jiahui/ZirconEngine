use super::*;

#[test]
fn runtime_15_status_output_m3_child_group_status_docs_are_child_owner() {
    let status_docs_source = format!(
        "{}\n{}",
        read_runtime_src(STATUS_DOCS_GUARD_PATH),
        status_docs_child_source_blob()
    );
    let status_row_docs_source = status_row_docs_guard_source();
    let status_rows = format!(
        "{}\n{}",
        read_runtime_src(STATUS_SUPPORT_CORE_AND_EVIDENCE_ROWS_PATH),
        read_runtime_src(STATUS_SUPPORT_STATUS_DOCS_ROWS_PATH)
    );
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    assert_contains_all(
        "M3 child-group status-doc guard records this split",
        &status_docs_source,
        &[
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
            HISTORICAL_GUARD_NAME,
        ],
    );
    assert_contains_all(
        "Runtime 15 M3 child-group status-row-doc child owns row status/doc anchors",
        &status_row_docs_source,
        &[
            &format!("fn {STATUS_ROW_DOC_GUARD_NAME}"),
            STATUS_ROW_DOC_STATUS_NAME,
            STATUS_ROW_DOC_STATUS_ID,
        ],
    );

    let historical_status_anchors = [
        ROW_DATA_STATUS_NAME,
        ROW_DATA_STATUS_ID,
        HISTORICAL_STATUS_NAME,
        HISTORICAL_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_groups.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_m3_child_group_status_docs.rs",
        HISTORICAL_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        (
            "status-output Runtime 15 M3 production support status-doc rows",
            status_rows.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &historical_status_anchors);
    }
}
