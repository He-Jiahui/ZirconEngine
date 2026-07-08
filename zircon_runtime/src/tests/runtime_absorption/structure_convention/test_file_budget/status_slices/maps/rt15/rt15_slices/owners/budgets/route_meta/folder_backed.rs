use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_is_folder_backed(
) {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{BUDGETS_ROUTE_METADATA_PATH}"
    ));
    let children = read_budget_route_metadata_children();

    assert_contains_all(
        "Runtime 15 expected-slice budget route metadata route",
        &parent,
        &[
            "#[path = \"route_meta/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"route_meta/doc_mirrors.rs\"]",
            "mod doc_mirrors;",
            "#[path = \"route_meta/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"route_meta/paths.rs\"]",
            "mod paths;",
            "#[path = \"route_meta/route_mounts.rs\"]",
            "mod route_mounts;",
            "#[path = \"route_meta/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
        ],
    );
    for moved_anchor in ["#[test]", "ROUTE_METADATA_ROWS_PATH", BUDGET_GUARD] {
        assert!(
            !parent.contains(moved_anchor),
            "owners/budgets/route_metadata.rs should delegate moved budget metadata anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "Runtime 15 expected-slice budget route metadata children",
        &children,
        &[
            BUDGET_GUARD,
            BUDGET_ROUTE_METADATA_GUARD,
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_is_child_owned",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_children_stay_budgeted",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_docs_are_synced",
            "runtime_15_status_output_runtime_15_expected_slice_child_owner_budget_route_metadata_status_mirrors_are_synced",
        ],
    );
}
