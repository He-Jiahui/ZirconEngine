use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_status_rows_are_synced() {
    let status_rows = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_status_row_data/runtime_15/m3/status_support/expected_slice_maps/structure_support/guard_rows.rs",
    );
    let status_map = read_status_structure_route_map_sources();
    let date_map = read_date_structure_route_map_sources();

    assert_contains_all(
        "structure-support guard row data",
        &status_rows,
        &[
            STRUCTURE_SUPPORT_GUARD_SLICE,
            STRUCTURE_SUPPORT_GUARD_STATUS,
            "review/structure_support_expected_slice.rs",
            "review/structure/status_mirrors.rs",
            STRUCTURE_SUPPORT_GUARD_NAME,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-support guard status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            STRUCTURE_SUPPORT_GUARD_SLICE,
            STRUCTURE_SUPPORT_GUARD_STATUS,
            "2026-07-05",
        ],
    );
    assert_contains_all(
        "structure-support status mirrors row data",
        &status_rows,
        &[
            STRUCTURE_SUPPORT_STATUS_MIRRORS_SLICE,
            STRUCTURE_SUPPORT_STATUS_MIRRORS_STATUS,
            STRUCTURE_SUPPORT_STATUS_MIRRORS_ROUTE_PATH,
            STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[0],
            STRUCTURE_SUPPORT_STATUS_MIRROR_CHILDREN[3],
            STRUCTURE_SUPPORT_STATUS_MIRRORS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-support status mirrors status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            STRUCTURE_SUPPORT_STATUS_MIRRORS_SLICE,
            STRUCTURE_SUPPORT_STATUS_MIRRORS_STATUS,
            "2026-07-07",
        ],
    );
}
