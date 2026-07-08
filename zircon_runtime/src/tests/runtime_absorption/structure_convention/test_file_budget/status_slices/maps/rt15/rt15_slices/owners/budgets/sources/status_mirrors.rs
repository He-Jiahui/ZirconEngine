use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_sources_status_mirrors_are_synced(
) {
    let rows = read_route_metadata_row_sources();
    let status_map = read_status_support_status_map_sources();
    let date_map = read_status_support_date_map_sources();

    assert_contains_all(
        "child-owner budget source inventory status row",
        &rows,
        &[
            BUDGET_SOURCE_SLICE,
            BUDGET_SOURCE_STATUS,
            BUDGETS_SOURCES_PATH,
            BUDGETS_SOURCES_CHILDREN[0],
            BUDGETS_SOURCES_CHILDREN[1],
            BUDGETS_SOURCES_CHILDREN[2],
            BUDGETS_SOURCES_CHILDREN[3],
            BUDGETS_SOURCES_CHILDREN[4],
            BUDGETS_SOURCES_CHILDREN[5],
            BUDGET_SOURCE_GUARD,
            "Cargo gate deferred",
        ],
    );
    assert_contains_all(
        "child-owner budget source inventory status map",
        &status_map,
        &[BUDGET_SOURCE_SLICE, BUDGET_SOURCE_STATUS],
    );
    assert_contains_all(
        "child-owner budget source inventory date map",
        &date_map,
        &[BUDGET_SOURCE_SLICE, "Some(\"2026-07-06\")"],
    );
}
