use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_parent_route_metadata_status_mirrors_are_synced() {
    let status_rows = read_structure_support_expected_slice_rows();
    let status_map = read_status_structure_route_map_sources();
    let date_map = read_date_structure_route_map_sources();

    assert_contains_all(
        "structure-support parent-route metadata child row",
        &status_rows,
        &[
            "Runtime 15 M3 structure-support expected-slice parent-route metadata child split",
            "runtime_15_structure_support_expected_slice_parent_route_metadata_child_split_static_passed_cargo_deferred",
            "runtime_15_structure_support_expected_slice_parent_route_metadata_is_child_owned",
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-support parent-route metadata folder-backed row",
        &status_rows,
        &[
            PARENT_ROUTE_METADATA_SLICE,
            PARENT_ROUTE_METADATA_STATUS,
            PARENT_ROUTE_METADATA_ROUTE_PATH,
            PARENT_ROUTE_METADATA_CHILDREN[0],
            PARENT_ROUTE_METADATA_CHILDREN[1],
            PARENT_ROUTE_METADATA_CHILDREN[2],
            PARENT_ROUTE_METADATA_CHILDREN[3],
            PARENT_ROUTE_METADATA_CHILDREN[4],
            PARENT_ROUTE_METADATA_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "structure-route status map",
        &status_map,
        &[
            "Runtime 15 M3 structure-support expected-slice parent-route metadata child split",
            "runtime_15_structure_support_expected_slice_parent_route_metadata_child_split_static_passed_cargo_deferred",
            PARENT_ROUTE_METADATA_SLICE,
            PARENT_ROUTE_METADATA_STATUS,
        ],
    );
    assert_contains_all(
        "structure-route date map",
        &date_map,
        &[
            "Runtime 15 M3 structure-support expected-slice parent-route metadata child split",
            PARENT_ROUTE_METADATA_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
