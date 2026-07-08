use super::*;

#[test]
fn runtime_15_structure_support_expected_slice_literal_ownership_is_folder_backed() {
    let parent = read_literal_owner_source("literal_ownership.rs");
    let children = LITERAL_OWNERSHIP_CHILDREN
        .iter()
        .map(|path| read_runtime_src(&format!("tests/runtime_absorption/{path}")))
        .collect::<Vec<_>>()
        .join("\n");

    assert_contains_all(
        "structure-support literal ownership parent mounts children",
        &parent,
        &[
            "#[path = \"literal/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"literal/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"literal/naming_literals.rs\"]",
            "mod naming_literals;",
            "#[path = \"literal/paths.rs\"]",
            "mod paths;",
            "#[path = \"literal/review_literals.rs\"]",
            "mod review_literals;",
            "#[path = \"literal/status_support_literals.rs\"]",
            "mod status_support_literals;",
            "use paths::*;",
            "use sources::*;",
        ],
    );
    for moved_anchor in [
        "#[test]",
        "fn runtime_15_structure_support_expected_slice_literals_are_child_owned",
        "STATUS_SUPPORT_PLAN_DOC_ROUTE_CHILDREN",
        "DATE_SUPPORT_PLAN_DOC_ROUTE_CHILDREN",
        LITERAL_OWNERSHIP_GUARD,
    ] {
        assert!(
            !parent.contains(moved_anchor),
            "structure-support literal ownership parent should delegate `{moved_anchor}`"
        );
    }
    assert_contains_all(
        "structure-support literal ownership children",
        &children,
        &[
            "runtime_15_structure_support_expected_slice_review_literals_are_child_owned",
            "runtime_15_structure_support_expected_slice_naming_literals_are_child_owned",
            "runtime_15_structure_support_expected_slice_status_support_literals_are_child_owned",
            "runtime_15_structure_support_expected_slice_literal_ownership_is_folder_backed",
            LITERAL_OWNERSHIP_GUARD,
        ],
    );
}
