use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_budgets_are_folder_backed() {
    let parent = read_runtime_src(&format!(
        "tests/runtime_absorption/{STRUCTURE_SUPPORT_BUDGETS_ROUTE_PATH}"
    ));
    let children = STRUCTURE_SUPPORT_BUDGET_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "structure-support expected-slice budget route owner",
        &parent,
        &[
            "#[path = \"budgets/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"budgets/paths.rs\"]",
            "mod paths;",
            "#[path = \"budgets/route_child_budgets.rs\"]",
            "mod route_child_budgets;",
            "#[path = \"budgets/source_budgets.rs\"]",
            "mod source_budgets;",
            "#[path = \"budgets/status_docs.rs\"]",
            "mod status_docs;",
            "use paths::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STRUCTURE_SUPPORT_EXPECTED_SLICE_CHILDREN",
        "STATUS_REVIEW_FOUNDATION_CHILD",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "structure/budgets.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "structure-support expected-slice budget children",
        &children,
        &[
            STRUCTURE_SUPPORT_BUDGETS_GUARD,
            "runtime_15_structure_support_expected_slice_sources_stay_within_budget",
            "runtime_15_structure_support_expected_slice_route_children_stay_within_budget",
            "runtime_15_structure_support_expected_slice_budgets_status_is_synced",
        ],
    );
}
