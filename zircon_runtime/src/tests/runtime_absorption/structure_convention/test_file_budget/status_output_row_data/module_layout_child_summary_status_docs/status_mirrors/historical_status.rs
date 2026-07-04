use super::*;

#[test]
fn runtime_15_module_layout_child_summary_status_docs_are_child_owner() {
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
    let status_doc_guard_source = format!(
        "{}\n{}",
        read_runtime_src(CHILD_SUMMARY_STATUS_DOCS_GUARD_PATH),
        child_summary_status_doc_child_source_blob()
    );

    assert_contains_all(
        "module-layout child-summary status-doc guard records this split",
        &status_doc_guard_source,
        &[
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
            HISTORICAL_GUARD_NAME,
        ],
    );
    assert_contains_all(
        "production support row data",
        &production_guard_support,
        &[
            ROW_DATA_GUARD_STATUS_NAME,
            ROW_DATA_GUARD_STATUS_ID,
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
            "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries.rs",
            "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs.rs",
            ROW_DATA_GUARD_NAME,
            HISTORICAL_GUARD_NAME,
        ],
    );
    assert_contains_all(
        "expected status map",
        &expected_status_map,
        &[
            ROW_DATA_GUARD_STATUS_NAME,
            ROW_DATA_GUARD_STATUS_ID,
            HISTORICAL_STATUS_NAME,
            HISTORICAL_STATUS_ID,
        ],
    );
    assert_contains_all(
        "expected date map",
        &expected_date_map,
        &[
            ROW_DATA_GUARD_STATUS_NAME,
            HISTORICAL_STATUS_NAME,
            "2026-06-30",
        ],
    );
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
    ] {
        assert_contains_all(
            label,
            source,
            &[
                ROW_DATA_GUARD_STATUS_NAME,
                ROW_DATA_GUARD_STATUS_ID,
                HISTORICAL_STATUS_NAME,
                HISTORICAL_STATUS_ID,
                "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summaries.rs",
                "structure_convention/test_file_budget/status_output_row_data/module_layout_child_summary_status_docs.rs",
                ROW_DATA_GUARD_NAME,
                HISTORICAL_GUARD_NAME,
            ],
        );
    }
}
