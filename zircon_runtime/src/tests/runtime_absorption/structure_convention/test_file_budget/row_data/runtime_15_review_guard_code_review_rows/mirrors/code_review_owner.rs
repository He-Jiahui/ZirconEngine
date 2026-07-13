use super::*;

#[test]
fn runtime_15_review_guard_code_review_rows_status_mirrors_are_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let row_data_owner = read_runtime_src(ROW_DATA_OWNER_PATH);
    let review_status_map = review_guard_status_map_source_blob();
    let review_date_map = review_guard_date_map_source_blob();

    let status_anchors = [
        CODE_REVIEW_ROWS_ROW_DATA_STATUS_NAME,
        CODE_REVIEW_ROWS_ROW_DATA_STATUS_ID,
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/review_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/structure_guard_rows.rs",
        "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/review_guard_splits/code_review_rows/typed_error_structure_rows.rs",
        CODE_REVIEW_ROWS_ROW_DATA_GUARD_NAME,
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("code-review row-data owner", row_data_owner.as_str()),
        (
            "Runtime 15 review-guard status map",
            review_status_map.as_str(),
        ),
        ("Runtime 15 review-guard date map", review_date_map.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
}
