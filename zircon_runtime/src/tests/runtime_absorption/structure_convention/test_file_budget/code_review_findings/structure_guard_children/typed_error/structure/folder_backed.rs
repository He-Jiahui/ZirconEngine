use super::*;

pub(super) fn assert_structure_assertions_guard_is_folder_backed() {
    let parent = read_runtime_src(STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER);
    let children = structure_assertion_guard_child_source_blob();

    assert_contains_all(
        "typed-error structure assertions guard parent mounts focused children",
        &parent,
        &[
            "#[path = \"structure/source_trees.rs\"]",
            "mod source_trees;",
            "#[path = \"structure/current_checks.rs\"]",
            "mod current_checks;",
            "#[path = \"structure/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"structure/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"structure/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILDREN",
            "current_checks::assert_typed_error_structure_assertion_checks_are_current",
        ],
    );
    assert_contains_all(
        "typed-error structure assertions guard children own source trees, checks, budgets, and status mirrors",
        &children,
        &[
            "typed_error_structure_assertions_child_tree",
            "assert_typed_error_structure_assertion_checks_are_current",
            "assert_structure_assertions_guard_is_folder_backed",
            "assert_structure_assertions_guard_child_budgets",
            "assert_structure_assertions_guard_status_mirrors_are_current",
            STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_FOLDER_BACKED_GUARD,
        ],
    );

    for forbidden in [
        "typed-error structure assertions subtree keeps typed-error mount checks",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders.rs",
        "typed_error_native_plugin_loader_route_child_tree",
    ] {
        assert!(
            !parent.contains(forbidden),
            "{STRUCTURE_GUARD_TYPED_ERROR_STRUCTURE_ASSERTIONS_CHILD_OWNER} should not own nested typed-error guard detail `{forbidden}`"
        );
    }
}

#[test]
fn runtime_15_code_review_findings_structure_guard_typed_error_structure_assertions_guard_is_folder_backed(
) {
    assert_structure_assertions_guard_is_folder_backed();
}
