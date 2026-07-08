use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_status_mirrors_are_synced(
) {
    let status_rows = read_route_metadata_row_sources();
    let status_map = read_status_support_status_map_sources();
    let date_map = read_status_support_date_map_sources();

    assert_contains_all(
        "status child-owner budget route metadata row data",
        &status_rows,
        &[
            BUDGET_SLICE,
            BUDGET_STATUS,
            BUDGETS_ROUTE_PATH,
            BUDGETS_SOURCES_PATH,
            BUDGETS_GUARD_BODY_PATH,
            BUDGETS_ROUTE_METADATA_PATH,
            BUDGET_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status child-owner budget route metadata split row data",
        &status_rows,
        &[
            BUDGET_ROUTE_METADATA_SLICE,
            BUDGET_ROUTE_METADATA_STATUS,
            BUDGETS_ROUTE_METADATA_PATH,
            BUDGET_ROUTE_METADATA_CHILDREN[0],
            BUDGET_ROUTE_METADATA_CHILDREN[1],
            BUDGET_ROUTE_METADATA_CHILDREN[2],
            BUDGET_ROUTE_METADATA_CHILDREN[3],
            BUDGET_ROUTE_METADATA_CHILDREN[4],
            BUDGET_ROUTE_METADATA_CHILDREN[5],
            BUDGET_ROUTE_METADATA_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "status child-owner budget route metadata map",
        &status_map,
        &[
            BUDGET_SLICE,
            BUDGET_STATUS,
            BUDGET_ROUTE_METADATA_SLICE,
            BUDGET_ROUTE_METADATA_STATUS,
        ],
    );
    assert_contains_all(
        "date child-owner budget route metadata map",
        &date_map,
        &[
            BUDGET_SLICE,
            "Some(\"2026-07-06\")",
            BUDGET_ROUTE_METADATA_SLICE,
            "Some(\"2026-07-06\")",
        ],
    );
}
