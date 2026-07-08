use super::*;

#[test]
fn runtime_15_status_output_runtime_15_expected_slice_child_owner_literal_ownership_is_folder_backed(
) {
    let parent = read_child_owner("literal_ownership.rs");
    let children = LITERAL_OWNERSHIP_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "Runtime 15 expected-slice child-owner literal route",
        &parent,
        &[
            "#[path = \"literal/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"literal/date_literals.rs\"]",
            "mod date_literals;",
            "#[path = \"literal/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"literal/paths.rs\"]",
            "mod paths;",
            "#[path = \"literal/source_groups.rs\"]",
            "mod source_groups;",
            "#[path = \"literal/status_literals.rs\"]",
            "mod status_literals;",
            "#[path = \"literal/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "use paths::*;",
            "use source_groups::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "STATUS_CHILD_SOURCE_PATHS",
        "DATE_CHILD_SOURCE_PATHS",
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "owners/literal_ownership.rs should delegate moved body anchor {moved_anchor}"
        );
    }
    assert_contains_all(
        "Runtime 15 expected-slice child-owner literal children",
        &children,
        &[
            "runtime_15_expected_slice_child_literals_stay_child_owned",
            "runtime_15_expected_slice_child_date_literals_stay_child_owned",
            LITERAL_OWNERSHIP_GUARD,
            "runtime_15_expected_slice_child_literal_ownership_status_mirrors_are_synced",
        ],
    );
}
