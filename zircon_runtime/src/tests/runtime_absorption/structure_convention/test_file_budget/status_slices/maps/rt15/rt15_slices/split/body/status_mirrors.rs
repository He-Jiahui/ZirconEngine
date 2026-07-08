use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_maps_guard_body_status_mirrors_are_synced() {
    let row_data = read_top_level_support_row_sources();
    let status_map = read_status_support_status_map_sources();
    let date_map = read_status_support_date_map_sources();

    assert_contains_all(
        "Runtime 15 expected-slice maps guard row data",
        &row_data,
        &[
            SLICE,
            STATUS,
            PARENT_PATH,
            CHILD_OWNER_PATH,
            NAMING_BOUNDARY_PATH,
            SPLIT_LAYOUT_PATH,
            GUARD,
        ],
    );
    assert_contains_all(
        "Runtime 15 expected-slice maps guard-body row data",
        &row_data,
        &[
            GUARD_BODY_SLICE,
            GUARD_BODY_STATUS,
            SPLIT_LAYOUT_GUARD_BODY_PATH,
            GUARD_BODY_CHILDREN[0],
            GUARD_BODY_CHILDREN[1],
            GUARD_BODY_CHILDREN[2],
            GUARD_BODY_CHILDREN[3],
            GUARD_BODY_CHILDREN[4],
            GUARD_BODY_CHILDREN[5],
            GUARD_BODY_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "Runtime 15 expected-slice maps status map",
        &status_map,
        &[SLICE, STATUS, GUARD_BODY_SLICE, GUARD_BODY_STATUS],
    );
    assert_contains_all(
        "Runtime 15 expected-slice maps date map",
        &date_map,
        &[SLICE, "2026-07-05", GUARD_BODY_SLICE, "2026-07-06"],
    );
}
