use super::super::super::super::super::super::super::*;
use super::super::super::super::*;

pub(super) fn assert_typed_error_source_inventory_delegation_is_child_backed() {
    let delegation_parent = read_runtime_src(TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILD);
    let delegation_children = typed_error_source_inventory_delegation_child_source_blob();

    assert_contains_all(
        "typed-error source inventory delegation parent mounts focused children",
        &delegation_parent,
        &[
            "#[path = \"delegation/folder_backed.rs\"]",
            "mod folder_backed;",
            "#[path = \"delegation/parent_delegation.rs\"]",
            "mod parent_delegation;",
            "#[path = \"delegation/source_inventory_mounts.rs\"]",
            "mod source_inventory_mounts;",
            "#[path = \"delegation/source_ownership.rs\"]",
            "mod source_ownership;",
            "#[path = \"delegation/status_current.rs\"]",
            "mod status_current;",
            "parent_delegation::assert_typed_error_structure_delegates_source_inventory",
            "source_inventory_mounts::assert_typed_error_source_inventory_parent_mounts_focused_owners",
            "source_ownership::assert_typed_error_source_inventory_paths_and_reads_are_child_owned",
            "folder_backed::assert_typed_error_source_inventory_guard_is_folder_backed",
        ],
    );
    for moved_anchor in [
        "const TYPED_ERROR_SOURCE_PATHS",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/asset_loaders/texture.rs",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/native_plugin_loader/live_host/replay_and_runtime/runtime_behavior.rs",
        "tests/runtime_absorption/code_review_findings/typed_error_convergence/shader_prewarm_cli/args_boundary.rs",
        "TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_SLICE",
        "TYPED_ERROR_SOURCE_INVENTORY_FOLDER_BACKED_STATUS",
    ] {
        assert!(
            !delegation_parent.contains(moved_anchor),
            "sources/delegation.rs should delegate `{moved_anchor}` to focused children"
        );
    }
    for (_, child_path, child_guard) in TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILDREN {
        assert!(
            delegation_children.contains(child_path),
            "typed-error source inventory delegation tree should inventory child path {child_path}"
        );
        assert!(
            delegation_children.contains(child_guard),
            "typed-error source inventory delegation child should own anchor {child_guard}"
        );
    }
    let mut budget_sources: Vec<(&'static str, String)> = vec![(
        TYPED_ERROR_SOURCE_INVENTORY_DELEGATION_CHILD,
        delegation_parent,
    )];
    budget_sources.extend(typed_error_source_inventory_delegation_child_sources());

    for (path, source) in budget_sources {
        let line_count = source.lines().count();
        assert!(
            line_count < TYPED_ERROR_CHILD_OWNER_LINE_BUDGET,
            "{path} should stay below the Runtime 15 test-file budget; got {line_count} lines"
        );
    }
}
