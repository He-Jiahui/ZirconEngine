use super::*;

pub(super) fn assert_expected_slice_owner_paths_status_is_current() {
    let status_rows = read_runtime_src(PRODUCTION_GUARD_SUPPORT_STATUS_SUPPORT_PRIORITY_ROWS_PATH);
    let status_map = read_runtime_src(STATUS_SUPPORT_ROW_DATA_STATUS_MAP_PATH);
    let date_map = read_runtime_src(STATUS_SUPPORT_ROW_DATA_DATE_MAP_PATH);
    let runtime_15_plan =
        read_repo("docs/plans/zircon_runtime/runtime/15-code-structure-and-module-conventions.md");
    let runtime_index = read_repo("docs/plans/zircon_runtime/runtime/index.md");
    let frameworks_plan = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );
    let review_findings = read_repo("docs/plans/engine-code-review-findings-2026-06.md");
    let structure_convention = read_repo("docs/plans/engine-code-structure-convention.md");
    let module_doc = read_repo("docs/zircon_runtime/structure/module-convention.md");

    let status_anchors = [
        EXPECTED_SLICE_OWNER_PATHS_FOLDER_BACKED_STATUS_NAME,
        EXPECTED_SLICE_OWNER_PATHS_FOLDER_BACKED_STATUS_ID,
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/base_and_top_level.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/route_metadata.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/structure_support.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/status_support_maps.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/review_guard_structure.rs",
        "structure_convention/test_file_budget/row_data/rt15_status_support/owner_paths/expected_slice_maps/warning_cleanup.rs",
        EXPECTED_SLICE_OWNER_PATHS_FOLDER_BACKED_GUARD_NAME,
        "Cargo gate deferred",
    ];
    for (label, source) in [
        ("Runtime 15 plan", runtime_15_plan.as_str()),
        ("Runtime index", runtime_index.as_str()),
        ("Frameworks 02 plan", frameworks_plan.as_str()),
        ("review findings", review_findings.as_str()),
        ("structure convention", structure_convention.as_str()),
        ("module convention doc", module_doc.as_str()),
        ("production guard status rows", status_rows.as_str()),
    ] {
        assert_contains_all(label, source, &status_anchors);
    }
    assert_contains_all(
        "M3 status-support status map records expected-slice owner paths split",
        &status_map,
        &[
            EXPECTED_SLICE_OWNER_PATHS_FOLDER_BACKED_STATUS_NAME,
            EXPECTED_SLICE_OWNER_PATHS_FOLDER_BACKED_STATUS_ID,
        ],
    );
    assert_contains_all(
        "M3 status-support date map records expected-slice owner paths split",
        &date_map,
        &[
            EXPECTED_SLICE_OWNER_PATHS_FOLDER_BACKED_STATUS_NAME,
            "2026-07-07",
        ],
    );
}
