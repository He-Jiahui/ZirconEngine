use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_literal_ownership_status_rows_are_synced() {
    let status_rows = read_structure_support_expected_slice_rows();
    let status_map = read_status_structure_route_map_sources();
    let date_map = read_date_structure_route_map_sources();

    assert_contains_all(
        "structure-support literal ownership row data",
        &status_rows,
        &[
            LITERAL_OWNERSHIP_SLICE,
            LITERAL_OWNERSHIP_STATUS,
            LITERAL_OWNERSHIP_PARENT_PATH,
            LITERAL_OWNERSHIP_CHILDREN[0],
            LITERAL_OWNERSHIP_CHILDREN[7],
            LITERAL_OWNERSHIP_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-support literal ownership status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            LITERAL_OWNERSHIP_SLICE,
            LITERAL_OWNERSHIP_STATUS,
            "2026-07-06",
        ],
    );
    assert_contains_all(
        "structure-support literal ownership status mirrors row data",
        &status_rows,
        &[
            LITERAL_STATUS_MIRRORS_SLICE,
            LITERAL_STATUS_MIRRORS_STATUS,
            LITERAL_STATUS_MIRRORS_ROUTE_PATH,
            LITERAL_STATUS_MIRROR_CHILDREN[0],
            LITERAL_STATUS_MIRROR_CHILDREN[3],
            LITERAL_STATUS_MIRRORS_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-support literal ownership status mirrors status/date maps",
        &format!("{status_map}\n{date_map}"),
        &[
            LITERAL_STATUS_MIRRORS_SLICE,
            LITERAL_STATUS_MIRRORS_STATUS,
            "2026-07-07",
        ],
    );
}
