use super::*;

#[test]
fn runtime_15_review_guard_source_inventory_status_is_mirrored() {
    let status_rows = read_review_guard_structure_rows();
    let status_map = read_status_review_foundation_sources();
    let date_map = read_date_review_foundation_sources();
    let frameworks_02 = read_repo(
        "docs/plans/zircon_runtime/frameworks/02/2026-07-09-module-kernel-and-lifecycle-unification-output-records.md",
    );

    assert_contains_all(
        "review-guard source inventory row data",
        &status_rows,
        &[
            SOURCES_SLICE,
            SOURCES_STATUS,
            SOURCES_ROUTE_PATH,
            SOURCES_CHILDREN[0],
            SOURCES_CHILDREN[1],
            SOURCES_CHILDREN[2],
            SOURCES_CHILDREN[3],
            SOURCES_CHILDREN[4],
            SOURCES_CHILDREN[5],
            SOURCES_CHILDREN[6],
            SOURCES_CHILDREN[7],
            SOURCES_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status review-guard source inventory map",
        &status_map,
        &[SOURCES_SLICE, SOURCES_STATUS],
    );
    assert_contains_all(
        "date review-guard source inventory map",
        &date_map,
        &[SOURCES_SLICE, "Some(\"2026-07-06\")"],
    );

    assert_contains_all(
        "review-guard structure row-data split row",
        &status_rows,
        &[
            REVIEW_GUARD_STRUCTURE_ROW_DATA_SLICE,
            REVIEW_GUARD_STRUCTURE_ROW_DATA_STATUS,
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/structure_guard_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/typed_error_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/root_route_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/guard_body_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/source_inventory_rows.rs",
            "plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/review_guard_structure/row_data_owner_rows.rs",
            REVIEW_GUARD_STRUCTURE_ROW_DATA_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Frameworks 02 review-guard structure row-data mirror",
        &frameworks_02,
        &[REVIEW_GUARD_STRUCTURE_ROW_DATA_FRAMEWORKS_STATUS],
    );
}
