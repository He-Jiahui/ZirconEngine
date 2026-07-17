use super::*;

#[test]
fn runtime_15_plugin_importer_status_output_guard_folder_backed_status_is_current() {
    let status_rows = status_support_review_guard_source_blob();
    let status_map = status_support_status_map_source_blob();
    let date_map = status_support_date_map_source_blob();
    let runtime_15_plan =
        read_repo(
            "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-code-structure-and-module-conventions-output-records.md",
        );
    let runtime_index = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-runtime-index-output-records.md",
    );
    let review_findings = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-review-findings-output-records.md",
    );
    let structure_convention = read_repo(
        "docs/plans/_archive/zircon_runtime/runtime/15/2026-07-09-engine-code-structure-output-records.md",
    );
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let status_anchors = [
        PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_STATUS_NAME,
        PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_STATUS_ID,
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/delegation.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/row_children.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/row_data_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/status_mirrors.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/child_split_status.rs",
        "structure_convention/test_file_budget/row_data/runtime_15_review_guard_code_review_rows/plugin_importer_rows/budgets.rs",
        PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("production guard review rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 status-support map records plugin-importer status-output guard split",
        &status_map,
        &[
            PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_STATUS_NAME,
            PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records plugin-importer status-output guard split",
        &date_map,
        &[
            PLUGIN_IMPORTER_STATUS_OUTPUT_GUARD_STATUS_NAME,
            "2026-07-04",
        ],
    );
}
