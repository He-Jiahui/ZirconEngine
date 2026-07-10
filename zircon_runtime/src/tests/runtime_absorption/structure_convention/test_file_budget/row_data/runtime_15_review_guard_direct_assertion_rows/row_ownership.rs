use super::*;
#[path = "row_ownership/child_owner_rows.rs"]
mod child_owner_rows;
#[path = "row_ownership/folder_backed_rows.rs"]
mod folder_backed_rows;
#[path = "row_ownership/status_current.rs"]
mod status_current;

#[test]
fn runtime_15_review_guard_direct_assertion_row_ownership_guard_is_child_backed() {
    let route = read_runtime_src(
        "tests/runtime_absorption/structure_convention/test_file_budget/row_data/runtime_15_review_guard_direct_assertion_rows/row_ownership.rs",
    );

    for (module, path, guard) in DIRECT_ASSERTION_ROW_OWNERSHIP_CHILDREN {
        assert_contains_all(
            "direct-assertion row-ownership route mounts child",
            &route,
            &[
                format!("#[path = \"row_ownership/{module}.rs\"]").as_str(),
                format!("mod {module};").as_str(),
            ],
        );

        let child = read_runtime_src(path);
        assert!(
            child.contains(guard),
            "{path} should own direct-assertion row-ownership guard {guard}"
        );
        assert!(
            child.lines().count() < 100,
            "{path} should stay focused after direct-assertion row-ownership child split"
        );
    }
    assert!(
        route.lines().count() < 40,
        "row_ownership.rs should remain a route/test-entry owner after child split"
    );
}
