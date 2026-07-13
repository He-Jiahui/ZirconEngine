use super::*;

#[test]
fn runtime_15_foundation_row_data_status_doc_mirrors_are_current() {
    let status_rows = read_runtime_src(STATUS_SUPPORT_ROWS_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    for (label, source) in [
        ("Runtime 15 status rows", status_rows.as_str()),
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
                FOUNDATION_ROW_DATA_SPLIT_NAME,
                FOUNDATION_ROW_DATA_SPLIT_ID,
                FOUNDATION_TOPIC_SPLIT_NAME,
                FOUNDATION_TOPIC_SPLIT_ID,
                "plan_status/status_output_tables/expected_status_row_data/runtime_15.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/core_rows.rs",
                "plan_status/status_output_tables/expected_status_row_data/runtime_15/foundation/typed_error_runtime_rows.rs",
                "runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
            ],
        );
    }

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
                FOUNDATION_GUARD_SPLIT_NAME,
                FOUNDATION_GUARD_SPLIT_ID,
                "structure_convention/test_file_budget/row_data/runtime_15_row_data.rs",
                "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data.rs",
                "runtime_15_status_output_runtime_15_foundation_row_data_is_child_owner",
            ],
        );
        assert_contains_all(
            label,
            source,
            &[
                STATUS_DOC_SPLIT_NAME,
                STATUS_DOC_SPLIT_ID,
                "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data.rs",
                "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status_docs.rs",
                "runtime_15_status_output_foundation_row_data_status_docs_are_child_owner",
            ],
        );
        assert_contains_all(
            label,
            source,
            &[
                FOLDER_BACKED_STATUS_NAME,
                FOLDER_BACKED_STATUS_ID,
                "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/delegation.rs",
                "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/status_maps.rs",
                "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/doc_mirrors.rs",
                "structure_convention/test_file_budget/row_data/runtime_15_foundation_row_data_status/row_count.rs",
            ],
        );
    }

    for (path, source) in status_doc_child_sources().into_iter().chain([(
        STATUS_DOCS_PARENT_PATH,
        read_runtime_src(STATUS_DOCS_PARENT_PATH),
    )]) {
        let line_count = source.lines().count();
        assert!(
            line_count < 400,
            "{path} should stay below the focused Runtime 15 foundation row-data status-doc guard budget; got {line_count} lines"
        );
    }
}
