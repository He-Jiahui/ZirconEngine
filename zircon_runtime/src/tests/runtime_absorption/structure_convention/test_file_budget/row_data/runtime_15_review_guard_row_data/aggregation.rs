use super::*;

#[path = "aggregation/review_guard_splits.rs"]
mod review_guard_splits;
#[path = "aggregation/runtime_15_m3_root.rs"]
mod runtime_15_m3_root;
#[path = "aggregation/runtime_15_root.rs"]
mod runtime_15_root;
#[path = "aggregation/status_current.rs"]
mod status_current;
#[path = "aggregation/top_level_rows.rs"]
mod top_level_rows;

#[test]
fn runtime_15_review_guard_row_data_aggregation_guard_is_child_backed() {
    let route = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_row_data/aggregation.rs",
    );

    for (module, path, guard) in REVIEW_GUARD_ROW_DATA_AGGREGATION_CHILDREN {
        assert_contains_all(
            "review-guard row-data aggregation route mounts child",
            &route,
            &[
                format!("#[path = \"aggregation/{module}.rs\"]").as_str(),
                format!("mod {module};").as_str(),
            ],
        );
        let child = read_runtime_src(path);
        assert!(
            child.contains(guard),
            "{path} should own review-guard row-data aggregation guard {guard}"
        );
        assert!(
            child.lines().count() < 100,
            "{path} should stay focused after aggregation child split"
        );
    }
    assert!(
        route.lines().count() < 45,
        "aggregation.rs should remain a route/test-entry owner after child split"
    );
}
