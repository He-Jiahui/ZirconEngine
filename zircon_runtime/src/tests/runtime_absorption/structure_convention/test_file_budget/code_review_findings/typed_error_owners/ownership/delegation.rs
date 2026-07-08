use super::super::super::super::*;
use super::*;

pub(super) fn assert_typed_error_child_ownership_is_folder_backed(
    sources: &TypedErrorChildOwnershipSources,
) {
    let child_ownership_parent = read_runtime_src(TYPED_ERROR_CHILD_OWNERSHIP_CHILD);
    let child_inventory = read_runtime_src(TYPED_ERROR_CHILD_OWNERSHIP_ROOT_CHILD_ROWS_CHILD);
    let child_tree = typed_error_child_ownership_child_source_blob();

    assert_contains_all(
        "code review findings structure guard parent mounts typed-error child owner",
        &sources.parent,
        &[
            "#[path = \"code_review_findings/typed_error_child_owners.rs\"]",
            "mod typed_error_child_owners;",
        ],
    );
    assert_contains_all(
        "typed-error child-ownership parent delegates focused children",
        &child_ownership_parent,
        &[
            "#[path = \"ownership/budgets.rs\"]",
            "mod budgets;",
            "#[path = \"ownership/delegation.rs\"]",
            "mod delegation;",
            "#[path = \"ownership/review_guards.rs\"]",
            "mod review_guards;",
            "#[path = \"ownership/status_mirrors.rs\"]",
            "mod status_mirrors;",
            "#[path = \"ownership/structure_subtree.rs\"]",
            "mod structure_subtree;",
            "#[path = \"ownership/root_paths.rs\"]",
            "mod root_paths;",
            "#[path = \"ownership/root_statuses.rs\"]",
            "mod root_statuses;",
            "#[path = \"ownership/root_child_rows.rs\"]",
            "mod root_child_rows;",
            "#[path = \"ownership/root_sources.rs\"]",
            "mod root_sources;",
            "#[path = \"ownership/root_inventory.rs\"]",
            "mod root_inventory;",
            "delegation::assert_typed_error_child_ownership_is_folder_backed",
            "structure_subtree::assert_typed_error_structure_subtree_is_child_owned",
            "review_guards::assert_typed_error_review_guards_are_preserved",
            "budgets::assert_typed_error_child_ownership_budgets_are_focused",
        ],
    );
    for moved_anchor in [
        "review_f5_texture_loader_uses_typed_error",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/abi_surfaces/plugin_descriptor/string_helpers.rs",
        "assert_contains_all(\n        \"typed-error moved-guard child owns review guard preservation checks\"",
    ] {
        assert!(
            !child_ownership_parent.contains(moved_anchor),
            "typed-error child-ownership parent should delegate moved anchor `{moved_anchor}` to focused children"
        );
    }
    for (_, child_path, anchor) in TYPED_ERROR_CHILD_OWNERSHIP_CHILDREN {
        assert!(
            child_inventory.contains(child_path),
            "typed-error child-ownership root child inventory should list {child_path}"
        );
        assert!(
            child_tree.contains(anchor),
            "typed-error child-ownership child {child_path} should own anchor {anchor}"
        );
    }
}

#[test]
fn runtime_15_typed_error_child_ownership_guard_is_folder_backed() {
    let sources = typed_error_child_ownership_sources();
    assert_typed_error_child_ownership_is_folder_backed(&sources);
}
