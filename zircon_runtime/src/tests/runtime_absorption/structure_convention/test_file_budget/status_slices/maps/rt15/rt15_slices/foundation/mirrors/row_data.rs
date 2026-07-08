use super::*;

#[test]
fn runtime_15_foundation_expected_slice_maps_status_mirrors_are_synced() {
    let row_data = read_top_level_support_row_sources();
    let status_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/status/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/expected_slice_guard_maps.rs",
    );
    let date_map = read_runtime_src(
        "tests/runtime_absorption/plan_status/status_output_tables/expected_slices/date/runtime_15/m3_structure_support/status_support_maps/m3_m4_expected_slice_maps/expected_slice_guard_maps.rs",
    );

    assert_contains_all(
        "Runtime 15 foundation expected-slice map row data",
        &row_data,
        &[
            FOUNDATION_MAP_SLICE,
            FOUNDATION_MAP_STATUS,
            "plan_status/status_output_tables/expected_slices/status/runtime_15/foundation.rs",
            "plan_status/status_output_tables/expected_slices/status/runtime_15/foundation/lock_poison.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/foundation.rs",
            "plan_status/status_output_tables/expected_slices/date/runtime_15/foundation/typed_error_plugin.rs",
            FOUNDATION_MAP_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 foundation expected-slice map guard row data",
        &row_data,
        &[
            FOUNDATION_GUARD_SLICE,
            FOUNDATION_GUARD_STATUS,
            "structure_convention/test_file_budget/status_slices/maps/rt15/rt15_slices/foundation.rs",
            FOUNDATION_GUARD_CHILDREN[0],
            FOUNDATION_GUARD_CHILDREN[1],
            FOUNDATION_GUARD_CHILDREN[2],
            FOUNDATION_GUARD_CHILDREN[3],
            FOUNDATION_GUARD_CHILDREN[4],
            FOUNDATION_GUARD_CHILDREN[5],
            FOUNDATION_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "foundation expected-slice map guard status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            FOUNDATION_GUARD_SLICE,
            FOUNDATION_GUARD_STATUS,
            FOUNDATION_STATUS_MIRRORS_SLICE,
            FOUNDATION_STATUS_MIRRORS_STATUS,
            "2026-07-07",
        ],
    );
    assert_contains_all(
        "Runtime 15 foundation expected-slice map status mirror row data",
        &row_data,
        &[
            FOUNDATION_STATUS_MIRRORS_SLICE,
            FOUNDATION_STATUS_MIRRORS_STATUS,
            FOUNDATION_STATUS_MIRRORS_PARENT,
            FOUNDATION_STATUS_MIRRORS_CHILDREN[0],
            FOUNDATION_STATUS_MIRRORS_CHILDREN[1],
            FOUNDATION_STATUS_MIRRORS_CHILDREN[2],
            FOUNDATION_STATUS_MIRRORS_CHILDREN[3],
            FOUNDATION_STATUS_MIRRORS_CHILDREN[4],
            FOUNDATION_STATUS_MIRRORS_GUARD,
            "Cargo gate deferred",
        ],
    );
}
