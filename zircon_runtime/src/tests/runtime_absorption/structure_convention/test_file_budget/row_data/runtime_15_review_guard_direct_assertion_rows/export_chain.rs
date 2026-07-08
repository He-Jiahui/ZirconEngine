use super::root_child_rows::DIRECT_ASSERTION_EXPORT_CHAIN_CHILDREN;
use super::*;

#[path = "export_chain/code_review_rows.rs"]
mod code_review_rows;
#[path = "export_chain/review_guard_row_data.rs"]
mod review_guard_row_data;
#[path = "export_chain/review_guard_splits.rs"]
mod review_guard_splits;
#[path = "export_chain/runtime_aggregation.rs"]
mod runtime_aggregation;
#[path = "export_chain/status_current.rs"]
mod status_current;

#[test]
fn runtime_15_review_guard_direct_assertion_export_chain_guard_is_child_backed() {
    let route = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/export_chain.rs",
    );

    for (module, path, guard) in DIRECT_ASSERTION_EXPORT_CHAIN_CHILDREN {
        assert_contains_all(
            "direct-assertion export-chain route mounts child",
            &route,
            &[
                format!("#[path = \"export_chain/{module}.rs\"]").as_str(),
                format!("mod {module};").as_str(),
            ],
        );
        let child = read_runtime_src(path);
        assert!(
            child.contains(guard),
            "{path} should own direct-assertion export-chain guard {guard}"
        );
        assert!(
            child.lines().count() < 110,
            "{path} should stay focused after export-chain child split"
        );
    }
    assert!(
        route.lines().count() < 45,
        "export_chain.rs should remain a route/test-entry owner after child split"
    );
}
