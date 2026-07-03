use super::*;

const STATUS_SUPPORT_ROWS_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/production_guard_support.rs";
const STATUS_SUPPORT_STATUS_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps.rs";
const STATUS_SUPPORT_DATE_MAP_PATH: &str =
    "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps.rs";

#[test]
fn runtime_15_review_guard_row_data_folder_backed_status_mirrors_are_current() {
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");
    let session_note =
        read_repo(".codex/sessions/20260612-0847-runtime-architecture-implementation.md");
    let status_support_rows = read_runtime_src(STATUS_SUPPORT_ROWS_PATH);
    let status_support_status_map = read_runtime_src(STATUS_SUPPORT_STATUS_MAP_PATH);
    let status_support_date_map = read_runtime_src(STATUS_SUPPORT_DATE_MAP_PATH);

    let folder_backed_status_anchors = [
        FOLDER_BACKED_STATUS_NAME,
        FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/delegation.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/moved_rows.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/aggregation.rs",
        "structure_convention/test_file_budget/status_output_row_data/runtime_15_review_guard_row_data/status_mirrors.rs",
        FOLDER_BACKED_GUARD_NAME,
        CHILD_OWNER_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("session note", session_note.as_str()),
        (
            "production guard support rows",
            status_support_rows.as_str(),
        ),
    ] {
        assert_contains_all(label, source, &folder_backed_status_anchors);
    }
    assert_contains_all(
        "Runtime 15 status-support expected status map records review-guard row-data folder-backed split",
        &status_support_status_map,
        &[FOLDER_BACKED_STATUS_NAME, FOLDER_BACKED_STATUS_ID],
    );
    assert_contains_all(
        "Runtime 15 status-support expected date map records review-guard row-data folder-backed split",
        &status_support_date_map,
        &[FOLDER_BACKED_STATUS_NAME, "2026-07-02"],
    );
}
